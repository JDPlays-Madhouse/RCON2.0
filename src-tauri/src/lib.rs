mod command;
mod integration;
mod logging;
mod rcon2;
mod settings;
use std::sync::Arc;

use anyhow::Context;
use integration::{PlatformAuthenticate, TwitchApiConnection};
use logging::{
    fetch_all_logs, log, log_to_channel, subscribe_logging_channel, unsubscribe_logging_channel,
    LogLevel, Logger,
};
use settings::Settings;
use tauri::async_runtime::Mutex;
use tauri::Manager;
use tracing::info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(unused_variables)]
pub async fn run() {
    let settings = Settings::new();
    // let _ = install_utils();

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
    let logger: Logger = Logger::new(&settings.config_folder, log_level);
    use log;
    use tracing_log::LogTracer;

    LogTracer::init();
    // use tauri_plugin_log::;
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;

    let logfile =
        RollingFileAppender::new(Rotation::DAILY, logger.log_folder.clone(), "rcon2.0.log");

    let (non_blocking_std_out, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let (non_blocking_logfile, _guard) = tracing_appender::non_blocking(logfile);

    let logfile_layer = fmt::layer()
        .with_writer(non_blocking_logfile)
        .with_ansi(false)
        .with_thread_ids(true);

    let stdout_layer = fmt::layer()
        .with_writer(non_blocking_std_out)
        .with_thread_ids(true);

    tracing_subscriber::registry()
        .with(logfile_layer)
        .with(stdout_layer)
        .init();

    info!("Log Established");
    let twitch_username = config.get_string("auth.twitch.username").unwrap();

    let twitch_client_id = config.get_string("auth.twitch.client_id").unwrap();
    let twitch_client_secret = config.get_string("auth.twitch.client_secret").unwrap();
    let twitch_redirect_url = config.get_string("auth.twitch.redirect_url").unwrap();
    let mut twitch_integration = TwitchApiConnection::new(
        twitch_username,
        twitch_client_id,
        twitch_client_secret,
        twitch_redirect_url,
    )
    .default_scopes();
    let _ = twitch_integration.check_token().await;
    twitch_integration.new_websocket().await;
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
        .invoke_handler(tauri::generate_handler![
            subscribe_logging_channel,
            unsubscribe_logging_channel,
            log,
            log_to_channel,
            fetch_all_logs,
            // get_channel_point_rewards
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Setup dotenv, tracing and error reporting with eyre
// pub fn install_utils() -> eyre::Result<()> {
//     let _ = dotenvy::dotenv(); //ignore error
//     install_tracing();
//     // install_eyre()?;
//     Ok(())
// }

// /// Install eyre and setup a panic hook
// fn install_eyre() -> eyre::Result<()> {
//     let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();
//
//     eyre_hook.install()?;
//
//     std::panic::set_hook(Box::new(move |pi| {
//         tracing::error!("{}", panic_hook.panic_report(pi));
//     }));
//     Ok(())
// }
// Install tracing with a specialized filter
// fn install_tracing() {
//     use tracing_error::ErrorLayer;
//     use tracing_subscriber::prelude::*;
//     use tracing_subscriber::{fmt, EnvFilter};
//
//     let fmt_layer = fmt::layer()
//         .with_file(true)
//         .with_line_number(true)
//         .with_target(true);
//     #[rustfmt::skip]
//     let filter_layer = EnvFilter::try_from_default_env()
//         .or_else(|_| EnvFilter::try_new("info"))
//         .map(|f| {
//             // common filters which can be very verbose
//             f.add_directive("hyper=error".parse().expect("could not make directive"))
//                 .add_directive("h2=error".parse().expect("could not make directive"))
//                 .add_directive("rustls=error".parse().expect("could not make directive"))
//                 .add_directive("tungstenite=error".parse().expect("could not make directive"))
//                 .add_directive("retainer=info".parse().expect("could not make directive"))
//                 .add_directive("want=info".parse().expect("could not make directive"))
//                 .add_directive("reqwest=info".parse().expect("could not make directive"))
//                 .add_directive("mio=info".parse().expect("could not make directive"))
//             //.add_directive("tower_http=error".parse().unwrap())
//         })
//         .expect("could not make filter layer");
//
//     tracing_subscriber::registry()
//         .with(filter_layer)
//         .with(fmt_layer)
//         .with(ErrorLayer::default())
//         .init();
// }
