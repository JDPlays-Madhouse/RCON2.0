//! Extra game features

use std::time::{Duration, SystemTimeError};

use serde::Serialize;

use crate::{
    game::{
        factorio::{ServerDescription, ServerDescriptionError},
        settings::GameSettings,
    },
    servers::{Game, GameServer},
};

pub mod factorio;
pub mod monitor;
pub mod settings;

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(tag = "game", content = "status")]
pub enum GameServerStatus {
    Factorio(factorio::FactorioServerStatus),
    NoGame,
}

impl GameServerStatus {
    pub async fn get_status(
        server: GameServer,
        settings: &GameSettings,
    ) -> Result<GameServerStatus, GameStatusError> {
        if server.game_address().is_none() & server.server_name().is_none() {
            return Ok(GameServerStatus::NoGame);
        }
        match server.game() {
            Game::Factorio => {
                let factorio_settings = settings.factorio();
                if factorio_settings.token().is_none() | factorio_settings.username().is_none() {
                    return Err(GameStatusError::FactorioEndpointError(
                        ServerDescriptionError::MissingUsernameAndToken,
                    ));
                }
                ServerDescription::from_game_server(
                    server,
                    factorio_settings.token().unwrap().as_str(),
                    factorio_settings.username().unwrap().as_str(),
                )
                .await
                .map(|desc| desc.game_server_status())
                .map_err(GameStatusError::FactorioEndpointError)
            }
        }
    }
}

pub trait GameStatus {
    /// Current Status of server.
    fn game_server_status(&self) -> GameServerStatus;
    /// Time since last heartbeat.
    fn time_since_last_heartbeat(&self) -> Result<Duration, SystemTimeError>;
}

#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum GameStatusError {
    #[error(transparent)]
    FactorioEndpointError(#[from] ServerDescriptionError),
}

pub fn status_emittor() {}

#[tauri::command]
pub async fn latest_game_server_status(server: GameServer) -> Result<GameServerStatus, String> {
    let settings = GameSettings::new();
    GameServerStatus::get_status(server, &settings)
        .await
        .map_err(|e| e.to_string())
}
