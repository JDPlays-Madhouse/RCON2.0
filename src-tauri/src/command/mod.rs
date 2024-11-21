use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
};
use uuid::Uuid;

static COMMANDS: LazyLock<Arc<Mutex<HashMap<String, Command>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CommandType {
    ChannelPoints(String),
    Chat,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
pub enum RconLuaType {
    File(PathBuf),
    Inline(String),
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
    pub lua_type: RconLuaType,
    // pub default_values: Option<CommandValue<T>>,
}

impl RconCommand {
    pub fn lua(&self) -> String {
        use RconLuaType::*;
        match &self.lua_type {
            Inline(command) => self.prefix.to_string() + command,
            lua_type => todo!("Needs to be implemented: {:?}", lua_type),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    id: String,
    pub variant: CommandType,
    pub rcon_lua: RconCommand,
}

impl Command {
    pub fn new(variant: CommandType, rcon_lua: RconCommand) -> Self {
        let id = Uuid::now_v7().to_string();
        let cmd = Self {
            id: id.clone(),
            variant,
            rcon_lua,
        };
        let mut commands = COMMANDS.lock().unwrap();
        commands.insert(id, cmd.clone());
        cmd
    }

    pub fn get(id: &str) -> Option<Self> {
        let commands = COMMANDS.lock().unwrap();
        commands.get(id).cloned()
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn tx_string(&self) -> String {
        let tx = self.rcon_lua.prefix.to_string();
        todo!("tx_string")
    }
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
//     fn command_value_data<T>(#[ca<se] input: T, #[case] input_type: ValueType)
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
