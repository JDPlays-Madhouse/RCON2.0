use std::{fs, path::PathBuf};

use anyhow::{bail, Result};
use config::{Map, Value, ValueKind};
use serde::{Deserialize, Serialize};

use super::settings::ScriptSettings;

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
