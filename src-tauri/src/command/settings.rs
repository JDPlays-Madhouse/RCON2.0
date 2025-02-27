use anyhow::{anyhow, Context, Error, Result};
use config::{
    builder::{BuilderState, DefaultState},
    ConfigBuilder, Map, Value, ValueKind,
};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};
use tracing::error;

use crate::settings::{ConfigValue, DefaultValue, Settings};
use config::{Config, Environment, File, FileFormat};

use super::Command;

pub enum FileType {
    Dir,
    File,
}

#[derive(Debug)]
pub struct ScriptSettings {
    pub config_builder: ConfigBuilder<DefaultState>,
    pub scripts_folder: PathBuf,
    pub config_filename: String,
    pub config_fileformat: FileFormat,
}

impl ScriptSettings {
    pub fn new() -> Self {
        let mut mut_self = Self::default();

        let builder = mut_self
            .config_builder
            .clone()
            .add_source(File::new(
                mut_self.config_filepath().to_str().unwrap(),
                mut_self.config_fileformat,
            ))
            .add_source(Environment::with_prefix("RCON_SCRIPTS"));
        mut_self.config_builder = builder.clone();
        let _ = mut_self.write();
        mut_self
    }

    pub fn scripts_folder() -> PathBuf {
        Self::new().scripts_folder
    }

    pub fn new_from_config(config: Config) -> Self {
        let mut settings = Self::new();
        settings.config_builder = settings.config_builder.add_source(config);
        settings
    }

    pub fn config_filepath(&self) -> PathBuf {
        let config_folder = &self.scripts_folder;
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
        match self.config().get::<Value>(key) {
            Ok(value) => Some(ConfigValue::from(value)),
            Err(e) => {
                error!("Config not available: {:?}", e);
                None
            }
        }
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

    pub fn config(&self) -> Config {
        self.config_builder
            .build_cloned()
            .context("Build config")
            .unwrap()
    }

    pub fn make_config_exists(&self, path: &Path, filetype: FileType) -> Result<FileType, Error> {
        match (ScriptSettings::file_exists(path), filetype) {
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
                self.write().expect("writing to file");
                Ok(FileType::File)
            }
        }
    }

    pub fn get_command(&self, id: &str) -> Option<Command> {
        let config = self.config();
        let command_table = match config.get_table(id) {
            Ok(c) => c,
            Err(e) => {
                error!("{}", e);
                return None;
            }
        };

        match Command::try_from(command_table) {
            Ok(command) => Some(command.set_name(id)),
            Err(e) => {
                error!("Command not found: {id}");
                dbg!(e);
                None
            }
        }
    }

    /// Gets all commands from the scripts config file.
    pub fn get_commands() -> Vec<Command> {
        let script_settings = ScriptSettings::new();
        let config = script_settings.config();

        let commands = config
            .try_deserialize::<Map<String, Value>>()
            .unwrap()
            .iter()
            .filter_map(|(k, v)| match Command::try_from(v.clone()) {
                Ok(c) => Some(c.set_name(k)),
                Err(_e) => None,
            })
            .collect();
        // BUG: Removes Channel point rewards...
        // script_settings.set_commands(&commands);
        commands
    }

    /// Sets a command in config and returns [`None`] if no command with that name exists otherwise
    /// returns [`Some<Command>`].
    pub fn set_command(&mut self, command: Command) -> Option<Command> {
        let old_command = self.get_command(&command.name);
        let builder = self
            .config_builder
            .clone()
            .set_override(command.name.clone(), command)
            .expect("Setting override");
        self.config_builder = builder;
        let _ = self.write();
        old_command
    }

    pub fn set_commands(&mut self, commands: &Vec<Command>) {
        let commands = commands.clone();
        let mut builder = self.config_builder.clone();
        for command in commands {
            builder = builder
                .set_override(command.name.clone(), command.clone())
                .expect("Setting override");
        }
        self.config_builder = builder;
        let _ = self.write();
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

impl Default for ScriptSettings {
    fn default() -> Self {
        let settings = Settings::new();
        let config_fileformat = FileFormat::Toml;
        let config_folder = settings.script_folder;
        let config_filename = ScriptSettings::filename("config", config_fileformat);

        let builder: ConfigBuilder<config::builder::DefaultState> = Config::builder();

        let script_settings = Self {
            config_builder: builder,
            config_filename,
            config_fileformat,
            scripts_folder: config_folder.clone(),
        };
        let _ = script_settings.make_config_exists(config_folder.as_path(), FileType::Dir);
        let _ =
            script_settings.make_config_exists(&script_settings.config_filepath(), FileType::File);
        script_settings
    }
}

#[allow(dead_code)]
impl ScriptSettings {
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
