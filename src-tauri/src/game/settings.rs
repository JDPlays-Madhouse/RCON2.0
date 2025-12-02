use config::Config;
use serde::{Deserialize, Serialize};

use crate::settings::{DefaultValue, Settings, SettingsError};

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum GameSettingsError {
    #[error("{0}")]
    SettingsError(#[from] SettingsError),
}
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct GameSettings {
    factorio: FactorioSettings,
}

impl GameSettings {
    pub fn default_settings_str() -> Vec<DefaultValue<&'static str>> {
        vec![FactorioSettings::default_settings_str()]
            .iter()
            .flatten()
            .copied()
            .collect()
    }

    pub fn new() -> Result<Self, GameSettingsError> {
        let config = Settings::new()
            .map_err(GameSettingsError::SettingsError)?
            .config();
        Ok(Self::from_config(config))
    }

    pub fn from_config(config: Config) -> Self {
        let mut mut_self = Self::default();
        if let Ok(factorio) = config.get_table("game.factorio") {
            let username = factorio
                .get("username")
                .map(|u| u.clone().into_string().unwrap_or_default());
            let token = factorio
                .get("token")
                .map(|u| u.clone().into_string().unwrap_or_default());
            mut_self.factorio.set_token(token).set_username(username);
        }
        mut_self
    }

    pub fn optional_new() -> Option<Self> {
        let new = Self::new();
        new.ok()
    }

    /// Any setting is [`Some`]
    pub fn is_some(&self) -> bool {
        self.factorio.is_some() // | self....
    }

    #[allow(dead_code)]
    /// All settings are [`None`]
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn factorio(&self) -> &FactorioSettings {
        &self.factorio
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FactorioSettings {
    username: Option<String>,
    token: Option<String>,
}

#[allow(dead_code)]
impl FactorioSettings {
    fn new(username: Option<String>, token: Option<String>) -> Self {
        Self { username, token }
    }

    pub fn default_settings_str() -> Vec<DefaultValue<&'static str>> {
        vec![("game.factorio.username", ""), ("game.factorio.token", "")]
    }

    /// Any setting is [`Some`]
    pub fn is_some(&self) -> bool {
        self.token.is_some() | self.username.is_some()
    }

    /// All settings are [`None`]
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn username(&self) -> Option<&String> {
        self.username.as_ref()
    }

    pub fn token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    fn set_username(&mut self, username: Option<String>) -> &mut Self {
        if let Some(name) = username {
            if name.trim().is_empty() {
                self.username = None
            } else {
                self.username = Some(name)
            }
        } else {
            self.username = None
        };
        self
    }

    fn set_token(&mut self, token: Option<String>) -> &mut Self {
        if let Some(t) = token {
            if t.trim().is_empty() {
                self.token = None
            } else {
                self.token = Some(t)
            }
        } else {
            self.token = None
        };
        self
    }
}
