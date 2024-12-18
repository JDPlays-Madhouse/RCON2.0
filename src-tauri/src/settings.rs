use anyhow::{anyhow, Context, Error, Result};
use config::{
    builder::{BuilderState, DefaultState},
    ConfigBuilder, Map, Value, ValueKind,
};
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::State;
use tracing::{error, info, instrument};

use crate::PROGRAM;
use config::{Config, Environment, File, FileFormat};
use dirs::config_dir;

pub enum FileType {
    Dir,
    File,
}

#[derive(Debug)]
pub struct Settings {
    pub config_builder: ConfigBuilder<DefaultState>,
    // pub script_config_builder: ConfigBuilder<DefaultState>,
    pub config_folder: PathBuf,
    pub log_folder: PathBuf,
    pub script_folder: PathBuf,
    pub config_filename: String,
    pub config_fileformat: FileFormat,
}

impl Settings {
    pub fn new() -> Self {
        let mut mut_self = Self::default();
        let mut builder = mut_self
            .config_builder
            .clone()
            .add_source(File::new(
                mut_self.config_filepath().to_str().unwrap(),
                mut_self.config_fileformat,
            ))
            .add_source(Environment::with_prefix(PROGRAM));
        mut_self.config_builder = builder.clone();
        let config = mut_self.config();

        let log_folder = PathBuf::from(
            config
                .get_string("log_folder")
                .expect("Getting log folder path"),
        );
        let _ = mut_self.make_config_exists(log_folder.as_path(), FileType::Dir);
        mut_self.log_folder = log_folder;

        let script_folder = PathBuf::from(
            config
                .get_string("script_folder")
                .expect("Getting script folder path"),
        );
        let _ = mut_self.make_config_exists(script_folder.as_path(), FileType::Dir);
        mut_self.script_folder = script_folder;
        if config.get_table("servers").unwrap().keys().count() <= 2 {
            builder = Settings::default_loop(
                builder,
                vec![
                    ("servers.default", "example"),
                    ("servers.example.address", "127.0.0.1"),
                    ("servers.example.game", "factorio"),
                    ("servers.example.password", "totally_secure_password"),
                ],
            );
            builder = Settings::default_loop(builder, vec![("servers.example.port", 4312)]);
            let _ = mut_self.set_config("servers.default", "example");

            mut_self.config_builder = builder.clone();
        }

        let _ = mut_self.write();
        mut_self
    }

    pub fn new_from_config(config: Config) -> Self {
        let mut settings = Self::new();
        settings.config_builder = settings.config_builder.add_source(config);
        settings
    }

    pub fn config_filepath(&self) -> PathBuf {
        let config_folder = &self.config_folder;
        let config_filename = Path::new(&self.config_filename);
        config_folder.join(config_filename)
    }

    pub fn filename(name: &'static str, file_format: FileFormat) -> String {
        let format = match file_format {
            FileFormat::Toml => ".toml",
            FileFormat::Yaml => ".yaml",
            FileFormat::Json => ".json",
            _ => panic!("Config filetype not supported."),
        };

        name.to_owned() + format
    }

    pub fn set_config<T: Into<ValueKind>>(&mut self, key: &str, value: T) -> Result<()> {
        let builder = self
            .config_builder
            .clone()
            .set_override(key, value)
            .context("Setting value in config")?;
        self.config_builder = builder;
        self.write().context("Writing config")?;
        Ok(())
    }

    pub fn get_config(&self, key: &str) -> Option<ConfigValue> {
        let setting = match self.config().get::<Value>(key) {
            Ok(value) => Some(ConfigValue::from(value)),
            Err(e) => {
                error!("Config not available: {:?}", e);
                None
            }
        };
        setting
    }
    pub fn remove_config(mut self, key: &str) -> Self {
        self.set_config(key, ValueKind::Nil)
            .expect("Setting Config");
        self.write().expect("writing to file");
        Self::new()
    }

    pub fn file_exists(path: &Path) -> bool {
        path.exists()
    }

    pub fn current_config() -> Config {
        Self::new().config()
    }

    pub fn config(&self) -> Config {
        self.config_builder
            .build_cloned()
            .context("Build config")
            .unwrap()
    }

    pub fn make_config_exists(&self, path: &Path, filetype: FileType) -> Result<FileType, Error> {
        match (Settings::file_exists(path), filetype) {
            (true, FileType::Dir) => {
                if path.is_dir() {
                    Ok(FileType::Dir)
                } else if path.is_file() {
                    Err(anyhow!(
                        "{} is a file when it needs to be a directory",
                        path.display()
                    ))
                } else {
                    Err(anyhow!("reached an unhandled path in Settings::make_exist"))
                }
            }
            (true, FileType::File) => {
                if path.is_file() {
                    Ok(FileType::File)
                } else if path.is_dir() {
                    Err(anyhow!(
                        "{} is a directory when it needs to be a file",
                        path.display()
                    ))
                } else {
                    Err(anyhow!("reached an unhandled path in Settings::make_exist"))
                }
            }
            (false, FileType::Dir) => match std::fs::create_dir_all(path) {
                Err(error) => Err(anyhow!(error)),
                Ok(_) => Ok(FileType::Dir),
            },
            (false, FileType::File) => {
                let _ = self.write();
                Ok(FileType::File)
            } // _ => Err(anyhow!("reached an unhandled path in Settings::make_exist")),
        }
    }

    pub fn write(&self) -> std::io::Result<()> {
        let serializable: Map<String, ConfigValue> = self
            .config()
            .try_deserialize::<Map<String, Value>>()
            .context("deserializing config")
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), ConfigValue::from(v)))
            .collect();
        let toml_out = toml::to_string_pretty(&serializable)
            .context("Convert to Toml")
            .unwrap();
        std::fs::write(self.config_filepath(), toml_out)
    }
}

#[derive(Debug, Clone)]
pub struct ConfigValue {
    pub kind: ValueKind,
}

impl From<&Value> for ConfigValue {
    fn from(value: &Value) -> Self {
        Self {
            kind: value.kind.clone(),
        }
    }
}
impl From<Value> for ConfigValue {
    fn from(value: Value) -> Self {
        Self {
            kind: value.kind.clone(),
        }
    }
}
impl ConfigValue {
    pub fn value<T: TryFrom<ConfigValue>>(&self) -> Result<T, T::Error> {
        self.clone().try_into()
    }
}

impl From<bool> for ConfigValue {
    fn from(value: bool) -> Self {
        Self {
            kind: ValueKind::Boolean(value),
        }
    }
}

impl Serialize for ConfigValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use config::ValueKind;
        match &self.kind {
            ValueKind::Nil => serializer.serialize_none(),
            ValueKind::Boolean(v) => serializer.serialize_bool(*v),
            ValueKind::I64(v) => serializer.serialize_i64(*v),
            ValueKind::I128(v) => serializer.serialize_i128(*v),
            ValueKind::U64(v) => serializer.serialize_u64(*v),
            ValueKind::U128(v) => serializer.serialize_u128(*v),
            ValueKind::Float(v) => serializer.serialize_f64(*v),
            ValueKind::String(v) => serializer.serialize_str(v),
            ValueKind::Table(t) => {
                let mut map = serializer.serialize_map(Some(t.len()))?;
                for (k, v) in t {
                    map.serialize_entry(&k, &ConfigValue::from(v))?;
                }
                map.end()
            }
            ValueKind::Array(a) => {
                let mut seq = serializer.serialize_seq(Some(a.len()))?;
                for e in a {
                    seq.serialize_element(&ConfigValue::from(e))?;
                }
                seq.end()
            }
        }
    }
}

pub type DefaultValue<T> = (&'static str, T);

impl Default for Settings {
    fn default() -> Self {
        let config_fileformat = FileFormat::Toml;
        let config_folder = config_dir()
            .context("Context directory now found")
            .unwrap()
            .join(PROGRAM);
        let config_filename = Settings::filename("config", config_fileformat);
        let config_filepath = PathBuf::from(&config_folder).join(&config_filename);
        let default_log_path = config_folder.join("logs");
        let default_script_path = config_folder.join("scripts");

        let mut builder: ConfigBuilder<config::builder::DefaultState> = Config::builder();

        // Set Default Settings here.
        let default_settings_str: Vec<DefaultValue<&str>> = vec![
            ("auth.twitch.username", ""),
            ("auth.twitch.client_id", ""),
            ("auth.twitch.client_secret", ""),
            (
                "auth.twitch.redirect_url",
                "http://localhost:27934/twitch/register",
            ),
            ("auth.youtube.username", ""),
            ("auth.youtube.api_token", ""),
            ("servers.default", ""),
            ("log_folder", default_log_path.to_str().unwrap()),
            ("script_folder", default_script_path.to_str().unwrap()),
            ("max_log_level", "Info"),
        ];
        builder = Settings::default_loop(builder, default_settings_str);

        let default_settings_bool: Vec<DefaultValue<bool>> = vec![
            ("servers.autostart", false),
            ("debug", false),
            ("show_logs", true),
            ("auth.twitch.auto_connect", true),
            ("auth.youtube.auto_connect", true),
        ];

        builder = Settings::default_loop(builder, default_settings_bool);

        let default_settings_list_str = vec![
            ("auth.platforms", vec!["Twitch", "YouTube"]),
            (
                "auth.twitch.websocket_subscription",
                vec![
                    "channel.chat.message",
                    "channel.subscribe",
                    "channel.subscription.message",
                    "channel.channel_points_custom_reward_redemption.add",
                    "channel.channel_points_custom_reward_redemption.update",
                ],
            ),
        ];
        builder = Settings::default_loop(builder, default_settings_list_str);

        // let default_settings_int: Vec<DefaultValue<u16>> = vec![("servers.example.port", 4312)];
        // builder = Settings::default_loop(builder, default_settings_int);

        let settings = Self {
            config_builder: builder,
            config_folder: config_folder.clone(),
            config_filename,
            config_fileformat,
            log_folder: default_log_path.clone(),
            script_folder: default_script_path.clone(),
        };
        let _ = settings.make_config_exists(config_folder.as_path(), FileType::Dir);
        let _ = settings.make_config_exists(&config_filepath, FileType::File);
        settings
    }
}

impl Settings {
    fn default_loop<T: BuilderState, D: Into<ValueKind>>(
        builder: ConfigBuilder<T>,
        default_values: Vec<DefaultValue<D>>,
    ) -> ConfigBuilder<T> {
        let mut b = builder;
        for (key, value) in default_values {
            b = b.set_default(key, value).unwrap();
        }
        b
    }
}

#[tauri::command]
#[instrument(level = "trace")]
pub fn set_config_string(_key: String, value: String) {
    dbg!(value);
}

#[tauri::command]
#[instrument(level = "trace")]
pub fn set_config_int(_key: String, value: i128) {
    dbg!(value);
}

#[tauri::command]
#[instrument(level = "trace")]
pub fn set_config_uint(_key: String, value: u128) {
    dbg!(value);
}

#[tauri::command]
#[instrument(level = "trace")]
pub fn set_config_float(_key: String, value: f64) {
    dbg!(value);
}

#[tauri::command]
#[instrument(level = "trace", skip(config))]
pub async fn get_config_bool(
    key: String,
    config: State<'_, Arc<futures::lock::Mutex<Config>>>,
) -> Result<bool, String> {
    let config = config.lock().await.clone();
    match config.get_bool(&key) {
        Ok(b) => {
            info!("{}: {}", &key, b);
            Ok(b)
        }
        Err(e) => {
            error!("{:?}", e);
            Err(format!("{:?}", e))
        }
    }
}

#[tauri::command]
#[instrument(level = "trace")]
pub fn set_config_bool(_key: String, value: bool) {
    dbg!(value);
}

#[tauri::command]
#[instrument(level = "trace")]
pub fn set_config_array(_key: String, value: Vec<String>) {
    dbg!(value);
}

#[tauri::command]
#[instrument(level = "trace", skip_all)]
pub async fn update_config(config: State<'_, Arc<futures::lock::Mutex<Config>>>) -> Result<(), ()> {
    let mut config_locked = config.lock().await;
    let new_config = Settings::current_config();
    *config_locked = new_config;
    Ok(())
}
