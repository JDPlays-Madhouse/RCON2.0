use serde::{Deserialize, Serialize};
use tracing::error;

use super::{Prefix, RconCommandLua};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RconCommand {
    pub prefix: Prefix,
    pub lua_command: RconCommandLua,
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
