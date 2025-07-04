use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use log::LevelFilter;
use crate::errors::VestaboardError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VblConfig {
    pub log_level: String,
    pub log_file_path: String,
    pub console_log_level: Option<String>,
}

impl Default for VblConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_file_path: "data/vestaboard.log".to_string(),
            console_log_level: Some("info".to_string()),
        }
    }
}

impl VblConfig {
    pub fn load() -> Result<Self, VestaboardError> {
        let config_path = PathBuf::from("data/vblconfig.toml");
        
        if !config_path.exists() {
            log::info!("Config file not found, creating default config at {}", config_path.display());
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| VestaboardError::io_error(e, "reading config file"))?;
        
        let config: VblConfig = toml::from_str(&config_content)
            .map_err(|e| VestaboardError::other(&format!("Invalid config format: {}", e)))?;
        
        log::debug!("Loaded config: {:?}", config);
        Ok(config)
    }

    pub fn save(&self) -> Result<(), VestaboardError> {
        let config_path = PathBuf::from("data/vblconfig.toml");
        
        // Ensure data directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| VestaboardError::io_error(e, "creating config directory"))?;
        }

        let config_content = toml::to_string_pretty(self)
            .map_err(|e| VestaboardError::other(&format!("Failed to serialize config: {}", e)))?;
        
        fs::write(&config_path, config_content)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VblConfig::default();
        assert_eq!(config.log_level, "info");
        assert_eq!(config.log_file_path, "data/vestaboard.log");
        assert_eq!(config.console_log_level, Some("info".to_string()));
    }

    #[test]
    fn test_log_level_parsing() {
        let config = VblConfig::default();
        assert_eq!(config.parse_log_level("error"), LevelFilter::Error);
        assert_eq!(config.parse_log_level("warn"), LevelFilter::Warn);
        assert_eq!(config.parse_log_level("info"), LevelFilter::Info);
        assert_eq!(config.parse_log_level("debug"), LevelFilter::Debug);
        assert_eq!(config.parse_log_level("trace"), LevelFilter::Trace);
        assert_eq!(config.parse_log_level("invalid"), LevelFilter::Info);
    }
}
