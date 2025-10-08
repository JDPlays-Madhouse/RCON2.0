use std::{
    marker::PhantomData,
    net::SocketAddr,
    time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH},
};

use reqwest::Client;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use tracing::instrument;

use crate::servers::{Game, GameServer};

static GET_ALL_GAMES_URL: &str = "https://multiplayer.factorio.com/get-games";

static GET_GAME_DETAILS_URL: &str = "https://multiplayer.factorio.com/get-game-details/";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FactorioServerStatus {
    name: String,
    host_address: SocketAddr,
    #[serde(deserialize_with = "de_time")]
    last_heartbeat: SystemTime,
    now: SystemTime,
    duration_since_last_heartbeat: Duration,
    game_id: u64,
    server_id: String,
}

/// Json object recieved from [`GET_ALL_GAMES_URL`]
///
/// ### Status 404 Error
///
/// - [Server is unreachable][MatchMakingApiError::ServerUnreachable]
///
/// ### Shape of response
///
/// ```json
/// {
///     "application_version": {
///         "build_mode": "headless",
///         "build_version": 83840,
///         "game_version": "2.0.66",
///         "platform": "linux64"
///     },
///     "description": "https://github.com/JDPlays-Madhouse/RCON2.0",
///     "game_id": 22382891,
///     "game_time_elapsed": 10292,
///     "has_password": true,
///     "headless_server": true,
///     "host_address": "0.0.0.0:61005",
///     "host_port": 61005,
///     "last_heartbeat": 1759370020.054248,
///     "max_players": 0,
///     "mods": [
///         {
///             "name": "base",
///             "version": "2.0.66"
///         },
///         {
///             "name": "common-prototypes-graphics",
///             "version": "0.0.2"
///         },
///         {
///             "name": "MileStoneSaves",
///             "version": "2.0.0"
///         },
///         ...
///     ],
///     "mods_crc": 0,
///     "name": "Ozy_Viking: RCON2.0 Development",
///     "players": [
///         "Ozy_Viking"
///     ],
///     "server_id": "dW5kZXJsaW5laW5oaWRkZW50aGVzZXdpbmdicmFzc3doZW5ldmVybGlzdG8=",
///     "tags": [
///         "Dev",
///         "RCON"
///     ]
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ServerDescription {
    application_version: ApplicationVersion,
    description: String,
    game_id: u64,
    game_time_elapsed: u64,
    has_password: bool,
    host_address: SocketAddr,
    #[serde(deserialize_with = "de_time")]
    last_heartbeat: SystemTime,
    max_players: u64,
    mods: Vec<Mod>,
    mods_crc: u64,
    name: String,
    players: Option<Vec<String>>,
    require_user_verification: Option<bool>,
    server_id: String,
    tags: Option<Vec<String>>,
}

impl ServerDescription {
    pub fn application_version(&self) -> &ApplicationVersion {
        &self.application_version
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn game_id(&self) -> u64 {
        self.game_id
    }

    pub fn game_time_elapsed(&self) -> u64 {
        self.game_time_elapsed
    }

    pub fn has_password(&self) -> bool {
        self.has_password
    }

    pub fn host_address(&self) -> SocketAddr {
        self.host_address
    }

    pub fn last_heartbeat(&self) -> SystemTime {
        self.last_heartbeat
    }

    pub fn max_players(&self) -> u64 {
        self.max_players
    }

    pub fn mods(&self) -> &[Mod] {
        &self.mods
    }

    pub fn mods_crc(&self) -> u64 {
        self.mods_crc
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn players(&self) -> Option<&Vec<String>> {
        self.players.as_ref()
    }

    pub fn require_user_verification(&self) -> Option<bool> {
        self.require_user_verification
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub fn tags(&self) -> Option<&Vec<String>> {
        self.tags.as_ref()
    }

    /// As [`FactorioServerStatus`], if duration since is looks wrong it errored.
    pub fn as_status(&self) -> FactorioServerStatus {
        FactorioServerStatus {
            name: self.name().to_string(),
            host_address: self.host_address(),
            last_heartbeat: self.last_heartbeat(),
            now: SystemTime::now(),
            game_id: self.game_id(),
            duration_since_last_heartbeat: self.duration_since_last_heartbeat().unwrap_or_default(),
            server_id: self.server_id.clone(),
        }
    }
}

impl ServerDescription {
    pub async fn from_game_server(
        game_server: GameServer,
        token: &str,
        username: &str,
    ) -> Result<Self, ServerDescriptionError> {
        if game_server.game() != Game::Factorio {
            return Err(ServerDescriptionError::NotFactorio);
        }
        if let Some(server) = game_server.game_address() {
            Self::new(server, token, Some(game_server.id())).await
        } else if let Some(name) = game_server.server_name() {
            let short = ServerDescriptionShort::find_server_by_name(name, token, username).await?;
            if let Some(short_description) = short {
                Self::new(
                    short_description.host_address(),
                    token,
                    Some(game_server.id()),
                )
                .await
            } else {
                let err = ServerDescriptionError::ServerNotFoundWithName(name.clone());
                tracing::error!("{}", err);
                Err(err)
            }
        } else {
            Err(ServerDescriptionError::NoHostAddressOrServerName)
        }
    }
    #[instrument]
    pub async fn new(
        host_address: SocketAddr,
        token: &str,
        server_id: Option<String>,
    ) -> Result<Self, ServerDescriptionError> {
        let client = Client::new();
        let request = client
            .get(GET_GAME_DETAILS_URL.to_string() + host_address.to_string().as_str())
            .header("token", token);
        let sever_description_response = request
            .send()
            .await
            .map_err(ServerDescriptionError::RequestError)?;
        if !sever_description_response.status().is_success() {
            let api_error: MatchMakingApiError = sever_description_response
                .json()
                .await
                .map_err(ServerDescriptionError::RequestError)?;
            return Err(ServerDescriptionError::MatchMakingApiError(api_error));
        }
        let mut server_descriptions: ServerDescription = sever_description_response
            .json()
            .await
            .map_err(ServerDescriptionError::RequestError)?;
        if let Some(sid) = server_id {
            server_descriptions.server_id = sid
        }
        Ok(server_descriptions)
    }

    pub async fn update(&mut self, token: &str) -> Result<&mut Self, ServerDescriptionError> {
        let new = Self::new(self.host_address(), token, Some(self.server_id.clone())).await?;
        let old = self.clone();
        if new.game_id() != old.game_id() {
            eprintln!("Game id diff: {} -> {}", old.game_id(), new.game_id());
        }
        *self = new;
        Ok(self)
    }

    pub fn duration_since_last_heartbeat(&self) -> Result<Duration, ServerDescriptionError> {
        SystemTime::now()
            .duration_since(self.last_heartbeat())
            .map_err(ServerDescriptionError::SystemTimeError)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ApplicationVersion {
    build_mode: String,
    build_version: u64,
    game_version: String,
    platform: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Mod {
    name: String,
    version: String,
}

/// Json object recieved from [`GET_GAME_DETAILS_URL`]
///
/// ### Status 401 Error
///
/// - [Missing username][MatchMakingApiError::MissingUsername]
/// - [User not found][MatchMakingApiError::MissingUsername]
/// - [Token doesn't match][MatchMakingApiError::MissingUsername]
/// - [Missing token][MatchMakingApiError::MissingToken]
///
/// ### Shape of response
///
/// ```json
/// {
///     "game_id": 22364809,
///     "name": "Ozy_Viking: RCON2.0 Development",
///     "description": "https://github.com/JDPlays-Madhouse/RCON2.0",
///     "max_players": 0,
///     "application_version": {
///         "game_version": "2.0.66",
///         "build_version": 83840,
///         "build_mode": "headless",
///         "platform": "linux64"
///     },
///     "game_time_elapsed": 10286,
///     "has_password": true,
///     "server_id": "FHJKASLHJDKSALHJKDLH....",
///     "tags": [
///         "Dev",
///         "RCON"
///     ],
///     "host_address": "192.28.289.189:61005",
///     "headless_server": true,
///     "has_mods": true,
///     "mod_count": 36
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ServerDescriptionShort {
    game_id: u64,
    name: String,
    description: String,
    max_players: u64,
    application_version: ApplicationVersion,
    #[serde(deserialize_with = "de_int_or_string")]
    game_time_elapsed: u64,
    has_password: bool,
    server_id: String,
    tags: Option<Vec<String>>,
    host_address: SocketAddr,
    headless_server: Option<bool>,
    has_mods: bool,
    mod_count: u64,
}

impl ServerDescriptionShort {
    pub fn game_id(&self) -> u64 {
        self.game_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn application_version(&self) -> &ApplicationVersion {
        &self.application_version
    }

    pub fn max_players(&self) -> u64 {
        self.max_players
    }

    pub fn game_time_elapsed(&self) -> u64 {
        self.game_time_elapsed
    }

    pub fn has_password(&self) -> bool {
        self.has_password
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub fn host_address(&self) -> SocketAddr {
        self.host_address
    }

    pub fn headless_server(&self) -> bool {
        self.headless_server.unwrap_or(false)
    }

    pub fn mod_count(&self) -> u64 {
        self.mod_count
    }

    pub fn tags(&self) -> Option<&Vec<String>> {
        self.tags.as_ref()
    }
    #[instrument]
    /// Return all factorio servers available using the [`GET_ALL_GAMES_URL`] endpoint.
    pub async fn get_all(username: &str, token: &str) -> Result<Vec<Self>, ServerDescriptionError> {
        let client = Client::new();
        let request = client
            .get(GET_ALL_GAMES_URL)
            .query(&[("username", username), ("token", token)]);
        let sever_descriptions_response = request
            .send()
            .await
            .map_err(ServerDescriptionError::RequestError)?;
        if !sever_descriptions_response.status().is_success() {
            let api_error: MatchMakingApiError = sever_descriptions_response
                .json()
                .await
                .map_err(ServerDescriptionError::RequestError)?;
            tracing::error!("{}", api_error);
            return Err(ServerDescriptionError::MatchMakingApiError(api_error));
        }

        sever_descriptions_response
            .json()
            .await
            .map_err(ServerDescriptionError::RequestError)
    }

    #[instrument]
    pub async fn find_server_by_name(
        username: &str,
        token: &str,
        server_name: &str,
    ) -> Result<Option<Self>, ServerDescriptionError> {
        let server_descriptions = Self::get_all(username, token).await?;

        Ok(server_descriptions
            .iter()
            .find(|s| s.name() == server_name)
            .cloned())
    }
}

impl super::GameStatus for ServerDescription {
    /// Will panic if SystemTime throws a fit.
    fn game_server_status(&self) -> super::GameServerStatus {
        super::GameServerStatus::Factorio(self.as_status())
    }

    fn time_since_last_heartbeat(&self) -> Result<std::time::Duration, SystemTimeError> {
        SystemTime::now().duration_since(self.last_heartbeat())
    }
}

#[derive(thiserror::Error, miette::Diagnostic, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "message")]
pub enum MatchMakingApiError {
    #[error("Username required")]
    #[serde(rename = "Missing username.")]
    MissingUsername,
    #[error("Username not found")]
    #[serde(rename = "User not found.")]
    UserNotFound,
    #[error("Invalid token for user")]
    #[serde(rename = "Token doesn't match.")]
    InvalidToken,
    #[error("Missing token.")]
    #[serde(rename = "Missing token.")]
    MissingToken,
    #[error("Server is unreachable")]
    #[help("Use 'ServerDesciptionShort::find_server_by_name' this will provide the host_address that factorio uses.")]
    #[serde(rename = "no game for given host_address")]
    ServerUnreachable,
}

#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum ServerDescriptionError {
    #[error("Match Making API: {0}")]
    MatchMakingApiError(#[from] MatchMakingApiError),
    #[error("API error: {0:?}")]
    RequestError(reqwest::Error),
    #[error("System time error: {0}")]
    SystemTimeError(SystemTimeError),
    #[error("Not factorio. Tried calling for the wrong game.")]
    NotFactorio,
    #[error("Server with name {0} not found")]
    ServerNotFoundWithName(String),
    #[error("No game_address or server_name provided in config.")]
    NoHostAddressOrServerName,
    #[error("Missing both the username and factorio token")]
    MissingUsernameAndToken,
}

fn de_int_or_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use std::fmt;
    use std::str::FromStr;
    struct IntOrString(PhantomData<fn() -> u64>);
    impl<'de> Visitor<'de> for IntOrString
    // where
    //     T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("int or string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            u64::from_str(value).map_err(Error::custom)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
    }
    deserializer.deserialize_any(IntOrString(PhantomData))
}

fn de_time<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
where
    D: Deserializer<'de>,
{
    use std::fmt;
    struct FloatToSystemTime(PhantomData<fn() -> SystemTime>);
    impl<'de> Visitor<'de> for FloatToSystemTime
    // where
    //     T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = SystemTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("float")
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(UNIX_EPOCH + Duration::from_secs_f64(v))
        }
    }
    deserializer.deserialize_any(FloatToSystemTime(PhantomData))
}
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    #[test]
    fn checking() {
        assert_eq!(u64::from_str("5"), Ok(5))
    }
}
