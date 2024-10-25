mod command;
mod integration;
mod logging;
mod rcon2;
mod settings;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use integration::{PlatformAuthenticate, TwitchApiConnection};
use logging::{
    fetch_all_logs, log, log_to_channel, subscribe_logging_channel, unsubscribe_logging_channel,
    LogLevel, Logger,
};
use settings::Settings;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(unused_variables)]
pub async fn run() {
    let settings = Settings::new();

    let logger = Arc::new(Mutex::new(Logger::default()));
    let config = settings.config();
    let log_level = if config.get_bool("debug").unwrap() {
        LogLevel::Debug
    } else {
        settings
            .config()
            .get_string("min_log_level")
            .context("Fetching loglevel from config")
            .unwrap()
            .try_into()
            .context("Log level Conversion")
            .unwrap_or(LogLevel::default())
    };
    logger.lock().unwrap().set_min_level(log_level);
    let twitch_username = config.get_string("auth.twitch.username").unwrap();

    let twitch_client_id = config.get_string("auth.twitch.client_id").unwrap();
    let twitch_client_secret = config.get_string("auth.twitch.client_secret").unwrap();
    let twitch_redirect_url = config.get_string("auth.twitch.redirect_url").unwrap();
    let mut twitch_integration = TwitchApiConnection::new(
        twitch_username,
        twitch_client_id,
        twitch_client_secret,
        twitch_redirect_url,
        Arc::clone(&logger),
    );
    let _ = twitch_integration.authenticate().await;

    // todo!("testing");

    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(logger);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            subscribe_logging_channel,
            unsubscribe_logging_channel,
            log,
            log_to_channel,
            fetch_all_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
