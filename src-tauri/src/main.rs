// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// Prevents CLI from working on windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    rcon2_lib::run().await;
}
