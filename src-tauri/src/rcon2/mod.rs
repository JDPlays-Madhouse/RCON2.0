use anyhow::{anyhow, Result};
use rcon::{AsyncStdStream, Connection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Game {
    Factorio,
}

pub struct GameServer {
    pub address: &'static str,
    pub port: u32,
    pub password: String,
    pub connection: Option<Connection<AsyncStdStream>>,
    pub game: Game,
    pub channel: String,
    pub logger: Option<String>,
}

impl GameServer {
    pub fn new(address: &'static str, port: u32, password: String) -> Self {
        Self {
            address,
            port,
            password,
            connection: None,
            game: Game::Factorio,
            channel: String::from("rx"),
            logger: None,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        self.connection = Some(
            <Connection<AsyncStdStream>>::builder()
                .enable_factorio_quirks(self.game == Game::Factorio)
                .connect(&self.address, &self.password)
                .await?,
        );
        Ok(())
    }

    pub async fn send_command(&mut self, command: String) -> Result<String> {
        match self.connection.as_mut() {
            Some(connection) => Ok(connection.cmd(&command).await?),
            None => Err(anyhow!("No Connection available.")),
        }
    }
}
