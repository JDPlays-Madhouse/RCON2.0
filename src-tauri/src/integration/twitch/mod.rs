use item_information::{jd_channel_points, CustomChannelPointRewardInfo};
use oauth::refresh_token;
use permissions::get_eventsub_consolidated_scopes;
use std::{
    str::FromStr,
    sync::{
        mpsc::{self, RecvError},
        Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use tauri::State;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument};

use crate::{
    command::Runner,
    integration::{
        websocket::{WebsocketController, WebsocketState},
        WEBSOCKET_STATE_TIMEOUT,
    },
};

use super::{
    status::{IntegrationError, IntegrationStatus},
    APIConnectionConfig, Api, IntegrationChannels, IntegrationCommand, IntegrationControl,
    IntegrationEvent, PlatformConnection, TokenError, Transmitter,
};
use anyhow::{bail, Result};
use config::Config;
use reqwest::Client as ReqwestClient;
use twitch_api::{
    eventsub::EventType,
    helix::points::{CustomReward, GetCustomRewardRequest},
    types::UserName,
    TwitchClient,
};
use twitch_oauth2::UserToken;
use twitch_oauth2::{Scope, TwitchToken};

pub mod item_information;
pub mod oauth;
pub mod permissions;
pub mod websocket;

use twitch_types::UserId;
use websocket::WebsocketClient;

pub struct TwitchApiConnection {
    pub username: Option<UserName>,
    client_id: String,
    client_secret: String,
    pub event_tx: Option<mpsc::Sender<IntegrationEvent>>,
    pub command_channels: IntegrationChannels<IntegrationCommand>,
    pub command_joinhandle: Option<JoinHandle<Result<(), mpsc::RecvError>>>,
    pub redirect_url: String,
    pub client: TwitchClient<'static, ReqwestClient>,
    pub websocket: Option<WebsocketClient>,
    pub websocket_joinhandle: Option<tauri::async_runtime::JoinHandle<()>>,
    pub session_id: Option<String>,
    pub scope: Vec<Scope>,
    pub runner: Runner,
    token: Mutex<Option<UserToken>>,
    connecting: bool,
    websocker_controller: WebsocketController,
}

impl std::fmt::Debug for TwitchApiConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TwitchApiConnection")
            .field("username", &self.username)
            .field("client_id", &self.client_id)
            // .field("client_secret", &self.client_secret)
            .field("scope", &self.scope)
            .field("event_tx", &self.event_tx)
            .field("command_channels", &self.command_channels)
            .field("command_joinhandle", &self.command_joinhandle)
            // .field("token", &self.token)
            .field("redirect_url", &self.redirect_url)
            .field("websocket", &self.websocket)
            .field("session_id", &self.session_id)
            .finish()
    }
}

impl TwitchApiConnection {
    pub fn new(config: APIConnectionConfig) -> Self {
        let username = config
            .get("username")
            .unwrap()
            .clone()
            .into_string()
            .expect("Unpacking twitch username");
        let client_id = config
            .get("client_id")
            .unwrap()
            .clone()
            .into_string()
            .expect("Unpacking twitch client_id");
        let client_secret = config
            .get("client_secret")
            .unwrap()
            .clone()
            .into_string()
            .expect("Unpacking twitch client_secret");
        let redirect_url = config
            .get("redirect_url")
            .unwrap()
            .clone()
            .into_string()
            .expect("Unpacking twitch redirect_url");
        let websocket_subs: Vec<EventType> = config
            .get("websocket_subscription")
            .unwrap()
            .clone()
            .into_array()
            .expect("Unpacking twitch websocket_subscription")
            .iter()
            .filter_map(
                |v| match EventType::from_str(v.clone().into_string().unwrap().as_str()) {
                    Ok(event) => Some(event),
                    Err(e) => {
                        error!("Error parsing event type: {:?}", e);
                        None
                    }
                },
            )
            .collect();
        let scope = get_eventsub_consolidated_scopes(websocket_subs);
        // get_all_required_scopes(scope);
        debug!("Scope: {:?}", scope);
        Self {
            username: Some(UserName::new(username)),
            client_id,
            client_secret,
            redirect_url,

            scope,
            event_tx: None,
            command_channels: IntegrationChannels::default(),
            command_joinhandle: Default::default(),
            client: twitch_api::TwitchClient::new(),
            websocket: Default::default(),
            websocket_joinhandle: Default::default(),
            session_id: Default::default(),
            runner: Runner::new(),
            token: Mutex::new(None),
            connecting: false,
            websocker_controller: WebsocketController::new(Api::Twitch),
        }
    }

    pub async fn token(&mut self) -> Option<UserToken> {
        let token_cont = self.token.lock().await;
        token_cont.clone()
    }
}

impl TwitchApiConnection {
    pub async fn run(&mut self, config: Config, force: bool) {
        self.new_websocket(config, force).await;
        match self.runner.run() {
            Ok(_) => {}
            Err(e) => {
                error!("{:?}", e)
            }
        }
    }

    pub async fn new_websocket(&mut self, config: Config, _force: bool) {
        self.connecting = true;
        debug!("new websocket");
        if let Some(joinhandle) = self.websocket_joinhandle.take() {
            joinhandle.abort();
            info!(
                "Old Twitch Websocket is finished: {}",
                joinhandle.inner().is_finished()
            );
        }
        let token = self.check_token().await.unwrap();
        let subscriptions_string: Vec<String> =
            config.get("auth.twitch.websocket_subscription").unwrap();
        let channel_points_enabled = subscriptions_string
            .iter()
            .any(|s| s.contains("channel_points_custom_reward"));
        let subscriptions: Vec<EventType> = subscriptions_string
            .iter()
            .map(|s| EventType::from_str(s).unwrap())
            .collect();
        info!("Websocket Subscriptions: {:?}", subscriptions);
        info!("Channel point rewards: {}", channel_points_enabled);
        if channel_points_enabled {
            let channel_point_custom_reward =
                CustomChannelPointRewardInfo::from_list(self.custom_rewards().await);
            CustomChannelPointRewardInfo::list_valid_trigger(channel_point_custom_reward, true);
        }

        self.websocket = Some(WebsocketClient::new(
            self.session_id.clone(),
            token,
            self.user_id().await.expect("Token is checked."),
            subscriptions,
            self.runner.tx(),
            self.websocker_controller.clone(),
        ));
        let websocket = self.websocket.clone().unwrap();
        self.websocket_joinhandle = Some(tauri::async_runtime::spawn(async move {
            use websocket::WebsocketError::*;
            let mut websocket_loop = websocket.clone();
            let mut restart_count: u16 = 0;
            let last_error = Instant::now();
            let acceptable_restart_rate = Duration::from_secs_f64(1.);
            let acceptable_error_limit = 10;
            while restart_count <= acceptable_error_limit {
                match websocket_loop.run().await {
                    Ok(_) => {}
                    Err(e @ InvalidToken) | Err(e @ TokenElapsed) => {
                        error!("Token error: {:?}", e);
                        break;
                    }
                    Err(e) => {
                        if last_error.elapsed() > acceptable_restart_rate {
                            restart_count += 1;
                        } else {
                            restart_count = restart_count.saturating_sub(1);
                        }
                        error!("Restart websocket count {restart_count}: {:?}", e);
                    }
                }
            }
        }));
        self.connecting = false;
    }
}

impl TwitchApiConnection {
    pub async fn check_token(&mut self) -> anyhow::Result<UserToken, TokenError> {
        let mut token: UserToken = match self.token().await {
            Some(t) => t,
            None => self.authenticate(true).await.expect("token"),
        };
        if token.is_elapsed() {
            if let Ok(t) = refresh_token(token.clone(), &self.client).await {
                token = t
            }
        }

        match token.validate_token(&self.client).await {
            Ok(vt) => {
                self.update_token(Some(token.clone())).await;
                let token_exp = vt
                    .expires_in
                    .expect("Token should have an expiration.")
                    .as_secs();
                let hours = token_exp / 3600;
                let mins = (token_exp % 3600) / 60;
                let secs = token_exp % 60;
                info!("Token expires in {}:{:02}:{:02}", hours, mins, secs);
                Ok(token)
            }
            Err(e) => {
                use twitch_oauth2::tokens::errors::ValidationError;
                match e {
                    ValidationError::NotAuthorized => match self.authenticate(false).await {
                        Ok(t) => {
                            self.update_token(Some(token.clone())).await;
                            Ok(t)
                        }
                        Err(e) => {
                            error!("ValidationError::NotAuthorized: {:?}", e);
                            Err(TokenError::UnknownError)
                        }
                    },
                    ValidationError::RequestParseError(request_parse_error) => {
                        error!("{:?}", request_parse_error);

                        match self.authenticate(false).await {
                            Ok(t) => {
                                self.update_token(Some(token.clone())).await;
                                Ok(t)
                            }
                            Err(e) => {
                                error!("ValidationError::RequestParseError: {:?}", e);
                                Err(TokenError::UnknownError)
                            }
                        }
                    }
                    ValidationError::Request(e) => {
                        error!("ValidationError::Request: {:?}", e);
                        match self.authenticate(false).await {
                            Ok(t) => {
                                self.update_token(Some(token.clone())).await;
                                Ok(t)
                            }
                            Err(e) => {
                                error!("ValidationError::RequestParseError: {:?}", e);
                                Err(TokenError::UnknownError)
                            }
                        }
                    }
                    ValidationError::InvalidToken(e) => {
                        error!("ValidationError::InvalidToken: {:?}", e);
                        match self.authenticate(false).await {
                            Ok(t) => {
                                self.update_token(Some(token.clone())).await;
                                Ok(t)
                            }
                            Err(e) => {
                                error!("ValidationError::InvalidToken: {:?}", e);
                                Err(TokenError::UnknownError)
                            }
                        }
                    }
                    _ => {
                        error!("Unknown Error: InvalidToken");
                        Err(TokenError::UnknownError)
                    }
                }
            }
        }
    }

    pub async fn update_token(&mut self, token: Option<UserToken>) {
        let mut token_cont = self.token.lock().await;
        *token_cont = token
    }
    pub async fn websocket_state(&mut self) -> Option<WebsocketState> {
        if self.websocket.is_none() {
            None
        } else {
            Some(
                self.websocker_controller
                    .get_state()
                    .await
                    .unwrap_or(WebsocketState::Errored),
            )
        }
    }

    pub async fn check_status(&mut self) -> anyhow::Result<IntegrationStatus, IntegrationError> {
        let api = Api::Twitch;
        if self.connecting {
            return Ok(IntegrationStatus::Connecting(api));
        };
        if let Some(ws_state) = self.websocket_state().await {
            if !ws_state.is_alive() {
                tracing::info!("{:?} state: {:?}", api, ws_state);
            };
            match ws_state {
                WebsocketState::Down => return Ok(IntegrationStatus::Disconnected(api)),
                WebsocketState::Alive => (),
                WebsocketState::Errored => {
                    return Ok(IntegrationStatus::Error {
                        api: Api::Twitch,
                        error: IntegrationError::WsError,
                    })
                }
            }
        } else {
            return Ok(IntegrationStatus::Disconnected(api));
        }
        let token: UserToken = match self.token().await {
            Some(t) => t,
            None => return Ok(IntegrationStatus::Disconnected(api)),
        };

        match token.validate_token(&self.client).await {
            Ok(vt) => Ok(IntegrationStatus::Connected {
                api,
                expires_at: Some(IntegrationStatus::seconds_to(
                    vt.expires_in.expect("Should not have non-expiring tokens"),
                )),
            }),
            Err(e) => {
                use twitch_oauth2::tokens::errors::ValidationError;
                match e {
                    ValidationError::NotAuthorized => {
                        error!("Token not authorized");
                        Err(IntegrationError::Token(TokenError::NotAuthorized))
                    }
                    ValidationError::RequestParseError(request_parse_error) => {
                        error!("{:?}", request_parse_error);
                        Err(IntegrationError::Token(TokenError::UnknownError))
                    }
                    ValidationError::Request(e) => {
                        error!("{:?}", e);
                        Err(IntegrationError::Token(TokenError::UnknownError))
                    }
                    ValidationError::InvalidToken(e) => {
                        error!("Invalid Token: {:?}", e);
                        Err(IntegrationError::Token(TokenError::InvalidToken))
                    }
                    _ => {
                        error!("Unknown Error");
                        Err(IntegrationError::Token(TokenError::UnknownError))
                    }
                }
            }
        }
    }
}

impl TwitchApiConnection {
    pub async fn user_id(&mut self) -> Option<UserId> {
        let token = self.token().await.clone();
        if token.is_some() {
            Some(token.clone().unwrap().user_id)
        } else if let Ok(t) = self.check_token().await {
            Some(t.user_id)
        } else {
            None
        }
    }

    pub async fn custom_rewards(&mut self) -> Vec<CustomReward> {
        let token = self
            .check_token()
            .await
            .expect("To call this, you must be authenticated");
        let request = GetCustomRewardRequest::broadcaster_id(
            self.user_id().await.expect("Everytoken has a user id."),
        );
        match self.client.helix.req_get(request, &(token.clone())).await {
            Ok(twitch_api::helix::Response { data, .. }) => data,
            Err(e) => {
                let message = e.to_string();
                error!(message);
                vec![]
            }
        }
    }
}

impl Transmitter for TwitchApiConnection {
    /// Adds or changes the integration event transmittor.
    ///
    /// Returns the old transmittor in an option.
    fn add_transmitter(
        &mut self,
        tx: mpsc::Sender<IntegrationEvent>,
    ) -> Option<mpsc::Sender<IntegrationEvent>> {
        let old_tx = self.event_tx.take();
        self.event_tx = Some(tx);
        old_tx
    }

    fn remove_transmitter(&mut self) -> Option<mpsc::Sender<IntegrationEvent>> {
        self.event_tx.take()
    }

    fn transmit_event(
        &self,
        event: IntegrationEvent,
    ) -> Result<(), std::sync::mpsc::SendError<IntegrationEvent>> {
        if self.event_tx.is_some() {
            self.event_tx.clone().unwrap().send(event)
        } else {
            Err(mpsc::SendError(event))
        }
    }
}

impl TwitchApiConnection {
    pub fn has_scope(&self, scope: String) -> bool {
        let scope = Scope::from(scope);
        self.scope.iter().any(|s| *s == scope)
    }

    pub fn add_scope<S: Into<Scope>>(mut self, new_scope: S) -> Self {
        let scope: Scope = new_scope.into();
        if !self.has_scope(scope.to_string()) {
            self.scope.push(scope)
        };
        self
    }

    pub fn remove_scope(mut self, scope: String) -> Self {
        let scope = Scope::from(scope);
        self.scope.retain(|s| *s != scope);
        self
    }
}

impl PlatformConnection for TwitchApiConnection {
    fn connect(&self) -> Result<()> {
        todo!()
    }
}

impl TwitchApiConnection {
    pub async fn authenticate(&mut self, use_cache: bool) -> Result<UserToken> {
        self.connecting = true;
        let token = match oauth::oauth(
            self.scope.clone(),
            self.client_id.clone(),
            self.client_secret.clone(),
            self.redirect_url.clone(),
            use_cache,
        )
        .await
        {
            Ok(t) => t,
            Err(e) => {
                error!("TwitchApiConnection::authenticate: {:?}", e);
                bail!(e);
            }
        };
        self.connecting = false;
        self.update_token(Some(token.clone())).await;
        info!("Twitch Authenticated");
        Ok(token)
    }
}

impl IntegrationControl for TwitchApiConnection {
    fn command_get_tx(&self) -> mpsc::Sender<IntegrationCommand> {
        self.command_channels.tx.clone()
    }

    fn start_thread(&mut self) -> Result<(), RecvError> {
        if self.command_channels.rx.is_none() {
            return Err(RecvError);
        }
        let command_rx = self.command_channels.rx.take().unwrap();
        std::thread::spawn(move || {
            loop {
                match command_rx.recv() {
                    Ok(cmd) => match cmd {
                        IntegrationCommand::Stop => {
                            dbg!(cmd);
                            break;
                        }
                        _ => {
                            dbg!(cmd);
                        }
                    },
                    Err(e) => {
                        dbg!(e);
                        return Err(e);
                    }
                }
            }
            Ok(())
        });
        Ok(())
    }
}

#[tauri::command]
#[instrument(skip(twitch_mutex))]
pub async fn get_channel_point_rewards(
    twitch_mutex: State<'_, Arc<futures::lock::Mutex<TwitchApiConnection>>>,
    testing: bool,
) -> Result<Vec<CustomChannelPointRewardInfo>, String> {
    if testing {
        Ok(CustomChannelPointRewardInfo::from_test_list(
            jd_channel_points(),
        ))
    } else {
        let mut twitch = twitch_mutex.lock().await;
        let rewards_list = twitch.custom_rewards().await;

        Ok(CustomChannelPointRewardInfo::from_list(rewards_list))
    }
}

#[tauri::command]
#[instrument(skip(twitch_mutex))]
pub async fn refresh_twitch_websocket(
    twitch_mutex: State<'_, Arc<futures::lock::Mutex<TwitchApiConnection>>>,
    config: State<'_, Arc<futures::lock::Mutex<config::Config>>>,
) -> Result<(), String> {
    let mut twitch = twitch_mutex.lock().await;
    let config = config.lock().await.clone();
    info!("Refreshing websocket");
    twitch.new_websocket(config, true).await;
    info!("Websocket refreshed");
    Ok(())
}
