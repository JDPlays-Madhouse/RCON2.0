use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use anyhow::{bail, Result};
use config::{Config, Value};
use rcon::{AsyncStdStream, Connection};
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use tauri::AppHandle;
use tracing::{debug, error, info};

pub static SERVERS: LazyLock<Mutex<HashMap<String, GameServer>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static CONNECTIONS: LazyLock<Mutex<HashMap<GameServer, GameServerConnected>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Valid Games go here, update the 2 lower impls like below.
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

impl TryFrom<String> for Game {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value: String = value.into();
        match value.to_lowercase().as_str() {
            "factorio" => Ok(Game::Factorio),
            _ => Err(format!("Invalid game: {:?}", value)),
        }
    }
}

pub struct GameServerConnected {
    pub server: GameServer,
    pub connection: Connection<AsyncStdStream>,
    active: bool,
}

impl GameServerConnected {
    pub async fn connect(server: GameServer) -> Result<GameServer> {
        match <Connection<AsyncStdStream>>::builder()
            .enable_factorio_quirks(server.game == Game::Factorio)
            .connect(&server.address, &server.password)
            .await
        {
            Ok(connection) => {
                let gameserverconnected = Self {
                    server: server.clone(),
                    connection,
                    active: true,
                };
                info!("Connected to server: {}", &server.id());
                CONNECTIONS
                    .lock()
                    .unwrap()
                    .insert(server.clone(), gameserverconnected);
                Ok(server)
            }
            Err(e) => {
                error!("Server {} failed to connect: {}", server.id(), e);
                bail!(e)
            }
        }
    }

    pub fn id(&self) -> String {
        self.server.id()
    }

    pub async fn send_command(&mut self, command_contents: String) -> Result<String> {
        Ok(self.connection.cmd(&command_contents).await?)
    }

    pub fn disconnect(self) -> Result<GameServer, GameServer> {
        let gameserver = self.server.clone();
        match CONNECTIONS
            .lock()
            .expect("Locking Connections")
            .remove(&gameserver)
        {
            Some(_) => Ok(gameserver),
            None => {
                error!("{}", gameserver.id());
                Err(gameserver)
            }
        }
    }
}

#[derive(Default, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct GameServer {
    pub name: String,
    pub address: String,
    pub port: u32,
    pub password: String,
    pub game: Game,
}

impl Serialize for GameServer {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Server", 6)?;
        state.serialize_field("id", &self.id())?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("game", &self.game)?;
        state.serialize_field("address", &self.address)?;
        state.serialize_field("port", &self.port)?;
        state.serialize_field("password", &self.password)?;
        state.end()
    }
}

impl std::fmt::Debug for GameServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameServer")
            .field("name", &self.name)
            .field("address", &self.address)
            .field("port", &self.port)
            .field("password", &"[...]")
            .field("game", &self.game)
            .finish()
    }
}

impl GameServer {
    /// Create a new GameServer, insert it into the SERVERS HashMap and return the id.
    pub fn new<T: Into<String>>(name: T, address: T, port: u32, password: T, game: Game) -> Self {
        let game_server = Self {
            name: name.into(),
            address: address.into(),
            port,
            password: password.into(),
            game,
        };
        let mut servers = SERVERS.lock().unwrap();
        let id = game_server.id().clone();
        servers.insert(id.clone(), game_server.clone());
        game_server
    }

    pub fn id(&self) -> String {
        format!("{}:{}", self.game, self.name)
    }

    pub async fn connect(&self) -> Result<GameServer> {
        GameServerConnected::connect(self.clone()).await
    }
}

impl GameServer {
    fn try_from_config(
        server_name: String,
        server_config: Value,
    ) -> std::result::Result<Self, anyhow::Error> {
        let map = match server_config.clone().into_table() {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to convert into Server: {:?}", server_config);
                bail!(e)
            }
        };
        let address = map.get("address").unwrap().clone().into_string().unwrap();
        let port = map.get("port").unwrap().clone().into_uint().unwrap() as u32;
        let password = map.get("password").unwrap().clone().into_string().unwrap();
        let game = Game::try_from(map.get("game").unwrap().clone().into_string().unwrap()).unwrap();

        Ok(GameServer::new(server_name, address, port, password, game))
    }
}

pub fn servers_from_settings(config: Config) -> Result<Vec<GameServer>> {
    match config.get_table("servers") {
        Ok(servers_conf) => {
            let mut servers_conf = servers_conf.clone();
            let default_server = servers_conf
                .shift_remove("default")
                .unwrap()
                .into_string()
                .unwrap();
            let autostart_server = servers_conf
                .shift_remove("autostart")
                .unwrap()
                .into_bool()
                .unwrap();

            for (name, server) in servers_conf {
                let server = GameServer::try_from_config(name, server).unwrap();
                SERVERS.lock().unwrap().insert(server.id(), server);
            }
            debug!("{:?}", SERVERS.lock().unwrap());
            Ok(SERVERS.lock().unwrap().clone().into_values().collect())
        }
        Err(_) => todo!(),
    }
}

pub fn server_from_settings(config: Config, name: String) -> Option<GameServer> {
    match config.get_table("servers") {
        Ok(servers_conf) => {
            if let Some(server) = servers_conf.get(&name) {
                let server = GameServer::try_from_config(name, server.clone()).unwrap();
                Some(server)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

#[tauri::command]
pub async fn list_game_servers(// app: tauri::AppHandle<R>,
    // window: tauri::Window<R>,
) -> Result<Vec<GameServer>, String> {
    let servers = SERVERS.lock().unwrap();
    let servers: Vec<GameServer> = servers.values().cloned().collect();
    Ok(servers)
}

#[tauri::command]
pub fn get_default_server(app_handle: AppHandle) -> Result<GameServer, String> {
    let settings = crate::settings::Settings::new();
    let config = settings.config();
    let default_server = match config.get_string("servers.default") {
        Ok(server) => server,
        Err(_) => {
            return Err(
                "No default server found. Select a sever to set it as the default.".to_string(),
            )
        }
    };
    app_handle.send_tao_window_event(window_id, WindowMessage::RequestRedraw);
    match server_from_settings(config, default_server) {
        Some(server) => Ok(server),
        None => {
            Err("No default server found. Select a sever to set it as the default.".to_string())
        }
    }
}

#[tauri::command]
pub fn set_default_server(server_name: String) -> Result<String, String> {
    let mut settings = crate::settings::Settings::new();
    match settings.set_config("servers.default", server_name) {
        Ok(_) => Ok("Default server set".to_string()),
        Err(e) => Err(format!("Failed to set default server: {:?}", e)),
    }
}
