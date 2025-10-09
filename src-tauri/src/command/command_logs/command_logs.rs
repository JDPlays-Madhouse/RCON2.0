use std::{
    ops::{Deref, DerefMut},
    sync::LazyLock,
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

use crate::command::command_logs::CommandLog;
use crate::{Arc, AsyncMutex};

pub static COMMAND_LOGS: LazyLock<Arc<AsyncMutex<CommandLogs>>> =
    LazyLock::new(|| Arc::new(AsyncMutex::new(CommandLogs::new())));

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandLogs(Vec<CommandLog>);

impl CommandLogs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the [`CommandLog`] to end of [`Vec`], if not added in order then [`CommandLogs::sort()`] is run.
    pub fn add_log(&mut self, command_log: CommandLog) {
        self.0.push(command_log);
        if !self.is_sorted() {
            self.sort()
        }
    }

    pub fn all_logs(&self) -> &[CommandLog] {
        self.0.as_slice()
    }

    pub fn all_logs_since(&self, since: SystemTime) -> &[CommandLog] {
        let i = self.0.partition_point(|l| l.time() < since);
        &self.0[i..]
    }

    pub fn sort(&mut self) {
        self.0.sort();
    }

    pub fn is_sorted(&self) -> bool {
        self.0.is_sorted()
    }

    pub fn last_log(&self) -> Option<&CommandLog> {
        self.0.last()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl AsRef<Vec<CommandLog>> for CommandLogs {
    fn as_ref(&self) -> &Vec<CommandLog> {
        &self.0
    }
}

impl AsMut<Vec<CommandLog>> for CommandLogs {
    fn as_mut(&mut self) -> &mut Vec<CommandLog> {
        &mut self.0
    }
}

impl Deref for CommandLogs {
    type Target = Vec<CommandLog>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CommandLogs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[tauri::command]
pub async fn get_command_logs() -> Result<CommandLogs, String> {
    Ok(COMMAND_LOGS.lock().await.clone())
}
// #[tauri::command]
// pub fn get_command_logs_since(since: SystemTime) -> Result<CommandLogs, String> {
//     todo!();
// }

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, time::Duration};

    use crate::{
        command::{
            Command, GameServerTrigger, Prefix, RconCommand, RconCommandLua, Trigger, Variable,
        },
        integration::IntegrationEvent,
        servers::{Game, GameServer},
    };

    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn system_times() -> Vec<SystemTime> {
        let mut times = vec![];
        let now = SystemTime::UNIX_EPOCH + Duration::from_millis(1759980000);
        for i in 0..10 {
            times.push(now + Duration::from_secs(i))
        }
        times.sort();
        times
    }

    #[fixture]
    fn rcon_command(game_server_trigger: GameServerTrigger) -> Command {
        let variables = Variable::from_config("x:int=5").unwrap();
        let command = RconCommand {
            prefix: Prefix::SC,
            lua_command: RconCommandLua::Inline("game.print(x);".to_string()),
            variables,
        };
        Command::from_config("test_command", command, vec![game_server_trigger])
    }

    #[fixture]
    fn server() -> GameServer {
        GameServer::new(
            "test",
            "localhost",
            2871,
            "secure_password",
            Game::Factorio,
            None,
            <Option<SocketAddr>>::None,
            None,
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
    fn chat_event() -> IntegrationEvent {
        IntegrationEvent::Chat {
            msg: "Hello".into(),
            author: "Legend".into(),
        }
    }

    #[fixture]
    fn game_server_trigger(server: GameServer, trigger: Trigger) -> GameServerTrigger {
        GameServerTrigger::new(server, trigger)
    }

    #[fixture]
    fn test_command_logs(
        rcon_command: Command,
        game_server_trigger: GameServerTrigger,
        chat_event: IntegrationEvent,
    ) -> CommandLogs {
        let mut command_logs = CommandLogs::new();
        for time in system_times() {
            let command_log = CommandLog {
                time,
                command: rcon_command.clone(),
                trigger: game_server_trigger.clone(),
                username: "Legend".into(),
                message: None,
                event: chat_event.clone(),
            };
            command_logs.push(command_log);
        }

        command_logs
    }

    #[rstest]
    fn all_logs_since(test_command_logs: CommandLogs, system_times: Vec<SystemTime>) {
        assert_eq!(test_command_logs.len(), system_times.len());

        let half = system_times.len() / 2;
        let pick = system_times[half];
        let logs_since = test_command_logs.all_logs_since(pick);
        for (time, log) in system_times[half..].iter().zip(logs_since) {
            assert_eq!(*time, log.time(), "{time:?} == {:?}", log.time());
        }

        assert_eq!(logs_since.len(), half);
    }
}
