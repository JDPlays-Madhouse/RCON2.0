use crate::integration::IntegrationEvent;
use anyhow::Result;
use tokio::{
    spawn,
    sync::mpsc::{channel, error::SendError, Receiver, Sender},
    task::JoinHandle,
};
use tracing::{error, info};

use super::{settings::ScriptSettings, Command};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerError {
    NoGameServer,
    FailedToStop,
    RunnerNotRunning,
    AlreadyCancelled,
    PanickedSubprocess,
    UnknownError,
}

/// Responsible for receiving [IntegrationEvent] and handling them.
#[derive(Debug)]
pub struct Runner {
    // id: Uuid,
    /// [Receiver] for the runner to listen on.
    rx: Option<Receiver<IntegrationEvent>>,
    /// [Sender] for the runner to pass to subscribers.
    tx: Sender<IntegrationEvent>,
    commands: Vec<Command>,
    joinhandle: Option<JoinHandle<Result<(), RunnerError>>>,
}

#[allow(clippy::new_without_default)]
impl Runner {
    pub fn new() -> Self {
        let (tx, rx) = channel::<IntegrationEvent>(100);

        Self {
            // id: Uuid::new_v4(),
            rx: Some(rx),
            tx,
            commands: ScriptSettings::get_commands(),
            joinhandle: None,
        }
    }

    /// Used for getting the transmittor, returns a clone.
    pub fn tx(&self) -> Sender<IntegrationEvent> {
        self.tx.clone()
    }

    pub async fn transmit(
        &mut self,
        event: IntegrationEvent,
    ) -> Result<(), SendError<IntegrationEvent>> {
        self.tx.send(event).await
    }

    /// Stops the runner, then generates new channels.
    pub async fn new_channel(&mut self) {
        if self.joinhandle.is_some() {
            let _ = self.abort().await;
        }
        let (tx, rx) = channel::<IntegrationEvent>(100);
        self.tx = tx;
        self.rx = Some(rx);
    }

    /// TODO: re-write this section.
    pub fn run(&mut self) -> Result<(), RunnerError> {
        let mut rx = match self.rx.take() {
            Some(rx) => rx,
            None => {
                let (tx, rx) = channel::<IntegrationEvent>(100);
                self.tx = tx;
                rx
            }
        };
        let mut commands = self.commands.clone();
        use IntegrationEvent::*;
        let jh: JoinHandle<std::result::Result<(), RunnerError>> = spawn(async move {
            loop {
                match rx.recv().await {
                    Some(Connected) => {
                        info!("runner connected");
                    }
                    Some(Disconnected) => {
                        info!("runner Disconnected");
                    }
                    Some(Stop) => return Ok(()),
                    Some(Pause) => loop {
                        match rx.recv().await {
                            Some(Continue) => break,
                            Some(Stop) => return Ok(()),
                            None => return Ok(()),
                            _ => continue,
                        }
                    },
                    Some(Unknown) => {
                        error!("An unknown event occurred")
                    }
                    Some(event) => {
                        info!("{:?}", &event);
                        for command in commands.iter_mut() {
                            info!("{:?}", &command);
                            command.handle_event(&event).await;
                        }
                    }

                    None => return Ok(()),
                }
            }
        });
        self.joinhandle = Some(jh);
        Ok(())
    }

    /// Returns [true] if the runner is actively running.
    pub fn is_running(&self) -> bool {
        if let Some(jh) = &self.joinhandle {
            !jh.is_finished()
        } else {
            false
        }
    }

    /// Run this function after any change to the running [Runner].
    pub async fn update(&mut self) -> Result<(), RunnerError> {
        if !self.is_running() {
            return Ok(());
        }
        todo!("Runner::update")
    }

    pub async fn abort(&mut self) -> Result<(), RunnerError> {
        match self.joinhandle.take() {
            Some(jh) => {
                let _ = self.transmit(IntegrationEvent::Stop).await;
                match jh.await {
                    Ok(Ok(_)) => Ok(()),
                    Ok(Err(e)) => Err(e),
                    Err(join_error) => {
                        if join_error.is_cancelled() {
                            error!("Runner already cancelled.");
                            Err(RunnerError::AlreadyCancelled)
                        } else if join_error.is_panic() {
                            use std::panic;
                            error!("Subprocess Panicked");
                            panic::resume_unwind(join_error.into_panic());
                            // Err(RunnerError::PanicedSubprocess)
                        } else {
                            Err(RunnerError::UnknownError)
                        }
                    }
                }
            }
            None => Err(RunnerError::RunnerNotRunning),
        }
    }
}
