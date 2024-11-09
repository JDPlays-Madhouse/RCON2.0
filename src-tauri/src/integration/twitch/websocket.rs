use anyhow::{bail, Context, Error, Result};
use tokio_tungstenite::tungstenite;
use tracing::{debug, error, info, warn};
use tracing::{trace, Instrument};
use twitch_api::{
    client::ClientDefault,
    eventsub::{
        self,
        event::websocket::{EventsubWebsocketData, ReconnectPayload, SessionData, WelcomePayload},
        Event, EventType,
    },
    types::{self},
    HelixClient,
};
use twitch_oauth2::{TwitchToken, UserToken};

#[derive(Debug)]
pub enum WebSocketError {
    TokenElapsed,
    InvaildToken,
    FailedToConnect(String),
    FailedToRun(String),
    InvailURL(String),
    ProcessMessage(String),
}

impl WebSocketError {
    pub fn map_err<T, E>(self, result: Result<T, E>) -> Result<T, WebSocketError> {
        result.map_err(|_e| self)
    }
}

#[derive(Clone)]
pub struct WebsocketClient {
    pub session_id: Option<String>,
    pub token: UserToken,
    pub client: HelixClient<'static, reqwest::Client>,
    pub user_id: types::UserId,
    pub connect_url: url::Url,
    pub subscriptions: Vec<eventsub::EventType>,
}

impl std::fmt::Debug for WebsocketClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebsocketClient")
            .field("session_id", &self.session_id)
            .field("token", &self.token)
            .field("user_id", &self.user_id)
            .field("connect_url", &self.connect_url)
            .finish()
    }
}

impl WebsocketClient {
    pub fn new(
        session_id: Option<String>,
        token: UserToken,
        user_id: types::UserId,
        subscriptions: Vec<eventsub::EventType>,
    ) -> Self {
        let client: HelixClient<_> = twitch_api::HelixClient::with_client(
            <reqwest::Client>::default_client_with_name(Some(
                "twitch-rs/eventsub".parse().expect("parsing name"),
            ))
            .expect("Default client with name"),
        );

        Self {
            session_id,
            token,
            client,
            user_id,
            connect_url: twitch_api::TWITCH_EVENTSUB_WEBSOCKET_URL.clone(),
            subscriptions,
        }
    }

    /// Connect to the websocket and return the stream
    pub async fn connect(
        &self,
    ) -> Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        WebSocketError,
    > {
        tracing::info!("connecting to twitch");
        let config = tungstenite::protocol::WebSocketConfig {
            max_message_size: Some(64 << 20), // 64 MiB
            max_frame_size: Some(16 << 20),   // 16 MiB
            accept_unmasked_frames: false,
            ..tungstenite::protocol::WebSocketConfig::default()
        };
        match tokio_tungstenite::connect_async_with_config(&self.connect_url, Some(config), false)
            .await
        {
            Ok((socket, _)) => Ok(socket),
            Err(e) => {
                error!("{}", e);
                Err(WebSocketError::FailedToConnect("Can't Connect".into()))
            }
        }
    }

    /// Run the websocket subscriber
    #[tracing::instrument(name = "subscriber", skip_all, fields())]
    pub async fn run(mut self) -> Result<(), WebSocketError> {
        // Establish the stream
        let mut s = match self.connect().await {
            Ok(s) => s,
            Err(e) => return Err(e),
        };
        info!(
            target = "Twitch::Integration::Websocket",
            "Connected to Twitch's Websocket",
        );
        // Loop over the stream, processing messages as they come in.
        loop {
            tokio::select!(
            Some(msg) = futures::StreamExt::next(&mut s) => {
                let span = tracing::info_span!("message received", raw_message = ?msg);
                let msg = match msg {
                    Err(tungstenite::Error::Protocol(
                        tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
                    )) => {
                        tracing::warn!(
                            "connection was sent with an unexpected frame or was reset, reestablishing it"
                        );
                        s = WebSocketError::FailedToRun("when reestablishing connection".into()).map_err(self
                            .connect()
                            .instrument(span)
                            .await
                            ).unwrap();
                        continue
                    }
                    _ => msg.map_err(|_e| WebSocketError::FailedToConnect("When getting message".into()))?,
                };
                WebSocketError::FailedToRun("Processing Message".into()).map_err(self.process_message(msg)
                    .instrument(span)
                    .await).unwrap()
            })
        }
    }

    /// Process a message from the websocket
    pub async fn process_message(&mut self, msg: tungstenite::Message) -> Result<(), Error> {
        match msg {
            tungstenite::Message::Text(s) => {
                tracing::trace!("{s}");
                // Parse the message into a [twitch_api::eventsub::EventsubWebsocketData]
                match Event::parse_websocket(&s).unwrap() {
                    EventsubWebsocketData::Welcome {
                        payload: WelcomePayload { session },
                        ..
                    }
                    | EventsubWebsocketData::Reconnect {
                        payload: ReconnectPayload { session },
                        ..
                    } => {
                        self.process_welcome_message(session).await.unwrap();
                        Ok(())
                    }
                    // Here is where you would handle the events you want to listen to
                    EventsubWebsocketData::Notification {
                        metadata: _,
                        payload,
                    } => {
                        match payload {
                            Event::ChannelChatMessageV1(eventsub::Payload { message, .. }) => {
                                match message.clone() {
                                    eventsub::Message::Notification(chat_payload) => {
                                        let message = chat_payload.chatter_user_name.to_string()
                                            + " - "
                                            + &chat_payload.message.text;
                                        info!(
                                            target = "Twitch::websocket::ChannelChatMessage",
                                            message
                                        );
                                    }
                                    _ => {
                                        error! {"Unhandled Message Payload: {:?}", message}
                                    }
                                };
                            }
                            Event::ChannelPointsCustomRewardRedemptionAddV1(
                                eventsub::Payload { message, .. },
                            ) => match message.clone() {
                                eventsub::Message::Notification(reward_payload) => {
                                    let message = format!(
                                        "New: {}({}) redeemed by {}: {}",
                                        reward_payload.reward.title,
                                        reward_payload.reward.id,
                                        reward_payload.user_name,
                                        reward_payload.user_input
                                    );
                                    info!(target = "Twitch::websocket::ChannelPointsCustomRewardRedemptionAdd", message);
                                }
                                _ => {
                                    error! {"Unhandled ChannelPointsCustomRewardRedemptionAddV1 Payload: {:?}", message}
                                }
                            },
                            Event::ChannelPointsCustomRewardRedemptionUpdateV1(
                                eventsub::Payload { message, .. },
                            ) => match message.clone() {
                                eventsub::Message::Notification(reward_payload) => {
                                    let message = format!(
                                        "Update: {}({}) redeemed by {}: {}",
                                        reward_payload.reward.title,
                                        reward_payload.reward.id,
                                        reward_payload.user_name,
                                        reward_payload.user_input
                                    );

                                    info!(target = "Twitch::websocket::ChannelPointsCustomRewardRedemptionUpdate", message);
                                }
                                _ => {
                                    error! {"Unhandled ChannelPointsCustomRewardRedemptionUpdateV1 Payload: {:?}", message}
                                }
                            },

                            m => {
                                let message =
                                    format!("Received an unimplemented websocket event: {:?}", m);
                                error!(target = "Twitch::websocket::Event", message);
                            }
                        }
                        Ok(())
                    }
                    EventsubWebsocketData::Revocation {
                        metadata,
                        payload: _,
                    } => {
                        error!("got revocation event: {metadata:?}");
                        bail!("got revocation event: {metadata:?}")
                    }
                    EventsubWebsocketData::Keepalive {
                        metadata: _,
                        payload: _,
                    } => Ok(()),
                    data => {
                        error!("Unexpected EventsubWebsocketData {:?}", data);
                        bail!("Unexpected EventsubWebsocketData {:?}", data)
                    }
                }
            }
            tungstenite::Message::Close(close) => {
                error!("Close frame: {:?}", close);
                bail!("Close frame: {:?}", close)
            }
            tungstenite::Message::Binary(vec) => todo!("Binary"),
            tungstenite::Message::Frame(frame) => todo!("frame"),
            tungstenite::Message::Ping(vec) => {
                trace!("Ping");
                Ok(())
            }
            tungstenite::Message::Pong(vec) => {
                trace!("Pong");
                Ok(())
            }
        }
    }

    pub async fn process_welcome_message(
        &mut self,
        data: SessionData<'_>,
    ) -> Result<(), WebSocketError> {
        self.session_id = Some(data.id.to_string());
        if let Some(url) = data.reconnect_url {
            self.connect_url = WebSocketError::InvailURL(url.to_string())
                .map_err(url.parse())
                .unwrap();
        }
        // check if the token is expired, if it is, request a new token. This only works if using a oauth service for getting a token
        if self.token.is_elapsed() {
            error!("Token is elapsed");
            return Err(WebSocketError::TokenElapsed);
        }

        let transport = eventsub::Transport::websocket(data.id.clone());

        for subscription in self.subscriptions.clone() {
            use eventsub::EventType::*;
            match subscription {
                ChannelPointsCustomRewardRedemptionAdd => {
                    match self.client
                        .create_eventsub_subscription(
                        eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1::broadcaster_user_id(
                                self.user_id.clone(),
                            ),
                            transport.clone(),
                            &self.token,
                        )
                    .await
                    {
                        Ok(sub)  =>  {
                            info!("Subscribed to {}", subscription);
                            debug!{"Subscription: {:?}", sub};
                        },
                        Err(e) => {error!("Failed to subscribe to {}: {}", subscription, e)},
                    }
                }
                ChannelPointsCustomRewardRedemptionUpdate => {
                    match self.client
                    .create_eventsub_subscription(
                        eventsub::channel::ChannelPointsCustomRewardRedemptionUpdateV1::broadcaster_user_id(
                            self.user_id.clone(),
                        ),
                        transport.clone(),
                        &self.token,
                    )
                    .await
                    {
                        Ok(sub)  =>  {
                            info!("Subscribed to {}", subscription);
                            debug!{"Subscription: {:?}", sub};
                        },
                        Err(e) => {error!("Failed to subscribe to {}: {}", subscription, e)},
                    }
                }
                ChannelChatMessage => {
                    match self.client
                        .create_eventsub_subscription(
                            eventsub::channel::ChannelChatMessageV1::new(
                                self.user_id.clone(),
                                self.user_id.clone(),
                            ),
                            transport.clone(),
                            &self.token,
                        )
                        .await
                    {
                        Ok(sub)  =>  {
                            info!("Subscribed to {}", subscription);
                            debug!{"Subscription: {:?}", sub};
                        },
                        Err(e) => {error!("Failed to subscribe to {}: {}", subscription, e)},
                    }
                }
                _ => {
                    error!(target:"Twitch::websocket::subscription", "Tried to subscribe to unimplemented subscription: {}", subscription)
                }
            }
        }
        Ok(())
    }
}
