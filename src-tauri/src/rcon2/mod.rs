use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use anyhow::{anyhow, Result};
use rcon::{AsyncStdStream, Connection};
use serde::{Deserialize, Serialize};
use tauri::Runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Hash)]
pub enum Game {
    #[default]
    Factorio,
}
impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Game::Factorio => write!(f, "Factorio"),
        }
    }
}

impl<T: Into<String>> From<T> for Game {
    fn from(value: T) -> Self {
        let value: String = value.into();
        match value.to_lowercase().as_str() {
            "Factorio" => Game::Factorio,
            _ => Game::Factorio,
        }
    }
}

pub static SERVERS: LazyLock<Mutex<HashMap<String, GameServer>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static CONNECTIONS: LazyLock<Mutex<HashMap<GameServer, GameServerConnected>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct GameServerConnected {
    pub server: GameServer,
    pub connection: Connection<AsyncStdStream>,
}

impl GameServerConnected {
    pub fn id(&self) -> String {
        self.server.id()
    }

    pub async fn send_command(&mut self, command: String) -> Result<String> {
        Ok(self.connection.cmd(&command).await?)
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Hash)]
pub struct GameServer {
    pub name: String,
    pub address: String,
    pub port: u32,
    pub password: String,
    pub game: Game,
}

impl std::fmt::Debug for GameServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameServer")
            .field("name", &self.name)
            .field("address", &self.address)
            .field("port", &self.port)
            .field("password", &self.password)
            .field("game", &self.game)
            .finish()
    }
}

impl GameServer {
    /// Create a new GameServer, insert it into the SERVERS HashMap and return the id.
    pub fn new<T: Into<String>>(name: T, address: T, port: u32, password: T, game: Game) -> String {
        let game_server = Self {
            name: name.into(),
            address: address.into(),
            port,
            password: password.into(),
            game,
            ..Self::default()
        };
        let mut servers = SERVERS.lock().unwrap();
        let id = game_server.id().clone();
        servers.insert(id.clone(), game_server);
        id
    }

    pub fn id(&self) -> String {
        format!("{}:{}", self.game, self.name)
    }

    pub async fn connect(&self) -> Result<GameServerConnected> {
        match <Connection<AsyncStdStream>>::builder()
            .enable_factorio_quirks(self.game == Game::Factorio)
            .connect(&self.address, &self.password)
            .await
        {
            Ok(connection) => Ok(GameServerConnected {
                server: self.clone(),
                connection,
            }),
            Err(e) => Err(anyhow!("Error connecting to server: {:?}", e)),
        }
    }
}

#[tauri::command]
async fn list_game_servers<R: Runtime>(
    app: tauri::AppHandle<R>,
    window: tauri::Window<R>,
) -> Result<Vec<GameServer>, String> {
    let servers = SERVERS.lock().unwrap();
    let servers: Vec<GameServer> = servers.values().cloned().collect();
    Ok(servers)
}
