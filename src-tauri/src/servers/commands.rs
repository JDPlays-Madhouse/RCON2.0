use serde::{Deserialize, Serialize};
use std::process::{Command, Output, Stdio};

#[derive(Debug, Default, Deserialize, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ServerCommands {
    start: Option<String>,
    stop: Option<String>,
}

impl ServerCommands {
    pub fn new(start: Option<String>, stop: Option<String>) -> Self {
        let mut mut_self = Self::default();
        mut_self.set_start(start).set_stop(stop);
        mut_self
    }

    pub fn optional_new(start: Option<String>, stop: Option<String>) -> Option<ServerCommands> {
        if start.is_none() & stop.is_none() {
            None
        } else {
            Some(Self::new(start, stop))
        }
    }

    pub fn set_start(&mut self, start: Option<String>) -> &mut Self {
        if let Some(v) = start {
            if v.trim().is_empty() {
                self.start = None
            } else {
                self.start = Some(v.trim().to_string())
            }
        } else {
            self.start = None
        };
        self
    }
    pub fn set_stop(&mut self, stop: Option<String>) -> &mut Self {
        if let Some(v) = stop {
            if v.trim().is_empty() {
                self.stop = None
            } else {
                self.stop = Some(v.trim().to_string())
            }
        } else {
            self.stop = None
        };
        self
    }

    pub fn start(&self) -> Option<&String> {
        self.start.as_ref()
    }

    pub fn stop(&self) -> Option<&String> {
        self.stop.as_ref()
    }
}

impl ServerCommands {
    /// Set current working directory to home.
    /// Pipes stdin, stdout, stderr so visible in [Output].
    fn common_command_prep(command: &mut Command) {
        if let Some(home_dir) = dirs::home_dir() {
            command.current_dir(home_dir);
        };
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
    }

    /// Uses [ServerCommands::common_command_prep] for all commands.
    pub fn start_command(&self) -> Option<Command> {
        if self.start().is_none() {
            return None;
        }
        let start = self.start.clone().unwrap();
        let mut start_command = start.split_ascii_whitespace();
        let program = start_command.next().unwrap_or_default();
        let mut command = Command::new(program);
        command.args(start_command);
        ServerCommands::common_command_prep(&mut command);
        Some(command)
    }

    /// Uses [ServerCommands::common_command_prep] for all commands.
    pub fn stop_command(&self) -> Option<Command> {
        if self.stop().is_none() {
            return None;
        }
        let stop = self.stop.clone().unwrap();
        let mut stop_command = stop.split_ascii_whitespace();
        let program = stop_command.next().unwrap_or_default();
        let mut command = Command::new(program);
        command.args(stop_command);
        ServerCommands::common_command_prep(&mut command);
        Some(command)
    }
    pub fn run_start(&self) -> Result<Output, ServerCommandError> {
        if let Some(mut command) = self.start_command() {
            command.output().map_err(ServerCommandError::IOError)
        } else {
            return Err(ServerCommandError::NoStartCommand);
        }
    }
    pub fn run_stop(&self) -> Result<Output, ServerCommandError> {
        if let Some(mut command) = self.stop_command() {
            command.output().map_err(ServerCommandError::IOError)
        } else {
            return Err(ServerCommandError::NoStopCommand);
        }
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ServerCommandError {
    #[error("No Start command to run")]
    NoStartCommand,
    #[error("No Stop command to run")]
    NoStopCommand,
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {

    use rstest::{fixture, rstest};

    use super::*;

    #[fixture]
    fn commands() -> ServerCommands {
        ServerCommands::new(
            Some("echo 'start server'".to_string()),
            Some("echo 'stop server'".to_string()),
        )
    }

    #[rstest]
    fn start_command(commands: ServerCommands) {
        let mut start = commands.start_command().unwrap();
        let output = start.output().unwrap();
        eprintln!("{:?}", output);

        assert!(
            output.status.success(),
            "failure when running: {}",
            commands.start().unwrap()
        );

        assert_eq!(
            String::from_utf8(output.stdout)
                .unwrap()
                .as_str()
                .trim_end(),
            "start server"
        );
    }

    #[rstest]
    fn stop_command(commands: ServerCommands) {
        let mut stop = commands.stop_command().unwrap();
        let output = stop.output().unwrap();
        eprintln!("{:?}", output);

        assert!(
            output.status.success(),
            "failure when running: {}",
            commands.stop().unwrap()
        );

        assert_eq!(
            String::from_utf8(output.stdout)
                .unwrap()
                .as_str()
                .trim_end(),
            "stop server"
        );
    }

    #[rstest]
    fn run_start_command(commands: ServerCommands) {
        let output = commands.run_start().unwrap();
        eprintln!("{:?}", output);

        assert!(
            output.status.success(),
            "failure when running: {}",
            commands.start().unwrap()
        );

        assert_eq!(
            String::from_utf8(output.stdout)
                .unwrap()
                .as_str()
                .trim_end(),
            "start server"
        );
    }

    #[rstest]
    fn run_stop_command(commands: ServerCommands) {
        let output = commands.run_stop().unwrap();
        eprintln!("{:?}", output);

        assert!(
            output.status.success(),
            "failure when running: {}",
            commands.stop().unwrap()
        );

        assert_eq!(
            String::from_utf8(output.stdout)
                .unwrap()
                .as_str()
                .trim_end(),
            "stop server"
        );
    }
}
