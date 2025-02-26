use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use anyhow::{bail, Result};
use config::{Config, Value};
use rcon::Connection;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use tauri::{ipc::Channel, State};
use tokio::net::TcpStream;
use tracing::{debug, error, info, instrument, trace};

use crate::{command::Command, settings::Settings};

pub static SERVERS: LazyLock<Mutex<HashMap<String, GameServer>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Any changes to a [GameServer] will become a new key therefore the connection will be lost.
pub static CONNECTIONS: LazyLock<tokio::sync::Mutex<HashMap<GameServer, GameServerConnected>>> =
    LazyLock::new(|| tokio::sync::Mutex::new(HashMap::new()));

/// Valid Games
///
/// Dev notes: Update the 2 lower impls (`impl std::fmt::Display for Game`, `impl TryFrom<String> for Game`) to match Factorio.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Hash, PartialOrd, Ord)]
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
        match value.to_lowercase().as_str() {
            "factorio" => Ok(Game::Factorio),
            _ => Err(format!("Invalid game: {:?}", value)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum ServerStatus {
    Connecting { server: GameServer },
    Checking { server: GameServer },
    Connected { server: GameServer },
    Error { msg: String, server: GameServer },
    Disconnected { server: Option<GameServer> },
}

pub struct GameServerConnected {
    pub server: GameServer,
    pub connection: Connection<TcpStream>,
    pub channel: Channel<ServerStatus>,
}

impl std::fmt::Debug for GameServerConnected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameServerConnected")
            .field("server", &self.server)
            .finish()
    }
}

impl GameServerConnected {
    pub async fn connect(server: GameServer, channel: Channel<ServerStatus>) -> Result<GameServer> {
        match channel.send(ServerStatus::Connecting {
            server: server.clone(),
        }) {
            Ok(_) => info!("Sending Connecting"),
            Err(e) => error!("{:?}", e),
        };
        {
            if (CONNECTIONS.lock().await).contains_key(&server) {
                let _ = channel.send(ServerStatus::Connected {
                    server: server.clone(),
                });
                return Ok(server);
            }
        }
        match <Connection<TcpStream>>::builder()
            .enable_factorio_quirks(server.game == Game::Factorio)
            .connect(&server.socket_address(), &server.password)
            .await
        {
            Ok(connection) => {
                let _ = channel.send(ServerStatus::Connected {
                    server: server.clone(),
                });
                let gameserverconnected = Self {
                    server: server.clone(),
                    channel,
                    connection,
                };
                info!("Connected to server: {}", &server.name);
                {
                    CONNECTIONS
                        .lock()
                        .await
                        .insert(server.clone(), gameserverconnected);
                }
                Ok(server)
            }
            Err(e) => {
                let _ = channel.send(ServerStatus::Error {
                    msg: format!("{e:?}"),
                    server: server.clone(),
                });
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

    pub async fn handshake(&self) -> ServerStatus {
        match TcpStream::connect(&self.server.socket_address()).await {
            Ok(_io) => ServerStatus::Connected {
                server: self.server.clone(),
            },
            Err(_e) => ServerStatus::Disconnected {
                server: Some(self.server.clone()),
            },
        }
    }

    pub async fn disconnect(server: GameServer) -> Result<GameServer, GameServer> {
        info!("Disconnecting from: {}", &server.name);
        match CONNECTIONS.lock().await.remove_entry(&server) {
            Some((s, c)) => {
                let channel = c.channel;
                let _ = channel.send(ServerStatus::Disconnected {
                    server: Some(s.clone()),
                });

                Ok(s)
            }
            None => Err(server),
        }
    }
}

#[derive(Default, Deserialize, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

    pub async fn connect(&self, channel: Channel<ServerStatus>) -> Result<GameServer> {
        GameServerConnected::connect(self.clone(), channel).await
    }

    pub fn socket_address(&self) -> String {
        self.address.clone() + ":" + &self.port.to_string()
    }

    pub fn try_get(id: &str) -> Option<GameServer> {
        if let Some(server) = SERVERS.lock().expect("Locking SERVERS").get(id) {
            return Some(server.clone());
        };
        let config = Settings::current_config();
        server_from_settings(config, id)
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
            let _default_server = servers_conf
                .shift_remove("default")
                .unwrap()
                .into_string()
                .unwrap();
            let _autostart_server = servers_conf
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

pub fn server_from_settings(config: Config, name: &str) -> Option<GameServer> {
    match config.get_table("servers") {
        Ok(servers_conf) => {
            if let Some(server) = servers_conf.get(name) {
                let server = GameServer::try_from_config(name.into(), server.clone()).unwrap();
                Some(server)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn default_server_from_settings(config: Config) -> Option<GameServer> {
    let default_server = match config.get_string("servers.default") {
        Ok(server) => server,
        Err(_) => {
            return None;
        }
    };

    server_from_settings(config, &default_server.to_lowercase())
}
#[tauri::command]
#[instrument(level = "trace")]
pub async fn list_game_servers(
    config: State<'_, Arc<futures::lock::Mutex<Config>>>,
) -> Result<Vec<GameServer>, String> {
    let config = config.lock().await.clone();
    let servers = servers_from_settings(config).unwrap_or_default();
    Ok(servers)
}

#[tauri::command]
#[instrument(level = "trace")]
pub async fn get_default_server(
    default_server: State<'_, Arc<futures::lock::Mutex<Option<GameServer>>>>,
    config: State<'_, Arc<futures::lock::Mutex<Config>>>,
) -> Result<GameServer, String> {
    let default_server_lock = default_server.lock().await;
    if let Some(server) = default_server_lock.clone() {
        return Ok(server);
    }
    let config = config.lock().await.clone();
    match default_server_from_settings(config) {
        Some(server) => Ok(server),
        None => {
            Err("No default server found. Select a server to set it as the default.".to_string())
        }
    }
}

#[tauri::command]
#[instrument(level = "trace")]
pub async fn set_default_server(
    default_server: State<'_, Arc<futures::lock::Mutex<Option<GameServer>>>>,
    server_name: String,
) -> Result<String, String> {
    let mut settings = crate::settings::Settings::new();
    let server = match server_from_settings(settings.config(), &server_name.to_lowercase()) {
        Some(s) => s,
        None => return Err(format!("No server found with that name: {:?}", server_name)),
    };
    {
        let mut default_server_lock = default_server.lock().await;
        *default_server_lock = Some(server);
    }
    match settings.set_config("servers.default", server_name.to_lowercase()) {
        Ok(_) => Ok("Default server set".to_string()),
        Err(e) => Err(format!("Failed to set default server: {:?}", e)),
    }
}

/// TODO: Change to a `impl From<GameServer> for ConfigValue`
#[tauri::command]
#[instrument(level = "trace")]
pub fn new_server(server: GameServer) -> Result<GameServer, String> {
    info! {"adding new rcon server: {:?}", server};
    let ret_server = server.clone();
    let mut settings = crate::settings::Settings::new();
    settings
        .set_config(&format!("servers.{}.address", server.name), server.address)
        .unwrap();
    settings
        .set_config(
            &format!("servers.{}.game", server.name),
            server.game.to_string(),
        )
        .unwrap();
    settings
        .set_config(
            &format!("servers.{}.password", server.name),
            server.password,
        )
        .unwrap();
    settings
        .set_config(&format!("servers.{}.port", server.name), server.port)
        .unwrap();
    Ok(ret_server)
}

#[tauri::command]
#[instrument(level = "trace")]
pub fn update_server(server: GameServer, old_server_name: String) -> Result<GameServer, String> {
    info! {"updating rcon server {old_server_name}: {:?}", server};

    let ret_server = server.clone();
    let mut settings = crate::settings::Settings::new();

    settings
        .set_config("servers.default", server.name.clone())
        .unwrap();
    settings
        .set_config(&format!("servers.{}.address", server.name), server.address)
        .unwrap();
    settings
        .set_config(
            &format!("servers.{}.game", server.name),
            server.game.to_string(),
        )
        .unwrap();
    settings
        .set_config(
            &format!("servers.{}.password", server.name),
            server.password,
        )
        .unwrap();
    settings
        .set_config(&format!("servers.{}.port", server.name), server.port)
        .unwrap();
    if server.name != old_server_name {
        let _ = settings.remove_config(&format!("servers.{}", old_server_name));
    }
    Ok(ret_server)
}

#[tauri::command]
#[instrument(level = "trace", skip(channel))]
pub async fn connect_to_server(
    channel: Channel<ServerStatus>,
    server: GameServer,
) -> Result<ServerStatus, ServerStatus> {
    match channel.send(ServerStatus::Connecting {
        server: server.clone(),
    }) {
        Ok(_) => info!("Sending Connecting"),
        Err(e) => error!("{:?}", e),
    };
    match server.connect(channel.clone()).await {
        Ok(s) => {
            let status = ServerStatus::Connected { server: s.clone() };
            let _ = channel.send(status.clone());
            Ok(status)
        }
        Err(e) => {
            error!("{e:?}"); // TODO: Handle errors.
            let status = ServerStatus::Error {
                msg: format!("{:?}", e),
                server,
            };
            let _ = channel.send(status.clone());
            Err(status)
        }
    }
}

#[tauri::command]
#[instrument(level = "debug")]
pub async fn send_command_to_server(
    server: GameServer,
    mut command: Command,
) -> Result<String, String> {
    trace!("send_command_to_server");
    let mut connections = CONNECTIONS.lock().await;
    trace!("CONNECTIONS Locked");
    let connection: &mut GameServerConnected = match connections.get_mut(&server) {
        Some(c) => c,
        None => return Err("Server not connected to.".to_string()),
    };
    match connection.send_command(command.tx_string()).await {
        Ok(r) => {
            trace!("CONNECTIONS Unlocked");
            Ok(r)
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}

#[tauri::command]
#[instrument(level = "trace")]
pub async fn check_connection(server: GameServer) -> ServerStatus {
    trace!("check_connection");
    let connections = CONNECTIONS.lock().await;
    trace!("CONNECTIONS Locked");
    let conn = connections.get(&server);
    let status = match conn {
        Some(c) => {
            let status = c.handshake().await;
            let _ = c.channel.send(status.clone());
            status
        }
        None => ServerStatus::Disconnected {
            server: Some(server.clone()),
        },
    };
    trace!("CONNECTIONS Unlocked");

    info!("Server Status: {:?}", status);
    status
}

#[tauri::command]
#[instrument(level = "trace")]
pub async fn disconnect_connection(server: GameServer) -> ServerStatus {
    match GameServerConnected::disconnect(server).await {
        Ok(s) => ServerStatus::Disconnected { server: Some(s) },
        Err(e) => {
            let msg = format!(
                "Error occurred while disconnecting from server \"{}\".",
                e.name
            );
            error!("{}", &msg);
            ServerStatus::Error { server: e, msg }
        }
    }
}
