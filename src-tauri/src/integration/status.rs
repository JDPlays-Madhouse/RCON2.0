use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::instrument;

use super::{Api, TokenError, TwitchApiConnection};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", content = "api")]
pub enum IntegrationStatus {
    Connected {
        api: Api,
        expires_at: Option<i64>,
    },
    Connecting(Api),
    Disconnected(Api),
    Error {
        api: Api,
        error: IntegrationError,
    },
    #[default]
    Unknown,
}
impl IntegrationStatus {
    pub fn seconds_to(duration: Duration) -> i64 {
        dbg!(&duration);
        let expires_at = SystemTime::now() + duration;
        match expires_at.duration_since(UNIX_EPOCH) {
            Ok(exp) => exp.as_secs() as i64,
            Err(e) => -(e.duration().as_secs() as i64),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "error", content = "data")]
pub enum IntegrationError {
    Token(TokenError),
    NotImplemented(Api),
    #[default]
    Unknown,
}

/// TODO: Add clearer UI indication of check.
#[tauri::command]
#[instrument(level = "trace")]
pub async fn integration_status(
    api: Api,
    twitch_integration: State<'_, Arc<futures::lock::Mutex<TwitchApiConnection>>>,
) -> Result<IntegrationStatus, IntegrationError> {
    match api {
        Api::Twitch => {
            let mut twitch_integration_locked = twitch_integration.lock().await;
            twitch_integration_locked.check_status().await
        }
        _ => Err(IntegrationError::NotImplemented(api)),
    }
}
