mod command;
mod integration;
mod rcon2;
mod settings;
use std::{thread, time};

use integration::{IntegrationCommand, IntegrationControl};

use crate::integration::TwitchApiConnection;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _settings = settings::Settings::new();
    let mut twitch_intgration = TwitchApiConnection::new("", "", "");
    let twitch_command_tx = twitch_intgration.command_get_tx();
    let _ = twitch_intgration.start_thread();
    for i in [
        IntegrationCommand::Continue,
        IntegrationCommand::Pause,
        IntegrationCommand::Continue,
        IntegrationCommand::Stop,
    ] {
        let _ = twitch_command_tx.send(i);
        thread::sleep(time::Duration::from_millis(500));
    }
    todo!("testing");
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
