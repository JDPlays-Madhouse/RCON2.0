use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::{
    command::{Command, GameServerTrigger, Trigger},
    integration::IntegrationEvent,
    servers::GameServer,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub struct CommandLog {
    pub(super) time: SystemTime,
    pub(super) command: Command,
    pub(super) trigger: GameServerTrigger,
    pub(super) username: String,
    pub(super) message: Option<String>,
    pub(super) event: IntegrationEvent,
}

impl Ord for CommandLog {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.time.cmp(&other.time) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.command.cmp(&other.command)
    }
}

impl PartialOrd for CommandLog {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl CommandLog {
    pub fn new(
        command: Command,
        trigger: GameServerTrigger,
        event: IntegrationEvent,
        username: String,
        message: Option<String>,
    ) -> Self {
        Self {
            time: SystemTime::now(),
            command,
            trigger,
            username,
            message,
            event,
        }
    }

    pub fn time(&self) -> SystemTime {
        self.time
    }

    pub fn command(&self) -> &Command {
        &self.command
    }

    pub fn trigger(&self) -> &GameServerTrigger {
        &self.trigger
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn message(&self) -> Option<&String> {
        self.message.as_ref()
    }

    pub fn event(&self) -> &IntegrationEvent {
        &self.event
    }

    pub fn from_server(command: &Command, server: &GameServer) -> Self {
        let trigger = GameServerTrigger::new(server.clone(), Trigger::Server);
        let event = IntegrationEvent::Server;
        Self::new(command.clone(), trigger, event, "<server>".into(), None)
    }
}
