use std::sync::{
    mpsc::{channel, Receiver, RecvError, SendError, Sender},
    Arc,
};

use anyhow::{bail, Result};
pub mod twitch;
use config::Value;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info, instrument};
pub use twitch::TwitchApiConnection;

pub mod websocket;
pub use websocket::{
    WebsocketCommand, WebsocketController, WebsocketControllerError, WebsocketState,
    WEBSOCKET_STATE_TIMEOUT,
};

mod event;
pub use event::{CustomRewardEvent, CustomRewardVariant, IntegrationEvent};

pub mod status;
pub use status::{integration_status, IntegrationError, IntegrationStatus};
use twitch_oauth2::TwitchToken;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Api {
    Twitch,
    YouTube,
    Patreon,
    StreamLabs,
}

impl TryFrom<config::Value> for Api {
    type Error = anyhow::Error;

    fn try_from(value: config::Value) -> std::result::Result<Self, Self::Error> {
        use Api::*;
        match value
            .into_string()
            .expect("shouldn't fail...")
            .to_lowercase()
            .as_str()
        {
            "twitch" => Ok(Twitch),
            "youtube" => Ok(YouTube),
            api => bail!("Non valid api: {}", api),
        }
    }
}

pub type APIConnectionConfig = IndexMap<String, Value>;

pub enum Connection {
    Stream,
    Fetch(String),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct IntegrationChannels<T> {
    pub tx: Sender<T>,
    pub rx: Option<Receiver<T>>,
}
impl<T> IntegrationChannels<T> {
    pub fn take_rx(&mut self) -> Option<Receiver<T>> {
        self.rx.take()
    }
}

impl<T> Default for IntegrationChannels<T> {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self { tx, rx: Some(rx) }
    }
}

pub struct Integration<P>
where
    P: PlatformConnection,
{
    pub api: Api,
    pub connection: Connection,
    pub platform: P,
}

pub trait PlatformConnection {
    fn connect(&self) -> Result<()>;
}

#[allow(async_fn_in_trait)]
pub trait PlatformAuthenticate {
    async fn authenticate(&mut self) -> Result<()>;
}

pub trait Transmitter {
    /// Adds or changes the integration event transmitter.
    ///
    /// Returns the old transmitter in an option.
    fn add_transmitter(&mut self, tx: Sender<IntegrationEvent>)
        -> Option<Sender<IntegrationEvent>>;
    fn remove_transmitter(&mut self) -> Option<Sender<IntegrationEvent>>;
    fn transmit_event(&self, event: IntegrationEvent) -> Result<(), SendError<IntegrationEvent>>;
}

pub trait Scopes {
    fn has_scope(&self, scope: String) -> bool;
    fn add_scope(self, new_scope: String) -> Self;
    fn default_scopes(self) -> Self;
    fn remove_scope(self, scope: String) -> Self;
}

pub trait IntegrationControl {
    fn command_get_tx(&self) -> Sender<IntegrationCommand>;
    fn start_thread(&mut self) -> Result<(), RecvError>;
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum IntegrationCommand {
    #[default]
    Stop,
    Pause,
    Continue,
}

impl IntegrationCommand {
    /// Returns `true` if the integration command is [`Stop`].
    ///
    /// [`Stop`]: IntegrationCommand::Stop
    #[must_use]
    pub fn is_stop(&self) -> bool {
        matches!(self, Self::Stop)
    }

    /// Returns `true` if the integration command is [`Pause`].
    ///
    /// [`Pause`]: IntegrationCommand::Pause
    #[must_use]
    pub fn is_pause(&self) -> bool {
        matches!(self, Self::Pause)
    }

    /// Returns `true` if the integration command is [`Continue`].
    ///
    /// [`Continue`]: IntegrationCommand::Continue
    #[must_use]
    pub fn is_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenError {
    TokenElapsed,
    InvalidScopes,
    InvalidToken,
    UnknownError,
    NotAuthorized,
}

#[tauri::command]
#[instrument(level = "trace")]
pub async fn connect_to_integration(
    api: Api,
    twitch_integration: State<'_, Arc<futures::lock::Mutex<TwitchApiConnection>>>,
    config: State<'_, Arc<futures::lock::Mutex<config::Config>>>,
    force: bool,
) -> Result<IntegrationStatus, IntegrationError> {
    use Api::*;
    let config = config.lock().await.clone();

    match api {
        Twitch => {
            let mut twitch = twitch_integration.lock().await;
            match twitch.check_token().await {
                Ok(t) => {
                    twitch.run(config, force).await;
                    Ok(IntegrationStatus::Connected {
                        api: Api::Twitch,
                        expires_at: dbg!(Some(IntegrationStatus::seconds_to(t.expires_in()))),
                    })
                }
                Err(e) => {
                    error!("{:?}", e);
                    Err(IntegrationError::Token(e))
                }
            }
        }
        _ => Err(IntegrationError::NotImplemented(api)),
    }
}

#[tauri::command]
#[instrument(level = "trace", skip(config))]
pub async fn list_of_integrations(
    config: State<'_, Arc<futures::lock::Mutex<config::Config>>>,
) -> Result<Vec<Api>, String> {
    let config = config.lock().await.clone();
    match config.get_array("auth.platforms") {
        Ok(l) => {
            let platforms = l
                .iter()
                .filter_map(|p| Api::try_from(p.clone()).ok())
                .collect();
            info!("Selected Integrations: {:?}", &platforms);
            Ok(platforms)
        }
        Err(e) => {
            error!("{:?}", e);
            Err("Failed to get list from config.".to_string())
        }
    }
}
