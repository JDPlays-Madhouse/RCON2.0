use std::{collections::HashMap, fmt::Debug, sync::Mutex};
use time::{format_description::well_known::Iso8601, OffsetDateTime};

use serde::{Deserialize, Serialize};
use tauri::{ipc::Channel, State};
use uuid::Uuid;

#[derive(Debug, Serialize, Default, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Trace = 5,
    Debug = 10,
    #[default]
    Info = 20,
    Warning = 30,
    Error = 40,
    Critical = 50,
}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub uuid: String,
    pub time: String,
    pub level: LogLevel,
    pub location: String,
    pub message: String,
}

impl Log {
    fn new(level: LogLevel, location: String, message: String) -> Self {
        let time_date = match OffsetDateTime::now_local() {
            Ok(td) => td,
            Err(_) => OffsetDateTime::now_utc(),
        };
        let time = time_date.format(&Iso8601::DEFAULT).unwrap();
        let uuid = Uuid::new_v4().to_string();
        Self {
            uuid,
            time,
            level,
            location,
            message,
        }
    }
}
#[allow(dead_code)]
#[derive(Default)]
pub struct Logger {
    pub channels: Mutex<HashMap<String, Channel<Log>>>,
    pub logs: Vec<Log>,
    pub min_display_loglevel: LogLevel,
}

impl Debug for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Logger")
            .field("channels", &self.channels.lock().unwrap().keys())
            .field("logs", &self.logs)
            .field("min_display_loglevel", &self.min_display_loglevel)
            .finish()
    }
}

#[allow(dead_code)]
impl Logger {
    pub fn subscribe_to_channel(&mut self, channel: Channel<Log>) -> String {
        let uuid: String = Uuid::new_v4().into();
        self.channels.lock().unwrap().insert(uuid.clone(), channel);
        uuid
    }
    pub fn unsubscribe_channel(&mut self, uuid: String) -> Option<Channel<Log>> {
        self.channels.lock().unwrap().remove(&uuid)
    }
    pub fn set_max_level(&mut self, log_level: LogLevel) {
        self.min_display_loglevel = log_level;
        dbg!(self);
    }
    pub fn log(&mut self, level: LogLevel, location: String, message: String) {
        let log = Log::new(level, location, message);
        self.add_log(log.clone());
        self.broadcast(log)
    }

    fn log_level_high_enough(&self, log: &Log) -> bool {
        log.level >= self.min_display_loglevel
    }
    fn add_log(&mut self, log: Log) {
        if !self.log_exists(&log.uuid) {
            self.logs.push(log);
        }
    }

    fn find_log(&self, uuid: String) -> Option<Log> {
        self.logs.iter().find(|l| l.uuid == uuid).cloned()
    }
    fn log_exists(&self, uuid: &str) -> bool {
        self.logs.iter().any(|l| l.uuid == uuid)
    }
    pub fn log_to_channel(
        &mut self,
        uuid: String,
        level: LogLevel,
        location: String,
        message: String,
    ) {
        let log = Log::new(level, location, message);
        self.add_log(log.clone());
        if self.log_level_high_enough(&log) {
            // dbg!(&self);
            // dbg!(&log);
            let _ = match self.channels.lock().unwrap().get(&uuid) {
                Some(channel) => channel.send(log),
                None => Ok(()),
            };
        }
    }

    pub fn broadcast(&self, log: Log) {
        if self.log_level_high_enough(&log) {
            for channel in self.channels.lock().unwrap().values() {
                let _ = channel.send(log.clone());
            }
        }
    }
}

#[tauri::command]
pub fn subscribe_logging_channel(logger: State<Mutex<Logger>>, channel: Channel<Log>) -> String {
    let uuid = logger.lock().unwrap().subscribe_to_channel(channel.clone());

    logger.lock().unwrap().log_to_channel(
        uuid.clone(),
        LogLevel::Debug,
        "Logger".into(),
        format!("Subscribed to channel: {}", uuid.clone()),
    );
    uuid
}

#[tauri::command]
pub fn fetch_all_logs(logger_: State<Mutex<Logger>>) -> Vec<Log> {
    let logger = logger_.lock().unwrap();
    logger
        .logs
        .clone()
        .iter()
        .filter(|l| logger.min_display_loglevel <= l.level)
        .cloned()
        .collect::<Vec<Log>>()
}

#[tauri::command]
pub fn unsubscribe_logging_channel(
    logger: State<Mutex<Logger>>,
    uuid: String,
) -> Result<(), String> {
    let unsub = logger.lock().unwrap().unsubscribe_channel(uuid.clone());

    match unsub {
        Some(channel) => {
            let log = Log::new(
                LogLevel::Debug,
                "Logger".into(),
                format!("Unsubscribed from channel: {}", uuid.clone()),
            );
            if logger.lock().unwrap().log_level_high_enough(&log) {
                let _ = channel.send(log);
            }
            Ok(())
        }
        None => Err("No channel under that id.".into()),
    }
}

#[tauri::command]
pub fn log(logger: State<Mutex<Logger>>, level: LogLevel, location: String, message: String) {
    logger.lock().unwrap().log(level, location, message);
}

#[tauri::command]
pub fn log_to_channel(
    logger: State<Mutex<Logger>>,
    uuid: String,
    level: LogLevel,
    location: String,
    message: String,
) {
    logger
        .lock()
        .unwrap()
        .log_to_channel(uuid, level, location, message);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn compare_log_levels() {
        assert!(LogLevel::Debug >= LogLevel::Trace);
        assert!(LogLevel::Debug >= LogLevel::Debug);
        assert!(!(LogLevel::Debug >= LogLevel::Info));
        assert!(LogLevel::default() < LogLevel::Warning);
    }
}
