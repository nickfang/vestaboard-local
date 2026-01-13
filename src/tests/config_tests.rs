#[cfg(test)]
mod tests {
  use crate::config::{
    Config, DEFAULT_CONSOLE_LOG_LEVEL, DEFAULT_LOG_FILE_PATH, DEFAULT_LOG_LEVEL, DEFAULT_SCHEDULE_BACKUP_PATH,
    DEFAULT_SCHEDULE_FILE_PATH,
  };
  use log::LevelFilter;
  use std::path::PathBuf;

  #[test]
  fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.log_level, DEFAULT_LOG_LEVEL);
    assert_eq!(config.log_file_path, DEFAULT_LOG_FILE_PATH);
    assert_eq!(config.console_log_level, Some(DEFAULT_CONSOLE_LOG_LEVEL.to_string()));
    assert_eq!(config.schedule_file_path, Some(DEFAULT_SCHEDULE_FILE_PATH.to_string()));
    assert_eq!(config.schedule_backup_path, Some(DEFAULT_SCHEDULE_BACKUP_PATH.to_string()));
  }

  #[test]
  fn test_schedule_path_getters() {
    // Test with default config
    let default_config = Config::default();
    assert_eq!(default_config.get_schedule_file_path(), PathBuf::from(DEFAULT_SCHEDULE_FILE_PATH));
    assert_eq!(default_config.get_schedule_backup_path(), PathBuf::from(DEFAULT_SCHEDULE_BACKUP_PATH));

    // Test with custom paths
    let custom_config = Config {
      log_level: DEFAULT_LOG_LEVEL.to_string(),
      log_file_path: DEFAULT_LOG_FILE_PATH.to_string(),
      console_log_level: Some(DEFAULT_CONSOLE_LOG_LEVEL.to_string()),
      schedule_file_path: Some("custom/schedule.json".to_string()),
      schedule_backup_path: Some("custom/backup.json".to_string()),
    };
    assert_eq!(custom_config.get_schedule_file_path(), PathBuf::from("custom/schedule.json"));
    assert_eq!(custom_config.get_schedule_backup_path(), PathBuf::from("custom/backup.json"));

    // Test with missing fields (backward compatibility)
    let minimal_config = Config {
      log_level: DEFAULT_LOG_LEVEL.to_string(),
      log_file_path: DEFAULT_LOG_FILE_PATH.to_string(),
      console_log_level: Some(DEFAULT_CONSOLE_LOG_LEVEL.to_string()),
      schedule_file_path: None,
      schedule_backup_path: None,
    };
    assert_eq!(minimal_config.get_schedule_file_path(), PathBuf::from(DEFAULT_SCHEDULE_FILE_PATH));
    assert_eq!(minimal_config.get_schedule_backup_path(), PathBuf::from(DEFAULT_SCHEDULE_BACKUP_PATH));
  }

  #[test]
  fn test_log_level_parsing() {
    let config = Config::default();
    assert_eq!(config.parse_log_level("error"), LevelFilter::Error);
    assert_eq!(config.parse_log_level("warn"), LevelFilter::Warn);
    assert_eq!(config.parse_log_level("info"), LevelFilter::Info);
    assert_eq!(config.parse_log_level("debug"), LevelFilter::Debug);
    assert_eq!(config.parse_log_level("trace"), LevelFilter::Trace);
    assert_eq!(config.parse_log_level("invalid"), LevelFilter::Info);
  }

  #[test]
  fn test_load_actual_config_file() {
    // This test loads the actual config file to verify it works with new fields
    match Config::load() {
      Ok(config) => {
        // Test that all getters work
        let _log_path = config.get_log_file_path();
        let _schedule_path = config.get_schedule_file_path();
        let _backup_path = config.get_schedule_backup_path();
        let _log_level = config.get_log_level();
        let _console_level = config.get_console_log_level();

        // Basic validation - should be sensible defaults
        assert!(!config.log_level.is_empty());
        assert!(!config.log_file_path.is_empty());

        // Schedule paths should resolve to something reasonable
        assert!(config.get_schedule_file_path().to_string_lossy().contains("schedule"));
        assert!(config.get_schedule_backup_path().to_string_lossy().contains("backup"));
      },
      Err(e) => {
        // Config loading should not fail
        panic!("Failed to load config: {}", e);
      },
    }
  }
}
