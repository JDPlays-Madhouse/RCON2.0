pub mod command_logs;
use anyhow::{bail, Result};
use config::{Map, Value, ValueKind};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use settings::ScriptSettings;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};
use tracing::{debug, error, info, instrument, trace, warn};

mod runner;
pub mod settings;
pub mod trigger;
pub use runner::Runner;
pub use trigger::{GameServerTrigger, Trigger};
mod command_type;
mod prefix;
pub use command_type::CommandType;
pub use prefix::Prefix;
mod command_lua;
pub use command_lua::{LuaFile, RconCommandLua};
mod rcon;
pub use rcon::RconCommand;
mod variable;
pub use variable::Variable;

use crate::{
    command::command_logs::{CommandLog, COMMAND_LOGS},
    integration::IntegrationEvent,
    servers::{GameServer, CONNECTIONS},
};

pub static COMMANDS: LazyLock<Arc<Mutex<HashMap<String, Command>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Command {
    pub name: String,
    pub rcon_lua: RconCommand,
    pub server_triggers: Vec<GameServerTrigger>,
}
#[allow(dead_code)]
impl Command {
    pub fn new<N: Into<String>>(name: N, rcon_lua: RconCommand) -> Self {
        Self {
            name: name.into(),
            rcon_lua,
            server_triggers: Vec::new(),
        }
    }

    pub fn set_name<N: Into<String>>(self, name: N) -> Self {
        Self {
            name: name.into(),
            ..self
        }
    }

    pub fn from_config<N, L, T>(name: N, rcon_lua: L, server_triggers: T) -> Self
    where
        N: Into<String>,
        L: Into<RconCommand>,
        T: Into<Vec<GameServerTrigger>>,
    {
        Self {
            name: name.into(),
            rcon_lua: rcon_lua.into(),
            server_triggers: server_triggers.into(),
        }
    }

    /// Adds command to [COMMANDS].
    pub fn add_to_commands(&self) -> String {
        trace!("add_to_commands");
        let mut commands = COMMANDS.lock().unwrap();
        trace!("COMMANDS Locked");
        commands.insert(self.id(), self.clone());
        trace!("COMMANDS Unlocked");

        self.id()
    }

    /// Fetches command from [COMMANDS] if possible, then checks config file, then returns
    /// [`Option<Command>`].
    pub fn get(id: &str) -> Option<Self> {
        {
            info!("Command::get");
            let commands = COMMANDS.lock().unwrap();
            info!("COMMANDS locked.");
            if let Some(command) = commands.get(id).cloned() {
                info!("COMMANDS Unlocked.");
                return Some(command);
            };
        }
        info!("COMMANDS Unlocked.");
        ScriptSettings::get_command(&ScriptSettings::new(), id)
    }

    pub fn id(&self) -> String {
        self.name.clone()
    }

    /// The full string for transmitting to the rcon server.
    pub fn tx_string(&mut self, message: Option<&str>, username: &str) -> String {
        self.rcon_lua.command(message, username)
    }

    pub async fn handle_event(&mut self, event: &IntegrationEvent) {
        for trigger in self.server_triggers.clone() {
            if let Some(server) = trigger.event_triggered(event) {
                COMMAND_LOGS.lock().await.add_log(CommandLog::new(
                    self.clone(),
                    trigger.clone(),
                    event.clone(),
                    event.username(),
                    event.message().map(|s| s.to_string()),
                ));
                info!("Server {} was triggered by {:?}", server.name, event);
                let mut connection_lock = CONNECTIONS.lock().await;
                if let Some(connection) = connection_lock.get_mut(&server) {
                    let _ = connection
                        .send_command(self.tx_string(event.message(), &event.username()))
                        .await;

                    info!("Sent \"{}\" to \"{}\" server.", self.name, &server.name);
                }
            }
        }
    }

    pub fn contains_server_trigger(&self, server: &GameServer, trigger: &Trigger) -> bool {
        let server_trigger = GameServerTrigger::new(server.clone(), trigger.clone());
        self.server_triggers.contains(&server_trigger)
    }

    pub fn add_server_trigger(
        &mut self,
        server: GameServer,
        trigger: Trigger,
        enabled: bool,
    ) -> Option<GameServerTrigger> {
        let mut server_trigger = GameServerTrigger::new(server, trigger);
        server_trigger.set_enabled(enabled);
        if self.server_triggers.contains(&server_trigger) {
            if let Some((pos, old_server_trigger)) = self
                .server_triggers
                .clone()
                .iter()
                .find_position(|st| (*st).clone() == server_trigger)
            {
                self.server_triggers[pos] = server_trigger;
                return Some(old_server_trigger.clone());
            };
        }
        self.server_triggers.push(server_trigger);
        None
    }

    pub fn set_server_triggers(
        &mut self,
        server_triggers: Vec<GameServerTrigger>,
    ) -> Vec<GameServerTrigger> {
        let old_server_triggers = self.server_triggers.clone();
        self.server_triggers = server_triggers;
        self.update_config();
        old_server_triggers
    }

    pub fn update_config(&self) {
        let mut settings = ScriptSettings::new();
        settings.set_command(self.clone());
    }

    pub fn remove_server_trigger(
        &self,
        _server: GameServer,
        _trigger: Trigger,
    ) -> Option<GameServerTrigger> {
        todo!("remove server trigger");
        // None
    }
}

impl From<Command> for Value {
    fn from(command: Command) -> Self {
        let mut map = Map::new();
        map.insert("name".to_string(), ValueKind::from(command.name));
        map.insert(
            "prefix".to_string(),
            ValueKind::from(command.rcon_lua.prefix.clone()),
        );
        map.insert(
            "command_type".to_string(),
            ValueKind::from(command.rcon_lua.lua_command.command_type()),
        );
        match command.rcon_lua.lua_command.command_type().as_str() {
            stringify!(File) => {
                map.insert(
                    "relative_path".to_owned(),
                    ValueKind::from(command.rcon_lua.lua_command),
                );
            }
            stringify!(Inline) => {
                map.insert(
                    "inline".to_owned(),
                    ValueKind::from(command.rcon_lua.lua_command),
                );
            }
            _ => {}
        }
        if !command.server_triggers.is_empty() {
            map.insert(
                "server_triggers".to_string(),
                ValueKind::Array(
                    command
                        .server_triggers
                        .iter()
                        .map(|v| Value::from(v.clone()))
                        .collect(),
                ),
            );
        }

        Self::new(None, ValueKind::from(map))
    }
}

impl TryFrom<Map<String, Value>> for Command {
    type Error = anyhow::Error;

    fn try_from(command_config_map: Map<String, Value>) -> std::result::Result<Self, Self::Error> {
        let mut errors: Vec<anyhow::Error> = vec![];
        let prefix = match Prefix::try_from(command_config_map.clone()) {
            Ok(p) => p,
            Err(e) => {
                error!("{}", &e);
                errors.push(e);
                Prefix::default()
            }
        };

        let lua_command = match RconCommandLua::try_from(command_config_map.clone()) {
            Ok(lua) => lua,
            Err(e) => {
                error!("{}", &e);
                errors.push(e);
                RconCommandLua::default()
            }
        };

        let variables = Variable::from_config_map(command_config_map.clone());

        let rconcommand = RconCommand {
            prefix,
            lua_command,
            variables,
        };

        let server_triggers: Vec<GameServerTrigger> =
            match command_config_map.get("server_triggers") {
                Some(t) => match t.clone().into_array() {
                    Ok(v) => v
                        .clone()
                        .iter()
                        .filter_map(|t| GameServerTrigger::try_from(t.clone()).ok())
                        .collect::<Vec<GameServerTrigger>>(),
                    Err(e) => bail!(e),
                },
                None => vec![],
            };

        if !errors.is_empty() {
            error!(
                "{} error/s occued in conversion from config file: {:?}",
                errors.len(),
                errors
            );
            Err(errors.remove(0))
        } else {
            Ok(Command::from_config("", rconcommand, server_triggers))
        }
    }
}

impl TryFrom<Value> for Command {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        let command_config_map = match value.try_deserialize::<Map<String, Value>>() {
            Ok(c) => c,
            Err(e) => bail!(e),
        };

        Self::try_from(command_config_map)
    }
}

#[tauri::command]
#[instrument(level = "trace")]
pub fn create_command(name: String, rcon_lua: RconCommand) -> Result<Command, String> {
    trace!("Create Command");
    let command = Command::new(name, rcon_lua);
    trace!("Created Command");
    command.add_to_commands(); // TODO: Add error handling for if command name exists.
    trace!("Added Command");
    Ok(command)
}

#[tauri::command]
#[instrument(level = "trace")]
pub fn get_command(name: String) -> Result<Option<Command>, String> {
    trace!("Get Command");
    Ok(Command::get(&name))
}

#[tauri::command]
pub async fn enable_server_trigger(
    id: String,
    enable: bool,
    server_trigger: GameServerTrigger,
) -> Result<Command, String> {
    debug!("enable_server_trigger");
    let commands = ScriptSettings::get_commands();
    if let Some(command) = commands.iter().find(|c| c.id() == id) {
        command.clone().server_triggers.iter_mut().for_each(|st| {
            if *st == server_trigger {
                st.set_enabled(enable);
            }
        });
        command.update_config();
        Ok(command.clone())
    } else {
        warn!("enable_server_trigger: Command not found.");
        Err("enable_server_trigger: Command not found.".to_string())
    }
}

#[tauri::command]
pub async fn update_server_trigger(
    command_name: String,
    server_triggers: Vec<GameServerTrigger>,
) -> Result<(), String> {
    debug!("update_server_triggers");
    // Get command
    let mut command = match Command::get(&command_name) {
        Some(c) => c,
        None => return Err("Command not found.".to_string()),
    };

    // Update command
    command.set_server_triggers(server_triggers);

    Ok(())
}

#[tauri::command]
#[instrument(level = "trace")]
pub async fn server_trigger_commands(
    server: GameServer,
) -> Result<Vec<(GameServerTrigger, Command)>, String> {
    let commands = ScriptSettings::get_commands();

    let commands_vec = commands
        .iter()
        .filter(|c| {
            c.server_triggers
                .clone()
                .iter()
                .any(|st| st.server == server)
        })
        .cloned()
        .collect_vec();

    let mut ret_vec = Vec::new();
    for command in commands_vec {
        for trigger in &command.server_triggers {
            ret_vec.push((trigger.clone(), command.clone()));
        }
    }
    Ok(ret_vec)
}

#[tauri::command]
#[instrument(level = "trace")]
pub async fn commands() -> Result<Vec<Command>, String> {
    Ok(ScriptSettings::get_commands())
}
