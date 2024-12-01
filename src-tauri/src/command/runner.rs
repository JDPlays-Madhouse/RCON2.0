use crate::integration::IntegrationEvent;
use crate::servers::GameServer;
use anyhow::Result;
use std::collections::{hash_map::Entry, HashMap};
use tokio::{
    spawn,
    sync::mpsc::{channel, error::SendError, Receiver, Sender},
    task::JoinHandle,
};
use tracing::{error, info};

use super::{Command, Trigger};

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
    /// [Receiver] for the runner to listen on.
    rx: Option<Receiver<IntegrationEvent>>,
    /// [Sender] for the runner to pass to subscribers.
    tx: Sender<IntegrationEvent>,
    server: Option<GameServer>,
    triggers: HashMap<IntegrationEvent, Vec<Trigger>>,
    commands: Vec<Command>,
    joinhandle: Option<JoinHandle<Result<(), RunnerError>>>,
}

impl Runner {
    pub fn new(server: Option<GameServer>) -> Self {
        let (tx, rx) = channel::<IntegrationEvent>(100);
        Self {
            rx: Some(rx),
            tx,
            server,
            triggers: HashMap::new(),
            commands: Vec::new(),
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

    pub fn run(&'static mut self) -> Result<(), RunnerError> {
        if self.server().is_none() {
            return Err(RunnerError::NoGameServer);
        }
        let mut rx = match self.rx.take() {
            Some(rx) => rx,
            None => {
                let (tx, rx) = channel::<IntegrationEvent>(100);
                self.tx = tx;
                rx
            }
        };
        let triggers = self.triggers.clone();
        let _server = self.server.clone().unwrap();

        use IntegrationEvent::*;
        let jh: JoinHandle<std::result::Result<(), RunnerError>> = spawn(async move {
            loop {
                match rx.recv().await {
                    Some(event) => {
                        if triggers.contains_key(&event.event_type()) {
                            let group = triggers.get(&event.event_type()).unwrap();
                            group.iter().for_each(|t| if t.is_match(&event) {});
                        }
                    }
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
                    Some(Unknown) => todo!(),

                    Some(e) => {
                        error!("Unexpected integration event: {:?}", e)
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

    pub fn server(&self) -> &Option<GameServer> {
        &self.server
    }

    pub fn set_server(mut self, server: Option<GameServer>) -> Self {
        self.server = server;
        self
    }

    /// Adds the trigger to the correct group and will remove duplicate triggers.
    ///
    /// TODO: Add in running handling.
    pub fn add_trigger(&mut self, trigger: &Trigger) {
        if self.is_running() {
            unimplemented!("running handling")
        }
        let key = trigger.event_type();
        if let Entry::Vacant(e) = self.triggers.entry(key.clone()) {
            e.insert(vec![trigger.clone()]);
        } else {
            let triggers = self
                .triggers
                .get_mut(&key)
                .expect("Already handled the None case.");
            triggers.push(trigger.clone());
            triggers.sort();
            triggers.dedup();
        }
    }

    /// Removes the trigger if it exists and returns it as an [Option], otherwise returns [None].
    ///
    /// Note: If the [IntegrationEvent] has no linked triggers, it will be dropped from the [HashMap].
    ///
    /// TODO: Add in running handling.
    pub fn remove_trigger(&mut self, trigger: &Trigger) -> Option<Trigger> {
        if self.is_running() {
            unimplemented!("running handling")
        }
        if self.trigger_exists(trigger) {
            if let Some(group) = self.triggers.get_mut(&trigger.event_type()) {
                group.retain(|t| t != trigger);
                if group.is_empty() {
                    self.triggers.remove_entry(&trigger.event_type());
                }
                Some(trigger.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    // pub fn link_command(&mut self, trigger: Trigger, command: Command) {
    //     if self.is_running() {
    //         unimplemented!("running handling")
    //     }
    //     if !self.trigger_exists(&trigger) {
    //         self.add_trigger(&trigger);
    //     }
    //
    //     if let Entry::Vacant(e) = self.commands.entry(trigger.clone()) {
    //         e.insert(vec![command.clone()]);
    //     } else {
    //         let commands = self
    //             .commands
    //             .get_mut(&trigger)
    //             .expect("Already handled the None case.");
    //         commands.push(command.clone());
    //         commands.sort();
    //         commands.dedup();
    //     }
    // }

    /// Returns [true] if this runner has this [Trigger].
    pub fn trigger_exists(&self, trigger: &Trigger) -> bool {
        match self.triggers.get(&trigger.event_type()) {
            None => false,
            Some(g) => g.contains(trigger),
        }
    }

    /// Returns [true] if any triggers for a given [IntegrationEvent] exist.
    pub fn trigger_event_exists(&self, event: &IntegrationEvent) -> bool {
        self.triggers.contains_key(&event.event_type())
    }
}
