pub mod command_log;
use std::sync::Arc;

pub use command_log::CommandLog;
mod command_logs;
pub use command_logs::*;
use tauri::State;

use crate::{
    servers::{GameServerConnected, CONNECTIONS},
    TwitchApiConnection,
};

#[tauri::command]
pub async fn resend_event(
    command_log: CommandLog,
    twitch_mutex: State<'_, Arc<futures::lock::Mutex<TwitchApiConnection>>>,
) -> Result<(), String> {
    // COMMAND_LOGS.lock().await.add_log(command_log.repeat_log()); -- Added when recieved by runner
    let event = command_log.event().clone();
    tracing::info!("Resending Event: {:?}", &event);
    let mut twitch = twitch_mutex.lock().await;
    match twitch.send_event_to_runner(event).await {
        Ok(_) => {
            tracing::info!("Successfully Sent event");
        }
        Err(_) => {
            tracing::error!("Failed to send event, try using 'resend command'")
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn resend_command(command_log: CommandLog) -> Result<(), String> {
    COMMAND_LOGS.lock().await.add_log(command_log.repeat_log());
    let mut command = command_log.command().clone();
    tracing::info!("Resending Command: {:?}", command.name);

    let mut connections = CONNECTIONS.lock().await;
    let connection: &mut GameServerConnected =
        match connections.get_mut(command_log.trigger().server()) {
            Some(c) => c,
            None => return Err("Server not connected to.".to_string()),
        };
    match connection
        .send_command(command.tx_string(command_log.message(), &command_log.username()))
        .await
    {
        Ok(_r) => {
            tracing::info!("Sent command: {:?}", command.name);
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}
