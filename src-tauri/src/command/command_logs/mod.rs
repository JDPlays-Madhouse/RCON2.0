pub mod command_log;
pub use command_log::CommandLog;
mod command_logs;
pub use command_logs::*;

#[tauri::command]
pub async fn resend_event(command_log: CommandLog) -> Result<(), String> {
    todo!();
}
