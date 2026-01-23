use std::time::Duration;

use crate::integration::event::{normalise_tier, CustomRewardVariant};
use crate::integration::websocket::{WebsocketCommand, WebsocketController, WebsocketState};
use crate::integration::{self, CustomRewardEvent, IntegrationEvent};
use anyhow::Result;
use futures::stream::FusedStream;
use futures::TryStreamExt;
use itertools::Itertools;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite;
use tracing::{debug, error, info, warn};
use tracing::{trace, Instrument};
use twitch_api::eventsub::EventType;
use twitch_api::eventsub::channel::goal::progress;
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

#[derive(Debug, PartialEq, Eq, thiserror::Error, miette::Diagnostic)]
pub enum WebsocketError {
    #[error("Token elapsed")]
    TokenElapsed,
    #[error("Error occured while reconnecting")]
    Reconnect,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Websocket Terminated")]
    Terminated,
    #[error("Failed to connect to {0}")]
    FailedToConnect(String),
    #[error("Failed to run to {0}")]
    FailedToRun(String),
    #[error("Invalid url: {0}")]
    InvalidURL(String),
    #[error("Error while processing message: {0}")]
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
    pub subscribed: Vec<eventsub::EventType>,
    pub event_tx: Sender<integration::IntegrationEvent>,
    pub controller: WebsocketController,
    state: WebsocketState,
    keep_alive_seconds: Duration,
}

impl std::fmt::Debug for WebsocketClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebsocketClient")
            .field("state", &self.state)
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
        controller: WebsocketController,
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
            subscribed: Vec::new(),
            keep_alive_seconds: Duration::from_secs(10),
            event_tx,
            state: WebsocketState::Down,
            controller,
        }
    }

    /// Connect to the websocket and return the stream
    pub async fn connect(
        &mut self,
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
            Ok((socket, _response)) => {
                self.state = WebsocketState::Alive;
                Ok(socket)
            }
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
            warn!("Token not refreshed!")
        }
        Some(self.token.clone())
    }

    /// Run the websocket subscriber
    #[tracing::instrument(skip_all, fields())]
    pub async fn run(&mut self) -> Result<(), WebsocketError> {
        let mut websocket_stream = match self.connect().await {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        info!("Connected to Twitch's Websocket");

        loop {
            if websocket_stream.is_terminated() {
                self.state = WebsocketState::Down;
                error!("Websocket Terminated");
                return Err(WebsocketError::Terminated);
            }

            let next = futures::StreamExt::next(&mut websocket_stream);
            let next_with_timeout = async_std::future::timeout(self.keep_alive_seconds, next);
            tokio::select!(
                ws_command = self.controller.recv_command() => {
                    let span = tracing::info_span!("websocket_command", raw_message = ?ws_command);
                    match ws_command {
                        Ok(WebsocketCommand::StatusCheck) => {
                            match self.controller.send(WebsocketCommand::State(self.state)){
                                Ok(_) => (),
                                Err(e) => {
                                    tracing::error!("Websocket command failed to send: {e}");
                                    continue
                                },
                            };
                        }
                        Ok(WebsocketCommand::Restart) => {
                            websocket_stream = match self.connect().instrument(span).await{
                                Ok(s) => s,
                                Err(e) => {
                                    tracing::error!("WebsocketCommand::Restart error while restarting: {e}");
                                    return Err(WebsocketError::Reconnect);
                                },
                            };
                        }
                        Ok(WebsocketCommand::State(_)) => continue,
                        Err(e) => {
                            tracing::error!("{e}");
                        }
                    }
                },
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
                                    websocket_stream = WebsocketError::FailedToRun("when reestablishing connection".into()).map_err(self
                                        .connect()
                                        .instrument(span)
                                        .await
                                        ).unwrap(); // BUG: Not handling error.
                                    continue
                                },
                                Ok(m) => m,
                                Err(e) => {
                                    tracing::error!("When getting message: {e}");
                                    self.state = WebsocketState::Down;
                                    return Err(WebsocketError::FailedToConnect(format!("When getting message: {e}")));
                                    },
                            };
                            match self.process_message(msg)
                                .instrument(span)
                                .await {
                                Ok(m) => m,
                                Err(e) => {
                                    if e == WebsocketError::Reconnect {
                                        websocket_stream = WebsocketError::FailedToRun("When reconnecting to websocket".into()).map_err(self
                                        .connect()
                                        .await
                                        ).unwrap(); // BUG: Not handling error.
                                    continue
                                    } else {
                                    self.state = WebsocketState::Down;
                                    return Err(e)}
                                }
                            }
                        }
                        Ok(None) => {
                            error!("Received none");
                        }
                        Err(timeout_error) => {
                            debug!("Twitch websocket has timed out, reestablishing now: {timeout_error}");
                            websocket_stream = WebsocketError::FailedToRun("when reestablishing connection after timeout".into()).map_err(self
                                        .connect()
                                        .await
                                        ).unwrap(); // BUG: Not handling error.
                            continue
                        }

                    }
                },
                else => {
                    self.state = WebsocketState::Down;
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
                    } => self.process_welcome_message(session).await,

                    EventsubWebsocketData::Reconnect {
                        payload: ReconnectPayload { session },
                        ..
                    } => {
                        self.session_id = Some(session.id.to_string());
                        if let Some(url) = session.reconnect_url {
                            self.connect_url = WebsocketError::InvalidURL(url.to_string())
                                .map_err(url.parse())
                                .unwrap();
                        }

                        if self.token.is_elapsed() {
                            match self.refresh_token().await {
                                Some(t) => self.token = t,
                                None => {
                                    return Err(WebsocketError::TokenElapsed);
                                }
                            };
                        }
                        Err(WebsocketError::Reconnect)
                    }

                    EventsubWebsocketData::Notification {
                        metadata: _,
                        payload,
                    } => {
                        match payload {
                            Event::ChannelChatMessageV1(eventsub::Payload { message, .. }) => {
                                self.channel_chat_message(message).await
                            }

                            Event::ChannelPointsCustomRewardRedemptionAddV1(
                                eventsub::Payload { message, .. },
                            ) => {
                                self.channel_points_custom_reward_redemption_add(message)
                                    .await
                            }

                            Event::ChannelPointsCustomRewardRedemptionUpdateV1(
                                eventsub::Payload { message, .. },
                            ) => {
                                self.channel_points_custom_reward_redemption_update(message)
                                    .await
                            }

                            Event::ChannelSubscribeV1(eventsub::Payload { message, .. }) => {
                                self.channel_subscribe(message).await
                            }

                            Event::ChannelSubscriptionMessageV1(eventsub::Payload {
                                message,
                                ..
                            }) => self.channel_subscribe_message(message).await,
                            Event::ChannelSubscriptionGiftV1(eventsub::Payload {
                                message, ..
                            }) => self.channel_subscription_gift(message).await,
                            Event::ChannelBitsUseV1(eventsub::Payload { message, .. }) => {
                                self.channel_bits_use(message).await
                            }
                            Event::ChannelHypeTrainBeginV1(eventsub::Payload{message, ..}) => {
                                self.channel_hype_train_begin(message).await
                            }
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

    async fn channel_subscribe_message(
        &mut self,
        message: eventsub::Message<eventsub::channel::ChannelSubscriptionMessageV1>,
    ) {
        match message.clone() {
            eventsub::Message::Notification(reward_payload) => {
                let message = format!(
                    "{:?} subscription: {} - {}",
                    reward_payload.tier, reward_payload.user_name, reward_payload.message.text
                );

                info!(
                    target = "rcon2::integration::twitch::websocket::ChannelSubscriptionMessageV1",
                    message
                );
                let _ = self
                    .event_tx
                    .send(IntegrationEvent::Subscription {
                        tier: normalise_tier(Some(reward_payload.tier.clone()), None),
                        user_name: reward_payload.user_name.to_string(),
                    })
                    .await;
            }
            _ => {
                error! {"Unhandled ChannelSubscriptionMessageV1 Payload: {:?}", message}
            }
        }
    }

    async fn channel_subscribe(
        &mut self,
        message: eventsub::Message<eventsub::channel::ChannelSubscribeV1>,
    ) {
        match message.clone() {
            eventsub::Message::Notification(reward_payload) => {
                let message = format!(
                    "{:?} subscription: {}",
                    reward_payload.tier, reward_payload.user_name,
                );

                info!(
                    target = "rcon2::integration::twitch::websocket::ChannelSubscribe",
                    message
                );
                let _ = self
                    .event_tx
                    .send(IntegrationEvent::Subscription {
                        tier: normalise_tier(Some(reward_payload.tier.clone()), None),
                        user_name: reward_payload.user_name.to_string(),
                    })
                    .await;
            }
            _ => {
                error! {"Unhandled ChannelSubscribeV1 Payload: {:?}", message}
            }
        }
    }

    async fn channel_subscription_gift(
        &mut self,
        message: eventsub::Message<eventsub::channel::ChannelSubscriptionGiftV1>,
    ) {
        match message.clone() {
            eventsub::Message::Notification(reward_payload) => {
                let user_name = reward_payload.user_name.clone().map(|n| n.to_string());
                let message = format!(
                    "{} {:?} subscription gift from {}",
                    reward_payload.total,
                    reward_payload.tier,
                    user_name.as_ref().unwrap_or(&String::from("annonymous")),
                );

                info!(
                    target = "rcon2::integration::twitch::websocket::ChannelSubscriptionGift",
                    message
                );
                let _ = self
                    .event_tx
                    .send(IntegrationEvent::GiftSub {
                        tier: normalise_tier(Some(reward_payload.tier.clone()), None),
                        user_name: user_name,
                        count: reward_payload.total as u64,
                    })
                    .await;
            }
            _ => {
                error! {"Unhandled ChannelSubscriptionGiftV1 Payload: {:?}", message}
            }
        }
    }

    async fn channel_points_custom_reward_redemption_update(
        &mut self,
        message: eventsub::Message<eventsub::channel::ChannelPointsCustomRewardRedemptionUpdateV1>,
    ) {
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
                        message: reward_payload.user_input.to_string(),
                    }))
                    .await;
            }
            _ => {
                error! {"Unhandled ChannelPointsCustomRewardRedemptionUpdateV1 Payload: {:?}", message}
            }
        }
    }

    async fn channel_points_custom_reward_redemption_add(
        &mut self,
        message: eventsub::Message<eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1>,
    ) {
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
                        message: reward_payload.user_input.to_string(),
                    }))
                    .await;
            }
            _ => {
                error! {"Unhandled ChannelPointsCustomRewardRedemptionAddV1 Payload: {:?}", message}
            }
        }
    }

    async fn channel_chat_message(
        &mut self,
        message: eventsub::Message<eventsub::channel::ChannelChatMessageV1>,
    ) {
        match message.clone() {
            eventsub::Message::Notification(chat_payload) => {
                let message =
                    chat_payload.chatter_user_name.to_string() + " - " + &chat_payload.message.text;
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
    /// Channel Bits Use docs: [dev.twitch.tv](https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelbitsuse)
    async fn channel_bits_use(
        &mut self,
        message: eventsub::Message<eventsub::channel::ChannelBitsUseV1>,
    ) {
        match message.clone() {
            eventsub::Message::Notification(reward_payload) => {
                let user_name = reward_payload.user_name.clone().take();
                let message = format!("{} bits given from {}", reward_payload.bits, user_name);
                info!(
                    target = "rcon2::integration::twitch::websocket::ChannelBitsUse",
                    message
                );
                let _ = self
                    .event_tx
                    .send(IntegrationEvent::Bits {
                        user_name,
                        bits: u64::try_from(reward_payload.bits)
                            .expect("usize is never larger than u64"),
                    })
                    .await;
            }
            _ => {
                error! {"Unhandled ChannelBitsUse Payload: {:?}", message}
            }
        }
    }
    
    /// Channel Hype Train begin docs: [dev.twitch.tv](https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelbitsuse)
    async fn channel_hype_train_begin(
        &mut self,
        message: eventsub::Message<eventsub::channel::ChannelHypeTrainBeginV1>,
    ) {
        match message.clone() {
            eventsub::Message::Notification(reward_payload) => {
                let hypetrain_id = reward_payload.id.clone().take();
                let level = reward_payload.level.clone();
                let progress = reward_payload.progress.clone();
                let message = format!("Hype train level {}", level);
                info!(
                    target = "rcon2::integration::twitch::websocket::ChannelHypeTrainBeginV1",
                    message
                );
                let _ = self
                    .event_tx
                    .send(IntegrationEvent::HypeTrain {
                    })
                    .await;
            }
            _ => {
                error! {"Unhandled ChannelBitsUse Payload: {:?}", message}
            }
        }
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
            match self.refresh_token().await {
                Some(t) => self.token = t,
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
            if self.subscribed.contains(&subscription) {
                info!("Already subscribed to: {}", &subscription);
                continue;
            }
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
                            self.subscribed.push(subscription);
                            info!("Subscribed to {}", subscription);
                            debug!{"Subscription: {:?}", sub};
                        },
                        Err(e) => {warn!("Failed to subscribe to {}: {}", subscription, e)},
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
                            self.subscribed.push(subscription);
                            info!("Subscribed to {}", subscription);
                            debug!{"Subscription: {:?}", sub};
                        },
                        Err(e) => {warn!("Failed to subscribe to {}: {}", subscription, e)},
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
                            self.subscribed.push(subscription);
                            info!("Subscribed to {}", subscription);
                            debug!{"Subscription: {:?}", sub};
                        },
                        Err(e) => {warn!("Failed to subscribe to {}: {}", subscription, e)},
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
                            self.subscribed.push(subscription);
                            info!("Subscribed to {}", subscription);
                            debug!{"Subscription: {:?}", sub};
                        },
                        Err(e) => {warn!("Failed to subscribe to {}: {}", subscription, e)},

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
                            self.subscribed.push(subscription);
                            info!("Subscribed to {}", subscription);
                            debug!{"Subscription: {:?}", sub};
                        },
                        Err(e) => {warn!("Failed to subscribe to {}: {}", subscription, e)},

                    }
                }
                ChannelBitsUse => {
                    match self.client
                        .create_eventsub_subscription(
                            eventsub::channel::ChannelBitsUseV1::broadcaster_user_id(self.user_id.clone()), 
                            transport.clone(), 
                            &self.token
                        ).await 
                    {
                        Ok(sub)  =>  {
                            self.subscribed.push(subscription);
                            info!("Subscribed to {}", subscription);
                            debug!{"Subscription: {:?}", sub};
                        },
                        Err(e) => {warn!("Failed to subscribe to {}: {}", subscription, e)},

                    }
                }
                _ => {
                    error!(target:"rcon2::integration::twitch::websocket::subscription", "Tried to subscribe to unimplemented subscription: {}", subscription)
                }
            }
        }
        self.current_subsciptions().await;
        Ok(())
    }

    /// TODO: Make return result and compare to current subscribed
    async fn current_subsciptions(&mut self) -> Vec<EventType> {
        let mut subscribed: Vec<EventType> = Vec::new();
        let _ = self
            .client
            .get_eventsub_subscriptions(None, None, None, &self.token)
            .map_ok(|r| {
                let mut subs = r.subscriptions.into_iter().map(|s| s.type_).collect_vec();
                subscribed.append(&mut subs);
            })
            .try_collect::<Vec<_>>()
            .await;

        subscribed = subscribed
            .into_iter()
            .unique_by(|s| s.to_str())
            .collect_vec();
        subscribed
    }

    pub fn state(&self) -> WebsocketState {
        self.state
    }
}
