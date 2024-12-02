use anyhow::{bail, Result};
use config::{Map, Value, ValueKind};
use serde::{Deserialize, Serialize};
use settings::ScriptSettings;
use std::{
    collections::HashMap,
    fmt::Display,
    fs,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
};
use tracing::{debug, error, info, trace};

mod runner;
pub mod settings;
mod trigger;
pub use runner::Runner;
pub use trigger::Trigger;

use crate::{
    servers::{self, GameServer},
    settings::Settings,
};

pub static COMMANDS: LazyLock<Arc<Mutex<HashMap<String, Command>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(tag = "type", content = "data")]
pub enum CommandType {
    ChannelPoints(String),
    Chat,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(tag = "prefix", content = "data")]
pub enum RconCommandPrefix {
    /// Everything before the actual command including slashes and spaces.
    Custom(String),
    /// Silent Command: /silent-command \<command\> <br/>
    /// Executes a Lua command (if allowed) without printing it to the console.
    SC,
    /// Measured Command: /measured-command \<command\> <br/>
    /// Executes a Lua command (if allowed) and measures time it took.
    MC,
    /// Command: /command \<command\> <br/>
    /// Executes a Lua command (if allowed).
    #[default]
    C,
}

impl Display for RconCommandPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RconCommandPrefix::*;
        match self {
            Custom(prefix) => write!(f, "{}", prefix),
            SC => write!(f, "/silent-command "),
            C => write!(f, "/command "),
            MC => write!(f, "/measured-command "),
        }
    }
}

impl From<String> for RconCommandPrefix {
    fn from(value: String) -> Self {
        use RconCommandPrefix::*;
        match value.clone().to_uppercase().as_str() {
            "SC" => SC,
            "C" => C,
            "MC" => MC,
            _ => {
                debug!(
                    "RconCommandPrefix not already known, assuming it is custom: {}",
                    &value
                );
                Custom(value)
            }
        }
    }
}

impl From<RconCommandPrefix> for ValueKind {
    fn from(prefix: RconCommandPrefix) -> Self {
        match prefix {
            RconCommandPrefix::Custom(command) => Self::String(command),
            RconCommandPrefix::SC => Self::from("SC"),
            RconCommandPrefix::MC => Self::from("MC"),
            RconCommandPrefix::C => Self::from("C"),
        }
    }
}

impl TryFrom<Map<String, Value>> for RconCommandPrefix {
    type Error = anyhow::Error;

    fn try_from(value: Map<String, Value>) -> std::result::Result<Self, Self::Error> {
        match value.get("prefix") {
            Some(p) => Ok(Self::from(p.to_string())),
            None => bail!("No 'prefix' property specified."),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "commandType", content = "command")]
pub enum RconCommandLua {
    File(LuaFile),
    Inline(String),
    Other,
}

impl RconCommandLua {
    pub fn command_type(&self) -> String {
        use RconCommandLua::*;
        match self {
            File(_) => stringify!(File).to_string(),
            Inline(_) => stringify!(Inline).to_string(),
            Other => stringify!(Other).to_string(),
        }
    }
}

impl From<RconCommandLua> for ValueKind {
    fn from(lua: RconCommandLua) -> Self {
        match lua {
            RconCommandLua::File(lua_file) => Self::from(lua_file.relative_path.to_str()),
            RconCommandLua::Inline(lua) => Self::from(lua),
            RconCommandLua::Other => todo!("impl From<RconLuaCommand> for ValueKind::Other"),
        }
    }
}

impl Default for RconCommandLua {
    fn default() -> Self {
        Self::Inline(String::new())
    }
}

impl TryFrom<Map<String, Value>> for RconCommandLua {
    type Error = anyhow::Error;

    fn try_from(value: Map<String, Value>) -> std::result::Result<Self, Self::Error> {
        use RconCommandLua::*;
        let command_type = match value.get("command_type") {
            Some(t) => t.to_string(),
            None => bail!("No 'command_type' specified."),
        };
        match command_type.to_lowercase().as_str() {
            "script" => {
                let script_key = "relative_path";
                match value.get(script_key) {
                    Some(s) => {
                        let path = PathBuf::from(s.to_string());
                        Ok(File(LuaFile::new(path)))
                    }
                    None => bail!("No {} property specified.", script_key),
                }
            }
            "inline" => match value.get("inline") {
                Some(lua) => Ok(Inline(lua.to_string())),
                None => bail!("No 'inline' property specified."),
            },
            _ => {
                bail!("Invalid command type.")
            }
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LuaFile {
    /// Relative path in scripts directory.
    pub relative_path: PathBuf,
    /// Command starts as None until file is read.
    contents: Option<String>,
}

impl LuaFile {
    pub fn contents(&mut self) -> Result<String> {
        if self.contents.is_none() {
            match fs::read_to_string(self.full_path()) {
                Ok(command) => {
                    self.contents = Some(command.clone());
                    Ok(command)
                }
                Err(e) => bail!(e),
            }
        } else {
            Ok(self.contents.clone().unwrap())
        }
    }

    fn new(path: PathBuf) -> Self {
        Self {
            relative_path: path,
            contents: None,
        }
    }

    fn full_path(&mut self) -> PathBuf {
        let folder = ScriptSettings::scripts_folder();
        folder.join(&self.relative_path)
    }
}

impl RconCommandLua {
    pub fn command(&mut self) -> Result<String> {
        use RconCommandLua::*;
        match self {
            File(lua_file) => lua_file.contents(),
            Inline(command) => Ok(command.clone()),
            _ => todo!("Rcon command not implemented"),
        }
    }
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct CommandValue<T> {
//     pub data: T,
//     pub r#type: ValueType,
// }
//
// impl<T> CommandValue<T> {
//     pub fn new(data: T, r#type: ValueType) -> Self {
//         Self { data, r#type }
//     }
// }

// #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
// pub enum ValueType {
//     Bool,
//     Int,
//     String,
//     Float,
// }

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RconCommand {
    pub prefix: RconCommandPrefix,
    pub lua_command: RconCommandLua,
    // pub default_values: Option<CommandValue<T>>,
}

impl RconCommand {
    /// The complete command to transmit to the server.
    pub fn command(&mut self) -> String {
        match self.lua_command.command() {
            Ok(command) => self.prefix.to_string() + &command,
            Err(e) => {
                error!("{:?}", e);
                String::new()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Command {
    pub name: String,
    pub rcon_lua: RconCommand,
    pub triggers: Vec<Trigger>,
    pub servers: Vec<GameServer>,
}

impl TryFrom<Map<String, Value>> for Command {
    type Error = anyhow::Error;

    fn try_from(value: Map<String, Value>) -> std::result::Result<Self, Self::Error> {
        let mut errors: Vec<anyhow::Error> = vec![];
        let prefix = match RconCommandPrefix::try_from(value.clone()) {
            Ok(p) => p,
            Err(e) => {
                error!("{}", &e);
                errors.push(e);
                RconCommandPrefix::default()
            }
        };

        let lua_command = match RconCommandLua::try_from(value.clone()) {
            Ok(lua) => lua,
            Err(e) => {
                error!("{}", &e);
                errors.push(e);
                RconCommandLua::default()
            }
        };

        let rconcommand = RconCommand {
            prefix,
            lua_command,
        };

        let mut command_servers: Vec<GameServer> = Vec::new();
        let settings = Settings::new();
        let config = settings.config();
        match value.get("servers") {
            Some(servers_val) => {
                let servers = match servers_val.clone().into_array() {
                    Ok(s) => s,
                    Err(e) => bail!(e),
                };
                for server in servers {
                    let name = match server.into_table() {
                        Ok(s) => match s.get("name") {
                            Some(n) => match n.clone().into_string() {
                                Ok(n) => n,
                                Err(e) => bail!(e),
                            },
                            None => {
                                error!("One of the servers is missing a 'name' property in scripts config.");
                                bail!("One of the servers is missing a 'name' property in scripts config.");
                            }
                        },
                        Err(e) => bail!(e),
                    };
                    match servers::server_from_settings(config.clone(), &name) {
                        Some(s) => command_servers.push(s),
                        None => {
                            error!("No server found with name: {}", name);
                            bail!("No server found with name: {}", name);
                        }
                    }
                }
            }
            None => {
                info!("No servers were listed.")
            }
        };

        let triggers: Vec<Trigger> = match value.get("triggers") {
            Some(trigger) => match trigger.clone().into_array() {
                Ok(v) => v
                    .clone()
                    .iter()
                    .filter_map(|t| Trigger::try_from(t.clone()).ok())
                    .collect::<Vec<Trigger>>(),
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
            Ok(Command::from_config(
                "",
                rconcommand,
                triggers,
                command_servers,
            ))
        }
    }
}

impl Command {
    pub fn new<N: Into<String>>(name: N, rcon_lua: RconCommand) -> Self {
        Self {
            name: name.into(),
            rcon_lua,
            triggers: vec![],
            servers: vec![],
        }
    }

    pub fn set_name<N: Into<String>>(self, name: N) -> Self {
        Self {
            name: name.into(),
            ..self
        }
    }

    pub fn from_config<N, L, T, S>(name: N, rcon_lua: L, triggers: T, servers: S) -> Self
    where
        N: Into<String>,
        L: Into<RconCommand>,
        T: Into<Vec<Trigger>>,
        S: Into<Vec<GameServer>>,
    {
        Self {
            name: name.into(),
            rcon_lua: rcon_lua.into(),
            triggers: triggers.into(),
            servers: servers.into(),
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
    pub fn tx_string(&mut self) -> String {
        self.rcon_lua.command()
    }

    pub fn contains_trigger(&self, trigger: &Trigger) -> bool {
        self.triggers.contains(trigger)
    }

    /// Adds the trigger and will remove duplicate triggers.
    ///
    /// TODO: Add in running handling.
    pub fn add_trigger(&mut self, trigger: &Trigger) {
        if !self.contains_trigger(trigger) {
            self.triggers.push(trigger.clone());
        }
    }

    /// Removes the [Trigger] from this command. Returns [`Some<Trigger>`] if trigger was being used
    /// by command otherwise [None].
    pub fn remove_trigger(&mut self, trigger: &Trigger) -> Option<Trigger> {
        if self.contains_trigger(trigger) {
            self.triggers.retain(|t| t != trigger);
            Some(trigger.clone())
        } else {
            None
        }
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

        map.insert(
            "servers".to_string(),
            ValueKind::Array(
                command
                    .servers
                    .iter()
                    .map(|s| Value::new(None, ValueKind::String(s.name.clone())))
                    .collect(),
            ),
        );
        map.insert(
            "triggers".to_string(),
            ValueKind::Array(
                command
                    .triggers
                    .iter()
                    .map(|v| Value::from(v.clone()))
                    .collect(),
            ),
        );

        Self::new(None, ValueKind::from(map))
    }
}

#[tauri::command]
pub fn create_command(name: String, rcon_lua: RconCommand) -> Result<Command, String> {
    trace!("Create Command");
    let command = Command::new(name, rcon_lua);
    trace!("Created Command");
    command.add_to_commands(); // TODO: Add error handling for if command name exists.
    trace!("Added Command");
    Ok(command)
}

#[tauri::command]
pub fn get_command(name: String) -> Result<Option<Command>, String> {
    trace!("Get Command");
    Ok(Command::get(&name))
}
