use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
};
use trigger::Trigger;
use uuid::Uuid;

pub mod settings;
pub mod trigger;

static COMMANDS: LazyLock<Arc<Mutex<HashMap<String, Command>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash)]
#[serde(tag = "type", content = "data")]
pub enum CommandType {
    ChannelPoints(String),
    Chat,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "commandType", content = "command")]
pub enum RconLuaCommand {
    File {
        /// Relative path in scripts directory.
        path: PathBuf,
        /// Command starts as None until file is read.
        command: Option<String>,
    },
    Inline(String),
}

impl RconLuaCommand {
    pub fn command(&self) -> String {
        use RconLuaCommand::*;
        match self {
            File { path, command } => {
                todo!("read the file")
            }
            Inline(command) => command.clone(),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RconCommand {
    pub prefix: RconCommandPrefix,
    pub lua_command: RconLuaCommand,
    // pub default_values: Option<CommandValue<T>>,
}

impl RconCommand {
    /// The complete command to transmit to the server.
    pub fn command(&self) -> String {
        self.prefix.to_string() + &self.lua_command.command()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub name: String,
    id: String,
    pub variant: CommandType,
    pub rcon_lua: RconCommand,
    pub triggers: Vec<Trigger>,
}

impl Command {
    pub fn new(name: String, variant: CommandType, rcon_lua: RconCommand) -> Self {
        let id = Uuid::now_v7().to_string();
        Self {
            name,
            id: id.clone(),
            variant,
            rcon_lua,
            triggers: vec![],
        }
    }

    pub fn from_config<I, V, L, T>(name: I, id: I, variant: V, rcon_lua: L, triggers: T) -> Self
    where
        I: Into<String>,
        V: Into<CommandType>,
        L: Into<RconCommand>,
        T: Into<Vec<Trigger>>,
    {
        Self {
            name: name.into(),
            id: id.into(),
            variant: variant.into(),
            rcon_lua: rcon_lua.into(),
            triggers: triggers.into(),
        }
    }

    pub fn add_to_commands(&self) -> String {
        let mut commands = COMMANDS.lock().unwrap();
        commands.insert(self.id(), self.clone());
        self.id.clone()
    }

    pub fn get(id: &str) -> Option<Self> {
        let commands = COMMANDS.lock().unwrap();
        commands.get(id).cloned()
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn tx_string(&self) -> String {
        self.rcon_lua.command()
    }
}

impl From<Command> for config::ValueKind {
    fn from(value: Command) -> Self {
        // let map = Map::new();
        // map.insert();
        // Self::Table(map)
        todo!()
    }
}

#[tauri::command]
pub fn create_command(
    name: String,
    variant: CommandType,
    rcon_lua: RconCommand,
) -> Result<Command, String> {
    let command = Command::new(name, variant, rcon_lua);
    command.add_to_commands(); // TODO: Add error handling for if command name exists.
    Ok(command)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use rstest::rstest;
//
//     #[rstest]
//     #[case::integer(8, ValueType::Int)]
//     #[case::float(8.1, ValueType::Float)]
//     #[case::bool(true, ValueType::Bool)]
//     #[case::string("test", ValueType::String)]
//     fn command_value_data<T>(#[case] input: T, #[case] input_type: ValueType)
//     where
//         T: std::fmt::Debug + std::cmp::PartialEq + std::clone::Clone,
//     {
//         let t = CommandValue::new(input.clone(), input_type);
//         assert_eq!(t.data, input);
//     }
//
//     #[rstest]
//     #[case::integer(8, ValueType::Int)]
//     #[case::float(8.1, ValueType::Float)]
//     #[case::bool(true, ValueType::Bool)]
//     #[case::string("test", ValueType::String)]
//     fn command_value_type<T>(#[case] input: T, #[case] input_type: ValueType)
//     where
//         T: std::fmt::Debug + std::cmp::PartialEq + std::clone::Clone,
//     {
//         let t = CommandValue::new(input.clone(), input_type);
//         assert_eq!(t.r#type, input_type);
//     }
// }
