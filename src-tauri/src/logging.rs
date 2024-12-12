use crate::settings::ConfigValue;
use anyhow::{bail, Result};
use config::ValueKind;
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::Arc,
};
use tauri::ipc::Channel;
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use tracing::error;
use tracing::instrument;
use tracing::{debug, info, Subscriber};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;
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

static FRONTEND_CHANNELS: LazyLock<Mutex<HashMap<String, Channel<Log>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static LOGS: LazyLock<Arc<Mutex<Vec<Log>>>> = LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

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
impl From<&tracing::Level> for LogLevel {
    fn from(value: &tracing::Level) -> Self {
        use tracing::Level;
        match *value {
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
pub struct Logger {}

#[allow(dead_code)]
impl Logger {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn subscribe_to_channel(channel: Channel<Log>) -> String {
        let uuid: String = Uuid::new_v4().into();
        FRONTEND_CHANNELS.lock().await.insert(uuid.clone(), channel);
        uuid
    }

    pub async fn unsubscribe_channel(uuid: String) -> Option<Channel<Log>> {
        FRONTEND_CHANNELS.lock().await.remove(&uuid)
    }

    async fn add_log(log: Log) {
        if !Logger::log_exists(&log.uuid).await {
            LOGS.lock().await.push(log.clone());
        }
    }

    async fn find_log(uuid: String) -> Option<Log> {
        LOGS.lock().await.iter().find(|l| l.uuid == uuid).cloned()
    }

    async fn log_exists(uuid: &str) -> bool {
        LOGS.lock().await.iter().any(|l| l.uuid == uuid)
    }

    pub async fn log_to_channel(uuid: String, level: LogLevel, target: String, message: String) {
        let log = Log::new(level, target, message);
        Logger::add_log(log.clone()).await;

        let _ = match FRONTEND_CHANNELS.lock().await.get(&uuid) {
            Some(channel) => channel.send(log),
            None => Ok(()),
        };
    }
    pub fn log(level: LogLevel, target: String, message: String) {
        let log = Log::new(level, target, message);
        tauri::async_runtime::spawn(async {
            Logger::add_log(log.clone()).await;
            Logger::broadcast(log).await;
        });
    }

    pub async fn broadcast(log: Log) {
        for channel in FRONTEND_CHANNELS.lock().await.values() {
            let _ = channel.send(log.clone());
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for Logger
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let meta = event.metadata();
        let mut visitor = FrontendVisitor::new();
        let visitor_ref: &mut dyn tracing::field::Visit = &mut visitor;
        event.record(visitor_ref);

        let target: String = visitor
            .map
            .get("target")
            .unwrap_or(&meta.target().to_string())
            .clone();
        let level: LogLevel = meta.level().into();
        let message = visitor
            .map
            .get("message")
            .map_or(format!("{:?}", visitor.map).to_string(), |v| v.clone())
            .to_owned();

        match visitor.map.get("channel") {
            Some(channel) => {
                let id = channel.clone();
                tauri::async_runtime::spawn(async move {
                    Logger::log_to_channel(id, level, target, message).await;
                });
            }
            None => Logger::log(level, target, message),
        }
    }
}

#[derive(Debug, Clone)]
struct FrontendVisitor {
    map: HashMap<String, String>,
}

impl FrontendVisitor {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl tracing::field::Visit for FrontendVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.map.insert(field.to_string(), format!("{:?}", value));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.map.insert(field.to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.map.insert(field.to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.map.insert(field.to_string(), value.to_string());
    }

    fn record_i128(&mut self, field: &tracing::field::Field, value: i128) {
        self.map.insert(field.to_string(), value.to_string());
    }

    fn record_u128(&mut self, field: &tracing::field::Field, value: u128) {
        self.map.insert(field.to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.map.insert(field.to_string(), value.to_string());
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.map.insert(field.to_string(), value.to_string());
    }
}

#[tauri::command]
#[instrument(level = "trace", skip(channel))]
pub async fn subscribe_logging_channel(channel: Channel<Log>) -> Result<String, String> {
    let uuid = Logger::subscribe_to_channel(channel.clone()).await;

    debug!("Subscribed to channel: {}", uuid.clone());
    Ok(uuid)
}

#[tauri::command]
#[instrument(level = "trace")]
pub async fn fetch_all_logs() -> Result<Vec<Log>, String> {
    Ok(LOGS.lock().await.clone())
}

#[tauri::command]
#[instrument(level = "trace")]
pub async fn unsubscribe_logging_channel(uuid: String) -> Result<(), String> {
    let unsub = Logger::unsubscribe_channel(uuid.clone()).await;

    match unsub {
        Some(_channel) => {
            debug!("Unsubscribed from channel: {}", uuid.clone());
            Ok(())
        }
        None => Err("No channel under that id.".into()),
    }
}

#[tauri::command]
#[instrument(level = "trace")]
pub fn log(level: LogLevel, target: String, message: String) {
    use tracing::{debug, error, info, trace, warn};
    use LogLevel::*;
    match level {
        Trace => trace!(target = target, message),
        Debug => debug!(target = target, message),
        Info => info!(target = target, message),
        Warning => warn!(target = target, message),
        Error => error!(target = target, message),
    }
}

#[tauri::command]
#[instrument(level = "trace")]
pub fn log_to_channel(uuid: String, level: LogLevel, target: String, message: String) {
    use tracing::{debug, error, info, trace, warn};
    use LogLevel::*;
    match level {
        Trace => trace!(target = target, channel = uuid, message),
        Debug => debug!(target = target, channel = uuid, message),
        Info => info!(target = target, channel = uuid, message),
        Warning => warn!(target = target, channel = uuid, message),
        Error => error!(target = target, channel = uuid, message),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn compare_log_levels() {
        assert!(LogLevel::Debug >= LogLevel::Trace);
        assert!(LogLevel::Debug >= LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::default() < LogLevel::Warning);
    }
}
