use std::fmt::Display;

use anyhow::bail;
use config::{Map, Value, ValueKind};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(tag = "prefix", content = "data")]
pub enum Prefix {
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

impl Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Prefix::*;
        match self {
            Custom(prefix) => write!(f, "{}", prefix),
            SC => write!(f, "/silent-command "),
            C => write!(f, "/command "),
            MC => write!(f, "/measured-command "),
        }
    }
}

impl From<String> for Prefix {
    fn from(value: String) -> Self {
        use Prefix::*;
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

impl From<Prefix> for ValueKind {
    fn from(prefix: Prefix) -> Self {
        match prefix {
            Prefix::Custom(command) => Self::String(command),
            Prefix::SC => Self::from("SC"),
            Prefix::MC => Self::from("MC"),
            Prefix::C => Self::from("C"),
        }
    }
}

impl TryFrom<Map<String, Value>> for Prefix {
    type Error = anyhow::Error;

    fn try_from(value: Map<String, Value>) -> std::result::Result<Self, Self::Error> {
        match value.get("prefix") {
            Some(p) => Ok(Self::from(p.to_string())),
            None => bail!("No 'prefix' property specified."),
        }
    }
}
