//! # RCON2
//!
//! ## Minimum Viable Product
//!
//! - [x] Authenticate with twitch.
//! - [x] Connect to twitch websocket.
//! - [x] Handle Chat and Channel Point Reward Events.
//!     - Only the redeem is handled not the comments.
//! - [x] Connect to an Rcon Server.
//! - [x] Send commands to Rcon Server.
//! - [x] Configure through TOML commands to send to RCON server with defined triggers.
//! - [x] React to triggers and send events to RCON server.
//!
//! ## Todo
//!
//! - [x] Write commands to file.
//! - [x] React to events.
//!
//! ## Classes
#![doc = simple_mermaid::mermaid!("../../docs/mermaid/classes.mmd")]
//!
//! ## Requirements
//!
//! 1. [x] Read twitch events directly i.e. no Streamer.bot etc.
//!     1. [ ] Events including subs/bits/follows/hype trains
//!     1. [x] Channel Points
//!     1. [x] Chat messages
//! 1. [ ] Detect certain messages and parse the message for battleship. (Only 1 command)
//!     1. [ ] Example: /muppet_streamer_schedule_explosive_delivery target targetPosition
//! 1. [ ] Convert the parsed message into a valid command.
//!     1. [ ] have default values for commands so invalid data with a valid command
//!               becomes valid command with default data.
//! 1. [ ] Read from SteamLabs/streamelements Patreon (own api) and Humble
//!        notifications and donations.
//! 1. [ ] Read from YouTube for chat/subs/memberships/supers.
//! 1. [x] Have a Pause button or Api end point to pause for bio breaks.
//!
//! 1. [x] Have a RCON app/interface that takes in specific Factorio commands as well
//!        as any other games.
//! 1. [x] Rcon interface needs to take configurations for any rcon server.
//! 1. [ ] Ensure that the amount of data is below the max per tick amount.
//!
//! 1. [ ] Provide visiual feedback through an OBS overlay (website) to give feedback
//!        on things like the boom factor.
//!        ![Example of OBS overlay](./docs/Example_visual_feedback.png)
//! 1. [ ] From twitch events read hype trains and be able to respond.
//!     1. JDGOESBoom with count down, if redeamed again dead factor goes up and
//!            restart count down.
//! 1. [ ] Be able to add RCON commands, modify, delete, display (CRUD), including
//!        default values like deadliness.
//! 1. [ ] Be able to test when adding commands.
//! 1. [x] Output a log with raw output for debugging
//!     1. [x] ESPECIALLY "custom-reward-id" from twitch channel points as ill need
//!            that data for adding new points rewards through streamer.bot. Or Work
//!            out what the ID code.
//! 1. [ ] Has to support some sort of user comments in the script so i can keep
//!        track/notes on new code.
//!
//! ### Definitions
//!
//! - JD Goes Boom factorio becomes a progress bar, like a power up bar.
//! - Bar goes up from donations being bits or paypal, ALSO sub points (being sub
//!   points/2 treated as a number) or any other data that is being captured.
//! - Progress bar has a Nades, then cluster nade, then arty, then nukes for maximum
//!   effect.
//!
//! -----
//!
//! ## Integrations
//!
//! ### Twitch
//!
//! 1. Get a Client ID and Client Secret from
//!     [dev.twitch.tv/console/apps/](https://dev.twitch.tv/console/apps/).
//! 2. For the redirect url make sure they are exactly the same
//!     e.g. `http://localhost:27934/twitch/register`. The port can be changed but
//!     both the dev console and the config file need to match.
//! 3. Run the application once and the config file will generate.
//!    1. Windows: ~\AppData\roaming\RCON2.0
//!    2. Linux: ~/.config/RCON2.0
//!    3. Apple: ~/Library/Application Support/RCON2.0
//! 4. Add the credentials to auth.twitch.
//! 5. websocket_subscription are the websocket events that you want to track
//!    defined by
//!    [twitch docs](https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/).
//!    Currently implemented are listed below, if there any not listed that you
//!    want, start an issue
//!    [JDPlays-Madhouse/RCON2.0/issues](https://github.com/JDPlays-Madhouse/RCON2.0/issues).
//!    1. channel.chat.message
//!    2. channel.channel_points_custom_reward_redemption.add
//!    3. channel.channel_points_custom_reward_redemption.update
//!
//! ### YouTube
//!
//! Not yet implemented.
//!
//! ### Patreon
//!
//! Not yet implemented.
//!
//! -----
//!
//! ## Testing
//!
//! 1. Start a mock-api websocket.
//!     ```sh
//!     twitch event websocket start-server
//!     ```
//! 2. Start application.
//!     ```sh
//!     TWITCH_HELIX_URL=http://localhost:8080 TWITCH_EVENTSUB_WEBSOCKET_URL=ws://127.0.0.1:8080/ws cargo tauri dev
//!     ```
//! 3. Send fake events.
//!     
//!     ```sh
//!     twitch event trigger channel.channel_points_custom_reward_redemption.add --transport=websocket -i 1
//!     ```

pub mod cli;
pub mod command;
pub mod integration;
pub mod logging;
pub mod servers;
pub mod settings;

use std::sync::Arc;

use anyhow::Context;
use cli::handle_cli_matches;
use command::settings::ScriptSettings;
use integration::TwitchApiConnection;
use logging::{LogLevel, Logger};
use settings::Settings;
use tauri::{AppHandle, Manager};
use tauri_plugin_cli::CliExt;
use time::UtcOffset;
use tracing::{debug, error, info};
use tracing_appender::rolling::RollingFileAppender;
use tracing_appender::rolling::Rotation;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub const PROGRAM: &str = "RCON2.0";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(unused_variables, unreachable_code)]
pub async fn run() {
    let settings = Settings::new();

    let config = settings.config();
    let _script_settings = ScriptSettings::new();
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
    info!("Log level: {}", log_level);

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

    let twitch_integration = Arc::new(futures::lock::Mutex::new(TwitchApiConnection::new(
        config.get_table("auth.twitch").unwrap(),
    )));

    let config_clone = config.clone();
    let twitch_int_clone = Arc::clone(&twitch_integration);
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_cli::init())
        .setup(move |app| {
            match app.cli().matches() {
                Ok(matches) => {
                    futures::executor::block_on(handle_cli_matches(
                        matches,
                        app,
                        Arc::clone(&twitch_int_clone),
                    ));
                }
                Err(e) => {
                    println!("{}", e);
                    std::process::exit(1);
                }
            }
            let default_server = servers::default_server_from_settings(config_clone.clone());

            app.manage(Arc::new(futures::lock::Mutex::new(config_clone.clone())));
            app.manage(Arc::new(futures::lock::Mutex::new(default_server)));
            app.manage(twitch_int_clone);

            tracing_subscriber::Registry::default()
                .with(level_filter)
                .with(logger_layer)
                .with(logfile_layer)
                .with(stdout_layer)
                .with(ErrorLayer::default())
                .init();
            debug!("Log Established");
            info!("Config File: {:?}", &settings.config_filepath());
            info!("Log Folder: {:?}", &settings.log_folder);
            info!("Script Folder: {:?}", &settings.script_folder);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            command::create_command,
            command::server_trigger_commands,
            logging::fetch_all_logs,
            logging::log,
            logging::log_to_channel,
            logging::subscribe_logging_channel,
            logging::unsubscribe_logging_channel,
            restart,
            servers::check_connection,
            servers::connect_to_server,
            servers::disconnect_connection,
            servers::get_default_server,
            servers::list_game_servers,
            servers::new_server,
            servers::send_command_to_server,
            servers::set_default_server,
            servers::update_server,
            settings::set_config_array,
            settings::set_config_bool,
            settings::get_config_bool,
            settings::set_config_float,
            settings::set_config_int,
            settings::set_config_string,
            settings::set_config_uint,
            settings::update_config,
            integration::connect_to_integration,
            integration::list_of_integrations,
            integration::status::integration_status,
            integration::twitch::get_channel_point_rewards,
            integration::twitch::refresh_twitch_websocket,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn restart(app: AppHandle) {
    /// Restart the program from the frontend.
    use tauri::Manager;
    tauri::process::restart(&app.env())
}
