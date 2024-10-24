mod command;
mod integration;
mod logging;
mod rcon2;
mod settings;
use std::sync::Mutex;

use logging::{
    fetch_all_logs, log, log_to_channel, subscribe_logging_channel, unsubscribe_logging_channel,
    LogLevel, Logger,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let logger = Mutex::new(Logger::default());
    logger.lock().unwrap().set_max_level(LogLevel::Info);
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
