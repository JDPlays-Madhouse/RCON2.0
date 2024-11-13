use std::{
    io::{self, Write},
    process,
    sync::Arc,
};
use tauri::App;
use tauri_plugin_cli::{CliExt, Matches};
use tracing::error;

use crate::integration::{PlatformAuthenticate, TwitchApiConnection};

pub async fn handle_cli_matches(
    matches: Matches,
    app: &App,
    twitch_integration: Arc<futures::lock::Mutex<TwitchApiConnection>>,
) {
    let mut exit: bool = false;
    for (name, arg_data) in matches.args.into_iter() {
        match name.as_str() {
            "help" => {
                let mut stdout = io::stdout().lock();
                let help = format!("{}", arg_data.value.as_str().unwrap());
                stdout
                    .write_all(help.as_bytes())
                    .expect("Writing to standard out.");
                std::process::exit(0);
            }
            "version" => {
                let version = format!("RCON2.0 Version: {}", app.package_info().version);
                println!("{}", version);
                std::process::exit(0);
            }
            _ => {
                error!("Unknown cli input: {}", name);
            }
        }
    }

    if let Some(subcommands) = matches.subcommand {
        let matches = subcommands.matches;
        let mut token_buf = Vec::new();
        let _ = writeln!(token_buf, "Auth Tokens:\n");

        for (name, arg_data) in matches.args.into_iter() {
            match name.as_str() {
                "help" => {
                    let mut stdout = io::stdout().lock();
                    let help = format!("{}", arg_data.value.as_str().unwrap());
                    stdout
                        .write_all(help.as_bytes())
                        .expect("Writing to standard out.");
                    std::process::exit(0);
                }

                "twitch" => {
                    if arg_data.value.as_bool().expect("Tried to parse as boolean") {
                        exit = true;
                        if let Err(err) = twitch_integration.lock().await.authenticate().await {
                            error!("{}", err);
                            process::exit(1);
                        };
                        match &twitch_integration.lock().await.token {
                            Some(token) => {
                                let _ =
                                    writeln!(token_buf, "Twitch: {}", token.access_token.secret());
                            }
                            None => {
                                error!("Failed to get Twitch Token")
                            }
                        }
                    }
                }
                "youtube" => {
                    if arg_data.value.as_bool().expect("Tried to parse as boolean") {
                        exit = true;
                        let _ = writeln!(token_buf, "YouTube: To be Implemented");
                    }
                }
                _ => todo!("Implement"),
            }
        }
        let mut stdout = io::stdout().lock();
        let _ = stdout.write_all(&token_buf);
    }

    if exit {
        std::process::exit(0)
    }
}
