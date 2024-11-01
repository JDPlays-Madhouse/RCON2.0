use crate::settings::{self, ConfigValue};
use anyhow::{anyhow, bail, Context, Error, Result};
use config::ValueKind;
use futures::executor;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::{ipc::Channel, State};
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use tracing::instrument::WithSubscriber;
use tracing::{info, Level};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
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
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().to_string().to_uppercase())
    }
}

impl From<tracing::Level> for LogLevel {
    fn from(value: tracing::Level) -> Self {
        use tracing::Level;
        match value {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warning,
            Level::ERROR => LogLevel::Error,
        }
    }
}
impl From<LogLevel> for tracing::Level {
    fn from(value: LogLevel) -> Self {
        use tracing::Level;
        match value {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warning => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}
impl From<log::Level> for LogLevel {
    fn from(value: log::Level) -> Self {
        use log::Level;
        match value {
            Level::Trace => LogLevel::Trace,
            Level::Debug => LogLevel::Debug,
            Level::Info => LogLevel::Info,
            Level::Warn => LogLevel::Warning,
            Level::Error => LogLevel::Error,
        }
    }
}
impl From<LogLevel> for log::Level {
    fn from(value: LogLevel) -> Self {
        use log::Level;
        match value {
            LogLevel::Trace => Level::Trace,
            LogLevel::Debug => Level::Debug,
            LogLevel::Info => Level::Info,
            LogLevel::Warning => Level::Warn,
            LogLevel::Error => Level::Error,
        }
    }
}

impl TryFrom<ConfigValue> for LogLevel {
    type Error = anyhow::Error;

    fn try_from(value: ConfigValue) -> Result<Self, anyhow::Error> {
        match value.kind {
            ValueKind::String(loglevel) => match loglevel.to_uppercase().as_str() {
                "TRACE" => Ok(LogLevel::Trace),
                "DEBUG" => Ok(LogLevel::Debug),
                "INFO" => Ok(LogLevel::Info),
                "WARNING" => Ok(LogLevel::Warning),
                "ERROR" => Ok(LogLevel::Error),
                "CRITICAL" => Ok(LogLevel::Error),
                _ => bail!("Invalid log level"),
            },
            _ => bail!("Invalid Type"),
        }
    }
}

impl TryFrom<&'static str> for LogLevel {
    type Error = anyhow::Error;

    fn try_from(value: &'static str) -> std::result::Result<Self, anyhow::Error> {
        match value.to_uppercase().as_str() {
            "TRACE" => Ok(LogLevel::Trace),
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARNING" => Ok(LogLevel::Warning),
            "ERROR" => Ok(LogLevel::Error),
            "CRITICAL" => Ok(LogLevel::Error),
            _ => bail!("Invalid log level"),
        }
    }
}
impl TryFrom<String> for LogLevel {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, anyhow::Error> {
        match value.to_uppercase().as_str() {
            "TRACE" => Ok(LogLevel::Trace),
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARNING" => Ok(LogLevel::Warning),
            "ERROR" => Ok(LogLevel::Error),
            "CRITICAL" => Ok(LogLevel::Error),
            _ => bail!("Invalid log level"),
        }
    }
}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub uuid: String,
    pub time: String,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
}

impl Log {
    fn new(level: LogLevel, target: String, message: String) -> Self {
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
            target,
            message,
        }
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}",
            self.time, self.level, self.target, self.message
        )
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Logger {
    pub channels: Arc<Mutex<HashMap<String, Channel<Log>>>>,
    pub logs: Arc<Mutex<Vec<Log>>>,
    pub min_display_loglevel: LogLevel,
    pub log_folder: PathBuf,
}

impl Debug for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Logger")
            .field("channels", &(self.channels.lock().unwrap()).keys())
            .field("logs", &self.logs)
            .field("min_display_loglevel", &self.min_display_loglevel)
            .field("log_folder", &self.log_folder)
            .finish()
    }
}

#[allow(dead_code)]
impl Logger {
    pub fn new<T: Into<PathBuf>, L: Into<LogLevel>>(
        config_folder_name: T,
        min_display_loglevel: L,
    ) -> Self {
        let log_folder = dirs::config_dir()
            .unwrap()
            .join(config_folder_name.into())
            .join("logs");
        if !log_folder.exists() {
            std::fs::create_dir_all(&log_folder);
        }
        Self {
            log_folder: log_folder.clone(),
            channels: Default::default(),
            logs: Default::default(),
            min_display_loglevel: min_display_loglevel.into(),
        }
    }

    pub fn subscribe_to_channel(&mut self, channel: Channel<Log>) -> String {
        let uuid: String = Uuid::new_v4().into();
        self.channels.lock().unwrap().insert(uuid.clone(), channel);
        uuid
    }
    pub fn unsubscribe_channel(&mut self, uuid: String) -> Option<Channel<Log>> {
        self.channels.lock().unwrap().remove(&uuid)
    }
    pub fn set_min_level(&mut self, log_level: LogLevel) {
        self.min_display_loglevel = log_level;
        // dbg!(self);
    }

    fn log_level_high_enough(&self, log: &Log) -> bool {
        log.level >= self.min_display_loglevel
    }
    fn add_log(&mut self, log: Log) {
        if !self.log_exists(&log.uuid) {
            self.logs.lock().unwrap().push(log.clone());
            self.log_to_file(log);
        }
    }

    pub fn log_to_file(&mut self, log: Log) {
        // writeln!(self.log_file, "{}", log);
    }

    fn find_log(&self, uuid: String) -> Option<Log> {
        self.logs
            .lock()
            .unwrap()
            .iter()
            .find(|l| l.uuid == uuid)
            .cloned()
    }
    fn log_exists(&self, uuid: &str) -> bool {
        self.logs.lock().unwrap().iter().any(|l| l.uuid == uuid)
    }
    pub fn log_to_channel(
        &mut self,
        uuid: String,
        level: LogLevel,
        target: String,
        message: String,
    ) {
        let log = Log::new(level, target, message);
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
        };
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let log_ = Log::new(
            record.level().into(),
            record.target().into(),
            record.args().to_string(),
        );
        self.logs.lock().unwrap().push(log_.clone());
        self.broadcast(log_);
        // record
    }

    fn flush(&self) {
        todo!("logger::flush")
    }
}
// pub fn log(&mut self, level: LogLevel, target: &str, message: &str) {
//     let log = Log::new(level, target.into(), message.into());
//     self.add_log(log.clone());
//     self.broadcast(log);
// }

#[tauri::command]
pub fn subscribe_logging_channel(
    logger_: State<'_, Arc<Mutex<Logger>>>,
    channel: Channel<Log>,
) -> String {
    let mut logger = logger_.lock().unwrap();
    let uuid = logger.subscribe_to_channel(channel.clone());

    logger.log_to_channel(
        uuid.clone(),
        LogLevel::Debug,
        "Logger".into(),
        format!("Subscribed to channel: {}", uuid.clone()),
    );
    uuid
}

#[tauri::command]
pub fn fetch_all_logs(logger_: State<'_, Arc<Mutex<Logger>>>) -> Vec<Log> {
    let logger = logger_.lock().unwrap();
    let logs = logger.logs.lock().unwrap().clone();
    logs.iter()
        .filter(|l| logger.min_display_loglevel <= l.level)
        .cloned()
        .collect::<Vec<Log>>()
}

#[tauri::command]
pub fn unsubscribe_logging_channel(
    logger_: State<'_, Arc<Mutex<Logger>>>,
    uuid: String,
) -> Result<(), String> {
    let mut logger = logger_.lock().unwrap();
    let unsub = logger.unsubscribe_channel(uuid.clone());

    match unsub {
        Some(channel) => {
            let log = Log::new(
                LogLevel::Debug,
                "Logger".into(),
                format!("Unsubscribed from channel: {}", uuid.clone()),
            );
            if logger.log_level_high_enough(&log) {
                let _ = channel.send(log);
            }
            Ok(())
        }
        None => Err("No channel under that id.".into()),
    }
}

#[tauri::command]
pub fn log(level: LogLevel, target: String, message: String) {
    log::log!(level.into(), "{} - {}", target, message)
}

#[tauri::command]
pub fn log_to_channel(
    logger: State<'_, Arc<Mutex<Logger>>>,
    uuid: String,
    level: LogLevel,
    target: String,
    message: String,
) {
    logger
        .lock()
        .unwrap()
        .log_to_channel(uuid, level, target, message);
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
