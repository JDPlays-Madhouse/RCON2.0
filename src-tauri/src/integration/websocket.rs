use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{self as Channel};
use Channel::{channel, error::RecvError, error::SendError, Receiver, Sender};

use crate::integration::Api;

pub type Result<T, E = WebsocketControllerError> = std::result::Result<T, E>;

pub static WEBSOCKET_STATE_TIMEOUT: f64 = 0.2;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum WebsocketState {
    #[default]
    Down,
    Alive,
    Errored,
}

impl WebsocketState {
    /// Returns `true` if the websocket state is [`Down`].
    ///
    /// [`Down`]: WebsocketState::Down
    #[must_use]
    pub fn is_down(&self) -> bool {
        matches!(self, Self::Down)
    }

    /// Returns `true` if the websocket state is [`Alive`].
    ///
    /// [`Alive`]: WebsocketState::Alive
    #[must_use]
    pub fn is_alive(&self) -> bool {
        matches!(self, Self::Alive)
    }

    /// Returns `true` if the websocket state is [`Errored`].
    ///
    /// [`Errored`]: WebsocketState::Errored
    #[must_use]
    pub fn is_errored(&self) -> bool {
        matches!(self, Self::Errored)
    }
}

#[derive(Debug)]
pub struct WebsocketController {
    api: Api,
    rx: Receiver<WebsocketCommand>,
    tx: Sender<WebsocketCommand>,
}

impl WebsocketController {
    pub async fn get_state(&self) -> Result<WebsocketState> {
        let mut temp_rx = self.new_rx();
        self.send(WebsocketCommand::StatusCheck)?;

        loop {
            match async_std::future::timeout(
                Duration::from_secs_f64(WEBSOCKET_STATE_TIMEOUT),
                temp_rx.recv(),
            )
            .await
            {
                Ok(Ok(ws_command)) => match ws_command {
                    WebsocketCommand::Restart | WebsocketCommand::StatusCheck => continue,
                    WebsocketCommand::State(websocket_state) => return Ok(websocket_state),
                },
                Ok(Err(e)) => {
                    tracing::error!("Error receieving: {}", &e);
                    return Err(WebsocketControllerError::RecvError(e));
                }
                Err(_) => return Ok(WebsocketState::Down),
            };
        }
    }
    pub async fn restart_websocket(&self) -> Result<WebsocketState> {
        self.send(WebsocketCommand::Restart)?;
        loop {
            match self.get_state().await? {
                WebsocketState::Down => continue,
                WebsocketState::Alive => return Ok(WebsocketState::Alive),
                WebsocketState::Errored => return Err(WebsocketControllerError::FailedToRestart),
            };
        }
    }

    pub fn api(&self) -> Api {
        self.api
    }

    pub fn rx(&self) -> &Receiver<WebsocketCommand> {
        &self.rx
    }
    pub fn new_rx(&self) -> Receiver<WebsocketCommand> {
        self.tx.subscribe()
    }

    pub fn tx(&self) -> &Sender<WebsocketCommand> {
        &self.tx
    }

    pub fn new(api: Api) -> Self {
        let (tx, rx) = channel(2);
        Self { api, rx, tx }
    }

    pub async fn recv_command(&mut self) -> Result<WebsocketCommand> {
        self.rx
            .recv()
            .await
            .map_err(WebsocketControllerError::RecvError)
    }

    pub fn send(&self, ws_command: WebsocketCommand) -> Result<usize> {
        let active_controllers = self
            .tx
            .send(ws_command)
            .map_err(WebsocketControllerError::SendError)?;
        if active_controllers <= 1 {
            return Err(WebsocketControllerError::NoRecieversAvailable);
        }
        Ok(active_controllers)
    }
}

impl Clone for WebsocketController {
    fn clone(&self) -> Self {
        Self {
            api: self.api,
            tx: self.tx.clone(),
            rx: self.tx.subscribe(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum WebsocketCommand {
    #[default]
    StatusCheck,
    State(WebsocketState),
    Restart,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum WebsocketControllerError {
    #[error("Error while sending command to websocket: {0}")]
    SendError(SendError<WebsocketCommand>),
    #[error("No recievers for this controller.")]
    NoRecieversAvailable,
    #[error("Recieving Error: {0}")]
    RecvError(RecvError),
    #[error("Failed to restart websocket")]
    FailedToRestart,
}
