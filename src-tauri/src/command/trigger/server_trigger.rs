use anyhow::{bail, Context};
use config::{Map, Value};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    integration::IntegrationEvent,
    servers::{self, GameServer},
};

use super::Trigger;

#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
/// Used as the link between [GameServer] and [Trigger].
pub struct GameServerTrigger {
    pub server: GameServer,
    pub trigger: Trigger,
    enabled: bool,
}

impl PartialOrd for GameServerTrigger {
    /// Ordering is calculated on the [GameServer] and [Trigger], not whether it is enabled.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GameServerTrigger {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.trigger
            .cmp(&other.trigger)
            .then(self.server.cmp(&other.server))
    }
}

impl PartialEq for GameServerTrigger {
    /// Equality is compared on the [GameServer] and [Trigger], not whether it is enabled.
    fn eq(&self, other: &Self) -> bool {
        self.server == other.server && self.trigger == other.trigger
    }
}

impl GameServerTrigger {
    pub fn new(server: GameServer, trigger: Trigger) -> Self {
        Self {
            server,
            trigger,
            enabled: false,
        }
    }

    /// Tests whether this pairing is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Tests whether this pairing is disabled.
    pub fn disabled(&self) -> bool {
        !self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Tests whether for a given [IntegrationEvent], it triggers, if so returns the [GameServer]
    /// as an option otherwise [None].
    ///
    /// Note: If [GameServerTrigger] is ['.disabled()'][GameServerTrigger::disabled], it will always return [None].
    pub fn event_triggered(&self, event: &IntegrationEvent) -> Option<GameServer> {
        if !self.enabled {
            None
        } else if self.trigger.is_match(event) {
            Some(self.server.clone())
        } else {
            None
        }
    }
}

impl From<GameServerTrigger> for Value {
    fn from(server_trigger: GameServerTrigger) -> Self {
        let mut map = Map::new();
        map.insert("enabled".to_string(), Value::from(server_trigger.enabled()));
        map.insert(
            "server_name".to_string(),
            Value::from(server_trigger.server.name),
        );
        let mut trigger_value = Value::from(server_trigger.trigger)
            .into_table()
            .context("Converting Trigger back to map")
            .unwrap();
        map.append(&mut trigger_value);

        Self::from(map)
    }
}

impl TryFrom<Value> for GameServerTrigger {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let gst_map = match value.clone().into_table() {
            Ok(t) => t,
            Err(e) => {
                error!("Invalid type for conversion");
                bail!(e)
            }
        };
        let server_name = match gst_map.get("server_name") {
            Some(s) => s.to_string(),
            None => {
                error!("No server name listed for a server_trigger");
                bail!("No server name listed for a server_trigger");
            }
        };
        let enabled = match gst_map.get("enabled") {
            Some(e) => match e.clone().into_bool() {
                Ok(e) => e,
                Err(e) => bail!(e),
            },
            None => {
                info!(
                    "Server {}, 'enabled' property not set. Defaulting to false",
                    &server_name
                );
                false
            }
        };
        let server = match servers::GameServer::try_get(&server_name) {
            Some(s) => s,
            None => {
                error!("No server with name '{}' found.", &server_name);
                bail!("No server with name '{}' found.", &server_name);
            }
        };

        let trigger = match Trigger::try_from(value) {
            Ok(t) => t,
            Err(e) => {
                error!(
                    "Trigger::try_from errored on server_name {} with: {}",
                    &server_name, e
                );
                bail!(e)
            }
        };
        Ok(Self {
            enabled,
            server,
            trigger,
        })
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use crate::{
        command::{Command, Prefix, RconCommand, RconCommandLua},
        servers::Game,
    };

    use super::*;

    #[fixture]
    fn command() -> Command {
        Command::new(
            "test",
            RconCommand {
                prefix: Prefix::C,
                lua_command: RconCommandLua::Inline("print('do not mind me just testing')".into()),
            },
        )
    }

    #[fixture]
    fn trigger() -> Trigger {
        Trigger::Chat {
            pattern: "test".to_string(),
            case_sensitive: true,
        }
    }

    #[fixture]
    fn integration_event_match() -> IntegrationEvent {
        IntegrationEvent::Chat {
            msg: "test".into(),
            author: "test".into(),
        }
    }

    #[fixture]
    fn integration_event_not_match() -> IntegrationEvent {
        IntegrationEvent::Chat {
            msg: "not".into(),
            author: "not".into(),
        }
    }

    #[fixture]
    fn server() -> GameServer {
        GameServer::new("test", "localhost", 2871, "secure_password", Game::Factorio)
    }

    #[fixture]
    fn server_trigger(server: GameServer, trigger: Trigger) -> GameServerTrigger {
        GameServerTrigger::new(server, trigger)
    }

    #[rstest]
    fn test_enabling(mut server_trigger: GameServerTrigger) {
        assert!(!server_trigger.enabled());
        assert!(server_trigger.disabled());
        server_trigger.enable();
        assert!(server_trigger.enabled());
        assert!(!server_trigger.disabled());
    }

    #[rstest]
    fn test_disabling(mut server_trigger: GameServerTrigger) {
        assert!(!server_trigger.enabled());
        assert!(server_trigger.disabled());
        server_trigger.enable();
        assert!(server_trigger.enabled());
        assert!(!server_trigger.disabled());
        server_trigger.disable();
        assert!(!server_trigger.enabled());
        assert!(server_trigger.disabled());
    }

    #[rstest]
    fn test_set_enabled(mut server_trigger: GameServerTrigger) {
        assert!(!server_trigger.enabled());
        assert!(server_trigger.disabled());
        server_trigger.set_enabled(true);
        assert!(server_trigger.enabled());
        assert!(!server_trigger.disabled());
        server_trigger.set_enabled(false);
        assert!(!server_trigger.enabled());
        assert!(server_trigger.disabled());
    }
}
