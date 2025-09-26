use crate::command::Variable;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use tracing::error;

use super::{Prefix, RconCommandLua};
/// variables = "x:float=0,y:float=0"
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RconCommand {
    pub prefix: Prefix,
    pub lua_command: RconCommandLua,
    pub variables: Option<Vec<Variable>>,
}

impl RconCommand {
    /// The complete command to transmit to the server.
    pub fn command(&mut self, message: Option<&str>, username: &str) -> String {
        match self.lua_command.command() {
            Ok(command) => {
                let mut commmand_string = self.prefix.to_string();

                if let Some(variables) = &self.variables {
                    for variable in variables {
                        let v = variable
                            .from_message(message)
                            .map(|v| v.variable_type().clone());
                        if let Err(e) = writeln!(
                            commmand_string,
                            "{}",
                            variable.command_local_lua(v, username)
                        ) {
                            error!(
                                "Error occured while printing variable {}: {e}",
                                variable.name()
                            );
                        };
                    }
                }
                if let Err(e) = write!(commmand_string, "{}", command) {
                    error!("Error occured while printing command: {e}");
                };

                commmand_string
            }
            Err(e) => {
                error!("{:?}", e);
                eprintln!("{:?}", e);
                String::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn inline_command_print() {
        let variables = Variable::from_config("x:int=5").unwrap();
        let mut command = RconCommand {
            prefix: Prefix::SC,
            lua_command: RconCommandLua::Inline("game.print(x);".to_string()),
            variables,
        };
        let expected = "/silent-command local x = 5;\ngame.print(x);";
        assert_eq!(command.command(None, "test").as_str(), expected);
    }

    #[rstest]
    fn inline_command_print_with_variable() {
        let variables = Variable::from_config("x:int=5,y=hello").unwrap();
        let mut command = RconCommand {
            prefix: Prefix::SC,
            lua_command: RconCommandLua::Inline("game.print(x);".to_string()),
            variables,
        };
        let expected = "/silent-command local x = 20;\nlocal y = \"hello\";\ngame.print(x);";
        assert_eq!(
            command.command(Some("you suck x=20"), "test").as_str(),
            expected
        );
    }
}
