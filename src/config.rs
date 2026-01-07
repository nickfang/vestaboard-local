use crate::cli_display::{ print_error, print_progress, print_success };
use crate::errors::VestaboardError;
use log::LevelFilter;
use serde::{ Deserialize, Serialize };
use std::fs;
use std::path::PathBuf;

// Configuration file and default paths
pub const CONFIG_FILE_PATH: &str = "data/vblconfig.toml";
pub const DEFAULT_LOG_LEVEL: &str = "info";
pub const DEFAULT_API_TIMEOUT_SECONDS: u64 = 5;
pub const DEFAULT_LOG_FILE_PATH: &str = "data/vestaboard.log";
pub const DEFAULT_CONSOLE_LOG_LEVEL: &str = "info";
pub const DEFAULT_SCHEDULE_FILE_PATH: &str = "data/schedule.json";
pub const DEFAULT_SCHEDULE_BACKUP_PATH: &str = "data/schedule_backup.json";
pub const DEFAULT_PLAYLIST_FILE_PATH: &str = "data/playlist.json";
pub const DEFAULT_RUNTIME_STATE_PATH: &str = "data/runtime_state.json";
pub const DEFAULT_LOCK_FILE_PATH: &str = "data/vestaboard.lock";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
  pub log_level: String,
  pub log_file_path: String,
  pub console_log_level: Option<String>,
  pub schedule_file_path: Option<String>,
  pub schedule_backup_path: Option<String>,
  pub check_interval_seconds: Option<u64>,
  pub playlist_file_path: Option<String>,
  pub runtime_state_path: Option<String>,
  pub lock_file_path: Option<String>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      log_level: DEFAULT_LOG_LEVEL.to_string(),
      log_file_path: DEFAULT_LOG_FILE_PATH.to_string(),
      console_log_level: Some(DEFAULT_CONSOLE_LOG_LEVEL.to_string()),
      schedule_file_path: Some(DEFAULT_SCHEDULE_FILE_PATH.to_string()),
      schedule_backup_path: Some(DEFAULT_SCHEDULE_BACKUP_PATH.to_string()),
      check_interval_seconds: Some(3),
      playlist_file_path: Some(DEFAULT_PLAYLIST_FILE_PATH.to_string()),
      runtime_state_path: Some(DEFAULT_RUNTIME_STATE_PATH.to_string()),
      lock_file_path: Some(DEFAULT_LOCK_FILE_PATH.to_string()),
    }
  }
}

impl Config {
  pub fn load() -> Result<Self, VestaboardError> {
    Self::load_internal(true)
  }

  /// Load configuration without printing progress messages
  /// Used for internal operations like logging initialization
  pub fn load_silent() -> Result<Self, VestaboardError> {
    Self::load_internal(false)
  }

  fn load_internal(show_messages: bool) -> Result<Self, VestaboardError> {
    let config_path = PathBuf::from(CONFIG_FILE_PATH);

    if !config_path.exists() {
      log::info!("Config file not found, creating default config at {}", config_path.display());
      if show_messages {
        print_progress("Creating default configuration...");
      }
      let default_config = Self::default();
      default_config.save()?;
      if show_messages {
        print_success("Default configuration created");
      }
      return Ok(default_config);
    }

    if show_messages {
      print_progress("Loading configuration...");
    }
    let config_content = fs::read_to_string(&config_path).map_err(|e| {
      let error = VestaboardError::io_error(e, "reading config file");
      if show_messages {
        print_error(&format!("Error loading configuration: {}", error.to_user_message()));
      }
      error
    })?;

    let config: Config = toml::from_str(&config_content).map_err(|e| {
      let error = VestaboardError::other(&format!("Invalid config format: {}", e));
      if show_messages {
        print_error(&format!("Error loading configuration: {}", error.to_user_message()));
      }
      error
    })?;

    log::debug!("Loaded config: {:?}", config);
    if show_messages {
      print_success("Configuration loaded");
    }
    Ok(config)
  }

  pub fn save(&self) -> Result<(), VestaboardError> {
    let config_path = PathBuf::from(CONFIG_FILE_PATH);

    // Ensure data directory exists
    if let Some(parent) = config_path.parent() {
      fs
        ::create_dir_all(parent)
        .map_err(|e| VestaboardError::io_error(e, "creating config directory"))?;
    }

    let config_content = toml
      ::to_string_pretty(self)
      .map_err(|e| VestaboardError::other(&format!("Failed to serialize config: {}", e)))?;

    fs
      ::write(&config_path, config_content)
      .map_err(|e| VestaboardError::io_error(e, "writing config file"))?;

    log::debug!("Saved config to {}", config_path.display());
    Ok(())
  }

  pub fn get_log_level(&self) -> LevelFilter {
    self.parse_log_level(&self.log_level)
  }

  pub fn get_console_log_level(&self) -> LevelFilter {
    self.console_log_level
      .as_ref()
      .map(|level| self.parse_log_level(level))
      .unwrap_or_else(|| self.get_log_level())
  }

  pub fn parse_log_level(&self, level: &str) -> LevelFilter {
    match level.to_lowercase().as_str() {
      "off" => LevelFilter::Off,
      "error" => LevelFilter::Error,
      "warn" => LevelFilter::Warn,
      "info" => LevelFilter::Info,
      "debug" => LevelFilter::Debug,
      "trace" => LevelFilter::Trace,
      _ => {
        eprintln!("Invalid log level '{}', defaulting to 'info'", level);
        LevelFilter::Info
      }
    }
  }

  pub fn get_log_file_path(&self) -> PathBuf {
    PathBuf::from(&self.log_file_path)
  }

  pub fn get_schedule_file_path(&self) -> PathBuf {
    PathBuf::from(self.schedule_file_path.as_deref().unwrap_or(DEFAULT_SCHEDULE_FILE_PATH))
  }

  pub fn get_schedule_backup_path(&self) -> PathBuf {
    PathBuf::from(self.schedule_backup_path.as_deref().unwrap_or(DEFAULT_SCHEDULE_BACKUP_PATH))
  }

  pub fn get_check_interval_seconds(&self) -> u64 {
    self.check_interval_seconds.unwrap_or(3)
  }

  pub fn get_playlist_file_path(&self) -> PathBuf {
    PathBuf::from(self.playlist_file_path.as_deref().unwrap_or(DEFAULT_PLAYLIST_FILE_PATH))
  }

  pub fn get_runtime_state_path(&self) -> PathBuf {
    PathBuf::from(self.runtime_state_path.as_deref().unwrap_or(DEFAULT_RUNTIME_STATE_PATH))
  }

  pub fn get_lock_file_path(&self) -> PathBuf {
    PathBuf::from(self.lock_file_path.as_deref().unwrap_or(DEFAULT_LOCK_FILE_PATH))
  }
}
