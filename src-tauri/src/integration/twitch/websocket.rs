use std::time::Duration;

use crate::integration::event::{normalise_tier, CustomRewardVariant};
use crate::integration::{self, CustomRewardEvent, IntegrationEvent};
use anyhow::Result;
use futures::stream::FusedStream;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite;
use tracing::{debug, error, info, warn};
use tracing::{trace, Instrument};
use twitch_api::{
    client::ClientDefault,
    eventsub::{
        self,
        event::websocket::{EventsubWebsocketData, ReconnectPayload, SessionData, WelcomePayload},
        Event,
    },
    types::{self},
    HelixClient,
};
use twitch_oauth2::{TwitchToken, UserToken};

#[derive(Debug)]
pub enum WebsocketError {
    TokenElapsed,
    Reconnect,
    InvalidToken,
    Terminated,
    FailedToConnect(String),
    FailedToRun(String),
    InvalidURL(String),
    ProcessMessage(String),
}

impl WebsocketError {
    pub fn map_err<T, E>(self, result: Result<T, E>) -> Result<T, WebsocketError> {
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
    pub event_tx: Sender<integration::IntegrationEvent>,
    keep_alive_seconds: Duration,
}

impl std::fmt::Debug for WebsocketClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebsocketClient")
            .field("session_id", &self.session_id)
            .field("token", &self.token)
            .field("user_id", &self.user_id)
            .field("connect_url", &self.connect_url)
            .field("keep_alive_seconds", &self.keep_alive_seconds)
            .finish()
    }
}

impl WebsocketClient {
    pub fn new(
        session_id: Option<String>,
        token: UserToken,
        user_id: types::UserId,
        subscriptions: Vec<eventsub::EventType>,
        event_tx: Sender<integration::IntegrationEvent>,
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
            keep_alive_seconds: Duration::from_secs(10),
            event_tx,
        }
    }

    /// Connect to the websocket and return the stream
    pub async fn connect(
        &self,
    ) -> Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        WebsocketError,
    > {
        tracing::debug!("Connecting to Twitch Websocket");
        let config = tungstenite::protocol::WebSocketConfig::default()
            .max_message_size(Some(64 << 20))
            .max_frame_size(Some(16 << 20))
            .accept_unmasked_frames(false);

        match tokio_tungstenite::connect_async_with_config(&self.connect_url, Some(config), false)
            .await
        {
            Ok((socket, _response)) => Ok(socket),
            Err(e) => {
                error!("{}", e);
                Err(WebsocketError::FailedToConnect("Can't Connect".into()))
            }
        }
    }

    /// Refreshes the token if it exists. Returns the new token if successful.
    pub async fn refresh_token(&mut self) -> Option<UserToken> {
        info!("Refreshing OAuth Token.");
        
        let old_token = self.token.clone();

        let reqwest_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        match self.token.refresh_token(&reqwest_client).await {
            Ok(_) => {}
            Err(e) => {
                error!("Error refreshing token: {}", e);
            }
        };

        match self.token.validate_token(&reqwest_client).await {
            Ok(vt) => {
                let token_exp = vt
                    .expires_in
                    .expect("Token should have an expiration.")
                    .as_secs();
                let hours = token_exp / 3600;
                let mins = (token_exp % 3600) / 60;
                let secs = token_exp % 60;
                info!("Valid token expires in {}:{:02}:{:02}", hours, mins, secs);
            }
            Err(e) => {
                error!("Validation Error: {:?}", e);
                return None;
            }
        }
        if old_token.access_token == self.token.access_token {
            info!("{} == {}: {}", old_token.access_token.secret(), self.token.access_token.secret(), old_token.access_token == self.token.access_token);
            warn!("Token not refreshed!")
        }
        Some(self.token.clone())
    }

    /// Run the websocket subscriber
    #[tracing::instrument(skip_all, fields())]
    pub async fn run(&mut self) -> Result<(), WebsocketError> {
        // Establish the stream
        let mut s = match self.connect().await {
            Ok(s) => s,
            Err(e) => return Err(e),
        };
        info!("Connected to Twitch's Websocket");
        // Loop over the stream, processing messages as they come in.

        loop {
            if s.is_terminated() {
                error!("Websocket Terminated");
                return Err(WebsocketError::Terminated);
            }


            let next = futures::StreamExt::next(&mut s);
            let next_with_timeout = async_std::future::timeout(self.keep_alive_seconds, next);
            tokio::select!(
                msg_result = next_with_timeout => {
                    match msg_result {
                        Ok(Some(msg)) => {
                            let span = tracing::info_span!("message received", raw_message = ?msg);
                            let msg = match msg {
                                Err(tungstenite::Error::Protocol(
                                    tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
                                )) => {
                                    tracing::warn!(
                                        "connection was sent with an unexpected frame or was reset, reestablishing it"
                                    );
                                    s = WebsocketError::FailedToRun("when reestablishing connection".into()).map_err(self
                                        .connect()
                                        .instrument(span)
                                        .await
                                        ).unwrap(); // BUG: Not handling error.
                                    continue
                                }
                                _ => msg.map_err(|_e| WebsocketError::FailedToConnect("When getting message".into()))?,
                            };
                            match self.process_message(msg)
                                .instrument(span)
                                .await {
                                Ok(m) => m,
                                Err(e) => return Err(e),
                            }
                            }
                        Ok(None) => {
                            error!("Received none");
                        }
                        Err(_timeout_error) => {
                            debug!("Twitch websocket has timed out, reestablishing now...");
                            s = WebsocketError::FailedToRun("when reestablishing connection after timeout".into()).map_err(self
                                        .connect()
                                        .await
                                        ).unwrap(); // BUG: Not handling error.
                            continue
                        }

                    }
                },
                else => {
                    error!("Websocket Terminated");
                    return Err(WebsocketError::Terminated);
                }
            );
        }
    }

    /// Process a message from the websocket
    pub async fn process_message(
        &mut self,
        msg: tungstenite::Message,
    ) -> Result<(), WebsocketError> {
        match msg {
            tungstenite::Message::Text(s) => {
                tracing::trace!("{s}");
                // Parse the message into a [twitch_api::eventsub::EventsubWebsocketData]
                match Event::parse_websocket(s.as_str()).unwrap() {
                    EventsubWebsocketData::Welcome {
                        payload: WelcomePayload { session },
                        ..
                    }
                    | EventsubWebsocketData::Reconnect {
                        payload: ReconnectPayload { session },
                        ..
                    }
                     => self.process_welcome_message(session).await,
                    EventsubWebsocketData::Notification {
                        metadata: _,
                        payload,
                    } => {
                        match payload {
                            Event::ChannelChatMessageV1(eventsub::Payload { message, .. }
                            ) => self.channel_chat_message(message).await,

                            Event::ChannelPointsCustomRewardRedemptionAddV1(
                                eventsub::Payload { message, .. },
                            ) => self.channel_points_custom_reward_redemption_add(message).await,

                            Event::ChannelPointsCustomRewardRedemptionUpdateV1(
                                eventsub::Payload { message, .. },
                            ) => self.channel_points_custom_reward_redemption_update(message).await,

                            Event::ChannelSubscribeV1(eventsub::Payload { message, .. }
                            ) => self.channel_subscribe(message).await,
                            
                            Event::ChannelSubscriptionMessageV1(eventsub::Payload {
                                message,
                                ..
                            }) => self.channel_subscribe_message(message).await,

                            m => {
                                let message =
                                    format!("Received an unimplemented websocket event: {:?}", m);
                                error!(
                                    target = "rcon2::integration::twitch::websocket::Event",
                                    message
                                );
                            }
                        }
                        Ok(())
                    }
                    EventsubWebsocketData::Revocation {
                        metadata,
                        payload: _,
                    } => {
                        error!("Revocation event: {metadata:?}");
                        Err(WebsocketError::Terminated)
                    }
                    EventsubWebsocketData::Keepalive {
                        metadata: _,
                        payload: _,
                    } => Ok(()),
                    data => {
                        error!("Unexpected EventsubWebsocketData {:?}", data);
                        Err(WebsocketError::FailedToRun(format!(
                            "Unexpected EventsubWebsocketData {:?}",
                            data
                        )))
                    }
                }
            }
            tungstenite::Message::Close(close) => {
                error!("Close frame: {:?}", close);
                Err(WebsocketError::Terminated)
            }
            tungstenite::Message::Binary(_vec) => todo!("Binary"),
            tungstenite::Message::Frame(_frame) => todo!("frame"),
            tungstenite::Message::Ping(_vec) => {
                trace!("Ping");
                Ok(())
            }
            tungstenite::Message::Pong(_vec) => {
                trace!("Pong");
                Ok(())
            }
        }
    }

    async fn channel_subscribe_message(&mut self, message: eventsub::Message<eventsub::channel::ChannelSubscriptionMessageV1>) {
        match message.clone() {
            eventsub::Message::Notification(reward_payload) => {
                let message = format!(
                    "{:?} subscription: {} - {}",
                    reward_payload.tier,
                    reward_payload.user_name,
                    reward_payload.message.text
                );

                info!(target = "rcon2::integration::twitch::websocket::ChannelSubscriptionMessageV1", message);
                let _ = self
                    .event_tx
                    .send(IntegrationEvent::Subscription {
                        tier: normalise_tier(
                            Some(reward_payload.tier.clone()),
                            None,
                        ),
                        user_name: reward_payload.user_name.to_string(),
                    })
                    .await;
            }
            _ => {
                error! {"Unhandled ChannelSubscriptionMessageV1 Payload: {:?}", message}
            }
        }
    }

    async fn channel_subscribe(&mut self, message: eventsub::Message<eventsub::channel::ChannelSubscribeV1>) {
        match message.clone() {
            eventsub::Message::Notification(reward_payload) => {
                let message = format!(
                    "{:?} subscription: {}",
                    reward_payload.tier, reward_payload.user_name,
                );

                info!(target = "rcon2::integration::twitch::websocket::ChannelSubscribe", message);
                let _ = self
                    .event_tx
                    .send(IntegrationEvent::Subscription {
                        tier: normalise_tier(
                            Some(reward_payload.tier.clone()),
                            None,
                        ),
                        user_name: reward_payload.user_name.to_string(),
                    })
                    .await;
            }
            _ => {
                error! {"Unhandled ChannelSubscribeV1 Payload: {:?}", message}
            }
        }
    }

    async fn channel_points_custom_reward_redemption_update(&mut self, message: eventsub::Message<eventsub::channel::ChannelPointsCustomRewardRedemptionUpdateV1>) {
        match message.clone() {
            eventsub::Message::Notification(reward_payload) => {
                let message = format!(
                    "Update: {}({}) redeemed by {}: {}",
                    reward_payload.reward.title,
                    reward_payload.reward.id,
                    reward_payload.user_name,
                    reward_payload.user_input
                );

                info!(target = "rcon2::integration::twitch::websocket::ChannelPointsCustomRewardRedemptionUpdate", message);
                let _ = self
                    .event_tx
                    .send(IntegrationEvent::ChannelPoint(CustomRewardEvent {
                        event_id: reward_payload.id.to_string(),
                        id: reward_payload.reward.id.to_string(),
                        title: reward_payload.reward.title,
                        user_name: reward_payload.user_name.to_string(),
                        variant: CustomRewardVariant::Update,
                    }))
                    .await;
            }
            _ => {
                error! {"Unhandled ChannelPointsCustomRewardRedemptionUpdateV1 Payload: {:?}", message}
            }
        }
    }

    async fn channel_points_custom_reward_redemption_add(&mut self, message: eventsub::Message<eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1>) {
        match message.clone() {
        eventsub::Message::Notification(reward_payload) => {
            let message = format!(
                "New: {}({}) redeemed by {}: {}",
                reward_payload.reward.title,
                reward_payload.reward.id,
                reward_payload.user_name,
                reward_payload.user_input
            );
            info!(target = "rcon2::integration::twitch::websocket::ChannelPointsCustomRewardRedemptionAdd", message);
            let _ = self
                .event_tx
                .send(IntegrationEvent::ChannelPoint(CustomRewardEvent {
                    event_id: reward_payload.id.to_string(),
                    id: reward_payload.reward.id.to_string(),
                    title: reward_payload.reward.title,
                    user_name: reward_payload.user_name.to_string(),
                    variant: CustomRewardVariant::New,
                }))
                .await;
        }
        _ => {
            error! {"Unhandled ChannelPointsCustomRewardRedemptionAddV1 Payload: {:?}", message}
        }
                                    }
    }

    async fn channel_chat_message(&mut self, message: eventsub::Message<eventsub::channel::ChannelChatMessageV1>) {
        match message.clone() {
            eventsub::Message::Notification(chat_payload) => {
                let message = chat_payload.chatter_user_name.to_string()
                    + " - "
                    + &chat_payload.message.text;
                info!(
                    target = "rcon2::integration::twitch::websocket::ChannelChatMessage",
                    message
                );
                let _ = self
                    .event_tx
                    .send(IntegrationEvent::Chat {
                        msg: chat_payload.message.text.to_string(),
                        author: chat_payload.chatter_user_name.to_string(),
                    })
                    .await;
            }
            _ => {
                error! {"Unhandled Message Payload: {:?}", message}
            }
        };
    }

    pub async fn process_welcome_message(
        &mut self,
        data: SessionData<'_>,
    ) -> Result<(), WebsocketError> {
        self.session_id = Some(data.id.to_string());
        if let Some(url) = data.reconnect_url {
            self.connect_url = WebsocketError::InvalidURL(url.to_string())
                .map_err(url.parse())
                .unwrap();
        }

        if self.token.is_elapsed() {
            match self.refresh_token().await{
                Some(t) => {
                    self.token = t
                }
                None => {
                    return Err(WebsocketError::TokenElapsed);
                }
            };
        }

        if let Some(keep_alive) = data.keepalive_timeout_seconds {
            self.keep_alive_seconds =
                Duration::from_secs(keep_alive as u64) + Duration::from_millis(200);
        };
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
                ChannelSubscribe => {
                    match self.client
                        .create_eventsub_subscription(
                            eventsub::channel::ChannelSubscribeV1::broadcaster_user_id(self.user_id.clone()), 
                            transport.clone(), 
                            &self.token
                        ).await 
                    {
                        Ok(sub)  =>  {
                            info!("Subscribed to {}", subscription);
                            debug!{"Subscription: {:?}", sub};
                        },
                        Err(e) => {error!("Failed to subscribe to {}: {}", subscription, e)},

                    }
                }
                ChannelSubscriptionMessage => {
                    match self.client
                        .create_eventsub_subscription(
                            eventsub::channel::ChannelSubscriptionMessageV1::broadcaster_user_id(self.user_id.clone()), 
                            transport.clone(), 
                            &self.token
                        ).await 
                    {
                        Ok(sub)  =>  {
                            info!("Subscribed to {}", subscription);
                            debug!{"Subscription: {:?}", sub};
                        },
                        Err(e) => {error!("Failed to subscribe to {}: {}", subscription, e)},

                    }
                }
                _ => {
                    error!(target:"rcon2::integration::twitch::websocket::subscription", "Tried to subscribe to unimplemented subscription: {}", subscription)
                }
                
            }
        }
        Ok(())
    }
}
