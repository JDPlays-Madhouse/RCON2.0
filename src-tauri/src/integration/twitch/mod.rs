use std::{
    sync::{
        mpsc::{self, RecvError},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use crate::logging::{LogLevel, Logger};

use super::{
    IntegrationChannels, IntegrationCommand, IntegrationControl, IntegrationEvent,
    PlatformAuthenticate, PlatformConnection, TokenError, Transmittor,
};
use anyhow::{bail, Context, Result};
use futures::executor;
use reqwest::Client as ReqwestClient;
use twitch_api::{client::ClientDefault, types::UserName, TwitchClient};
use twitch_oauth2::UserToken;
use twitch_oauth2::{tokens::errors::ValidationError, Scope};

pub mod oauth;
pub mod websocket;

use twitch_types::UserId;
use websocket::WebsocketClient;
fn default_scopes_twitch() -> Vec<Scope> {
    vec![
        Scope::UserReadChat,
        Scope::ChatRead,
        Scope::ChannelReadRedemptions,
    ]
}
const LOGLOCATION: &str = "Twitch Integration";
pub struct TwitchApiConnection {
    pub username: Option<UserName>,
    client_id: String,
    client_secret: String,
    pub scope: Vec<Scope>,
    pub event_tx: Option<mpsc::Sender<IntegrationEvent>>,
    pub command_channels: IntegrationChannels<IntegrationCommand>,
    pub command_joinhandle: Option<JoinHandle<Result<(), mpsc::RecvError>>>,
    pub token: Option<UserToken>,
    pub redirect_url: String,
    pub logger: Arc<Mutex<Logger>>,
    pub client: TwitchClient<'static, ReqwestClient>,
    pub websocket: Option<WebsocketClient>,
    pub websocket_joinhandle: Option<tokio::task::JoinHandle<()>>,
    pub session_id: Option<String>,
}

impl TwitchApiConnection {
    pub fn new<T: Into<String>>(
        username: T,
        client_id: T,
        client_secret: T,
        redirect_url: T,
        logger: Arc<Mutex<Logger>>,
    ) -> Self {
        Self {
            username: Some(UserName::new(username.into())),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            redirect_url: redirect_url.into(),

            logger,
            scope: Default::default(),
            event_tx: None,
            command_channels: IntegrationChannels::default(),
            command_joinhandle: Default::default(),
            token: Default::default(),
            client: twitch_api::TwitchClient::new(),
            websocket: Default::default(),
            websocket_joinhandle: Default::default(),
            session_id: Default::default(),
        }
    }
}

impl TwitchApiConnection {
    pub async fn new_websocket(&mut self) {
        let token = self.check_token().await.unwrap();
        self.websocket = Some(WebsocketClient::new(
            self.session_id.clone(),
            token,
            self.user_id().await,
            Arc::clone(&self.logger),
        ));
        let websocket = self.websocket.clone().unwrap();
        let jh = tokio::spawn(async {
            dbg!("thread start");
            let _ = websocket.run().await;
            dbg!("thread end");
        });
        self.websocket_joinhandle = Some(jh);
        self.logger.lock().unwrap().log(
            LogLevel::Info,
            "Integration::Twitch::ApiConnection",
            "Websocket Established!",
        );
    }
}

impl TwitchApiConnection {
    pub async fn check_token(&mut self) -> anyhow::Result<UserToken, TokenError> {
        if self.token.is_none() {
            let _ = self.authenticate().await;
        }
        let token = self.token.clone().context("Token not being set.").unwrap();
        match token.access_token.validate_token(&self.client).await {
            Ok(_) => Ok(token),
            Err(e) => {
                use twitch_oauth2::tokens::errors::ValidationError;
                match e {
                    ValidationError::NotAuthorized => Err(TokenError::TokenNotAuthorized),
                    ValidationError::RequestParseError(request_parse_error) => {
                        dbg!(request_parse_error);
                        Err(TokenError::UnknownError)
                    }
                    ValidationError::Request(e) => {
                        dbg!(e);
                        Err(TokenError::UnknownError)
                    }
                    ValidationError::InvalidToken(_) => Err(TokenError::InvalidToken),
                    _ => todo!(),
                }
            }
        }
    }
}

impl TwitchApiConnection {
    pub async fn user_id(&mut self) -> UserId {
        if self.token.is_some() {
            self.token.clone().unwrap().user_id
        } else {
            self.authenticate().await;
            self.token.clone().unwrap().user_id
        }
    }
}
impl Transmittor for TwitchApiConnection {
    /// Adds or changes the integration event transmitor.
    ///
    /// Returns the old transmittor in an option.
    fn add_transmitor(
        &mut self,
        tx: mpsc::Sender<IntegrationEvent>,
    ) -> Option<mpsc::Sender<IntegrationEvent>> {
        let old_tx = self.event_tx.take();
        self.event_tx = Some(tx);
        old_tx
    }

    fn remove_transmitor(&mut self) -> Option<mpsc::Sender<IntegrationEvent>> {
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

    pub fn default_scopes(mut self) -> Self {
        for scope in default_scopes_twitch() {
            self = self.add_scope(scope);
        }
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

impl PlatformAuthenticate for TwitchApiConnection {
    async fn authenticate(&mut self) -> Result<()> {
        self.token = Some(
            oauth::oauth(
                self.scope.clone(),
                self.client_id.clone(),
                self.client_secret.clone(),
                self.redirect_url.clone(),
                Arc::clone(&self.logger),
            )
            .await
            .unwrap(),
        );
        self.logger
            .lock()
            .unwrap()
            .log(LogLevel::Info, LOGLOCATION, "Authenticated for Twitch.");

        Ok(())
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
        self.command_joinhandle = Some(std::thread::spawn(move || {
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
        }));
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use rstest::{fixture, rstest};
//
//     use crate::integration::IntegrationChannels;
//
//     use super::*;
//
//     #[fixture]
//     fn twitch_connection() -> TwitchApiConnection {
//         TwitchApiConnection::new("username", "client_id", "client_secret")
//     }
//
//     #[fixture]
//     fn channels() -> IntegrationChannels<IntegrationEvent> {
//         IntegrationChannels::default()
//     }
//
//     #[rstest]
//     fn connection_initilization(twitch_connection: TwitchApiConnection) {
//         let empty_vec: Vec<&str> = Vec::new();
//         assert_eq!(twitch_connection.username, "username");
//         assert_eq!(twitch_connection.client_id, "client_id");
//         assert_eq!(twitch_connection.client_secret, "client_secret");
//         assert_eq!(twitch_connection.scope, empty_vec);
//     }
//
//     #[rstest]
//     fn add_one_scope(mut twitch_connection: TwitchApiConnection) {
//         let scope: Vec<&str> = Vec::from(["test::scope"]);
//         twitch_connection = twitch_connection.add_scope("test::scope");
//         assert_eq!(twitch_connection.username, "username");
//         assert_eq!(twitch_connection.client_id, "client_id");
//         assert_eq!(twitch_connection.client_secret, "client_secret");
//         assert_eq!(twitch_connection.scope, scope);
//     }
//
//     #[rstest]
//     fn add_two_scope(mut twitch_connection: TwitchApiConnection) {
//         let scope: Vec<&str> = Vec::from(["test::scope1", "test::scope2"]);
//         twitch_connection = twitch_connection
//             .add_scope("test::scope1")
//             .add_scope("test::scope2");
//         assert_eq!(twitch_connection.username, "username");
//         assert_eq!(twitch_connection.client_id, "client_id");
//         assert_eq!(twitch_connection.client_secret, "client_secret");
//         assert_eq!(twitch_connection.scope, scope);
//     }
//
//     #[rstest]
//     fn add_default_scope(mut twitch_connection: TwitchApiConnection) {
//         twitch_connection = twitch_connection.default_scopes();
//         assert_eq!(twitch_connection.username, "username");
//         assert_eq!(twitch_connection.client_id, "client_id");
//         assert_eq!(twitch_connection.client_secret, "client_secret");
//         assert_eq!(twitch_connection.scope, default_scopes_twitch());
//     }
//
//     #[rstest]
//     fn has_scope(mut twitch_connection: TwitchApiConnection) {
//         let scope = "test::scope";
//         let empty_scopes: Vec<&str> = vec![];
//
//         assert!(!twitch_connection.has_scope(scope));
//         assert_eq!(twitch_connection.scope, empty_scopes);
//         twitch_connection = twitch_connection.add_scope(scope);
//         assert!(twitch_connection.has_scope(scope));
//         assert_eq!(twitch_connection.scope, vec![scope]);
//     }
//
//     #[rstest]
//     fn remove_scope(mut twitch_connection: TwitchApiConnection) {
//         let scope = "test::scope";
//         let empty_scopes: Vec<&str> = vec![];
//
//         assert!(!twitch_connection.has_scope(scope));
//         assert_eq!(twitch_connection.scope, empty_scopes);
//         twitch_connection = twitch_connection.add_scope(scope);
//         assert!(twitch_connection.has_scope(scope));
//         assert_eq!(twitch_connection.scope, vec![scope]);
//         twitch_connection = twitch_connection.remove_scope(scope);
//         assert!(!twitch_connection.has_scope(scope));
//         assert_eq!(twitch_connection.scope, empty_scopes);
//     }
//
//     #[rstest]
//     fn channel_transmitting(
//         mut channels: IntegrationChannels<IntegrationEvent>,
//         mut twitch_connection: TwitchApiConnection,
//     ) {
//         twitch_connection.add_transmitor(channels.tx.clone());
//         let msg = IntegrationEvent::Chat("Test 124");
//         assert!(twitch_connection
//             .transmit_event(IntegrationEvent::Connected)
//             .is_ok());
//         assert!(twitch_connection.transmit_event(msg).is_ok());
//
//         let rx = channels.take_rx().unwrap();
//         assert_eq!(rx.recv().unwrap(), IntegrationEvent::Connected);
//         assert_eq!(rx.recv().unwrap(), msg);
//     }
// }
