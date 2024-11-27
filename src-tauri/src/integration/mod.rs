use std::sync::mpsc::{channel, Receiver, RecvError, SendError, Sender};

use anyhow::Result;

pub mod twitch;
use config::Value;
use indexmap::IndexMap;
pub use twitch::TwitchApiConnection;

pub enum Api {
    Twitch,
    YouTube,
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
    P: PlatformConnection + PlatformAuthenticate + IntegrationControl,
{
    pub api: Api,
    pub connection: Connection,
    pub platform: P,
}

#[allow(dead_code)]
#[derive(Debug, Default, PartialEq, Clone)]
pub enum IntegrationEvent {
    #[default]
    Connected,
    Chat {
        msg: String,
        author: String,
    },
    ChannelPoint {
        id: String,
        redeemer: String,
    },
    Unknown,
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
