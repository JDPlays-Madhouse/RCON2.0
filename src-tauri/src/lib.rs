mod cli;
mod command;
mod integration;
mod logging;
mod servers;
mod settings;

use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use cli::handle_cli_matches;
use integration::TwitchApiConnection;
use logging::{
    fetch_all_logs, log, log_to_channel, subscribe_logging_channel, unsubscribe_logging_channel,
    LogLevel, Logger,
};
use servers::get_default_server;
use servers::set_default_server;
use servers::{list_game_servers, servers_from_settings};
use settings::Settings;
use tauri::AppHandle;
use tauri_plugin_cli::CliExt;
use time::UtcOffset;
use tracing::{debug, error, info};
use tracing_appender::rolling::RollingFileAppender;
use tracing_appender::rolling::Rotation;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;

pub const PROGRAM: &str = "RCON2.0";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(unused_variables)]
pub async fn run() {
    let settings = Settings::new();
    let config = settings.config();
    let log_level = if config.get_bool("debug").unwrap() {
        LogLevel::Debug
    } else {
        settings
            .config()
            .get_string("max_log_level")
            .context("Fetching loglevel from config")
            .unwrap()
            .try_into()
            .context("Log level Conversion")
            .unwrap_or(LogLevel::default())
    };

    let file_prefix = String::from(PROGRAM) + ".log";
    // use tauri_plugin_log::;
    let logfile =
        RollingFileAppender::new(Rotation::DAILY, settings.log_folder.clone(), file_prefix);

    let (non_blocking_std_out, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let (non_blocking_logfile, _guard) = tracing_appender::non_blocking(logfile);
    use tracing_subscriber::fmt::time::OffsetTime;

    let offset = match UtcOffset::current_local_offset() {
        Ok(tz) => tz,
        Err(e) => {
            error!("Failed to get local timezone: {:?}", e);
            UtcOffset::UTC
        }
    };
    let timer = OffsetTime::new(offset, time::format_description::well_known::Rfc3339);

    let logfile_layer = fmt::layer()
        .with_writer(non_blocking_logfile)
        .with_ansi(false)
        .with_timer(timer.clone())
        .with_thread_ids(true);

    let stdout_layer = fmt::layer()
        .with_writer(non_blocking_std_out)
        .with_timer(timer)
        .with_thread_ids(false);

    let level_filter = tracing_subscriber::filter::LevelFilter::from_level(log_level.into());

    let logger_layer: Logger = Logger::new();

    debug!("Log Established");
    match servers_from_settings(config.clone()) {
        Ok(servers) => {
            let server_names: Vec<&str> =
                servers.iter().map(|server| server.name.as_str()).collect();
            info!("Retrieved configs for servers: {:?}", server_names)
        }
        Err(e) => {
            error!("{:?}", e)
        }
    };
    let mut twitch_integration = Arc::new(futures::lock::Mutex::new(TwitchApiConnection::new(
        config.get_table("auth.twitch").unwrap(),
    )));
    let twitch_int_clone = Arc::clone(&twitch_integration);
    tokio::spawn(async move {
        match twitch_int_clone.lock().await.check_token().await {
            Ok(_) => info!("Twitch Token is valid"),
            Err(e) => info!("Twitch Token is invalid: {:?}", e),
        };

        twitch_int_clone.lock().await.new_websocket(config).await
    });
    let twitch_int_clone = Arc::clone(&twitch_integration);
    tauri::Builder::default()
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_cli::init())
        .setup(move |app| {
            match app.cli().matches() {
                Ok(matches) => {
                    futures::executor::block_on(handle_cli_matches(matches, app, twitch_int_clone));
                }
                Err(e) => {
                    println!("{}", e);

                    std::process::exit(1);
                }
            }
            // todo!();
            tracing_subscriber::Registry::default()
                .with(level_filter)
                .with(logger_layer)
                .with(logfile_layer)
                .with(stdout_layer)
                .with(ErrorLayer::default())
                .init();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            subscribe_logging_channel,
            unsubscribe_logging_channel,
            log,
            log_to_channel,
            fetch_all_logs,
            list_game_servers, // get_channel_point_rewards
            restart,
            get_default_server,
            set_default_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn restart(app: AppHandle) {
    use tauri::{process::restart, Manager};
    restart(&app.env())
}
