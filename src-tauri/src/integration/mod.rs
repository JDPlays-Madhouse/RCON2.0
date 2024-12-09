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
use tracing::{error, info};
pub use twitch::TwitchApiConnection;

#[derive(Debug, Serialize, Deserialize)]
pub enum Api {
    Twitch,
    YouTube,
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

#[allow(dead_code)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Hash)]
pub enum IntegrationEvent {
    #[default]
    Connected,
    Disconnected,
    Chat {
        msg: String,
        author: String,
    },
    ChannelPoint {
        id: String,
        redeemer: String,
    },
    Unknown,
    Stop,
    Pause,
    Continue,
    /// TODO: add implementation for updating triggers and commands for runner.
    Update,
}

impl IntegrationEvent {
    /// Returns `true` if the integration event is [`Chat`].
    ///
    /// [`Chat`]: IntegrationEvent::Chat
    #[must_use]
    pub fn is_chat(&self) -> bool {
        matches!(self, Self::Chat { .. })
    }

    /// Returns `true` if the integration event is [`ChannelPoint`].
    ///
    /// [`ChannelPoint`]: IntegrationEvent::ChannelPoint
    #[must_use]
    pub fn is_channel_point(&self) -> bool {
        matches!(self, Self::ChannelPoint { .. })
    }
    /// A default implementation of each enum which can be used for being a key.
    pub fn event_type(&self) -> Self {
        use IntegrationEvent::*;
        match self {
            Chat { .. } => Chat {
                msg: Default::default(),
                author: Default::default(),
            },
            Connected => Connected,
            ChannelPoint { .. } => ChannelPoint {
                id: Default::default(),
                redeemer: Default::default(),
            },
            Unknown => Unknown,
            Stop => Stop,
            Pause => Pause,
            Continue => Continue,
            Update => Update,
            Disconnected => Disconnected,
        }
    }
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

#[derive(Debug, Default, PartialEq, Copy, Clone)]
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

#[derive(Copy, Clone, Debug)]
pub enum TokenError {
    TokenElapsed,
    InvalidScopes,
    InvalidToken,
    UnknownError,
    TokenNotAuthorized,
}

#[tauri::command]
pub async fn connect_to_integration(
    api: Api,
    twitch_integration: State<'_, Arc<futures::lock::Mutex<TwitchApiConnection>>>,
    config: State<'_, Arc<std::sync::Mutex<config::Config>>>,
) -> Result<Api, String> {
    use Api::*;
    let config = config.lock().unwrap().clone();

    match api {
        Twitch => {
            let mut twitch = twitch_integration.lock().await;
            match twitch.check_token().await {
                Ok(_t) => {
                    twitch.new_websocket(config).await;
                    Ok(Api::Twitch)
                }
                Err(e) => {
                    error!("{:?}", e);
                    Err("Failed to authenticate.".to_string())
                }
            }
        }
        _ => unimplemented!("Integration not implemented"),
    }
}

#[tauri::command]
pub async fn list_of_integrations(
    config: State<'_, Arc<std::sync::Mutex<config::Config>>>,
) -> Result<Vec<Api>, String> {
    let config = config.lock().unwrap().clone();
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
