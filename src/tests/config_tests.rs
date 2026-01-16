#[cfg(test)]
mod tests {
  use crate::api::TransportType;
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
    assert_eq!(config.transport, None);
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
      check_interval_seconds: Some(5),
      playlist_file_path: None,
      runtime_state_path: None,
      lock_file_path: None,
      transport: None,
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
      check_interval_seconds: None,
      playlist_file_path: None,
      runtime_state_path: None,
      lock_file_path: None,
      transport: None,
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
        let _transport = config.get_transport();

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

  // Transport configuration tests

  #[test]
  fn test_get_transport_defaults_to_local() {
    let config = Config::default();
    assert_eq!(config.get_transport(), TransportType::Local);
  }

  #[test]
  fn test_get_transport_returns_configured_value() {
    let mut config = Config::default();

    config.transport = Some(TransportType::Local);
    assert_eq!(config.get_transport(), TransportType::Local);

    config.transport = Some(TransportType::Internet);
    assert_eq!(config.get_transport(), TransportType::Internet);
  }

  #[test]
  fn test_transport_none_defaults_to_local() {
    let config = Config {
      log_level: DEFAULT_LOG_LEVEL.to_string(),
      log_file_path: DEFAULT_LOG_FILE_PATH.to_string(),
      console_log_level: None,
      schedule_file_path: None,
      schedule_backup_path: None,
      check_interval_seconds: None,
      playlist_file_path: None,
      runtime_state_path: None,
      lock_file_path: None,
      transport: None,
    };
    assert_eq!(config.get_transport(), TransportType::Local);
  }

  #[test]
  fn test_transport_toml_parsing_local() {
    let toml_str = r#"
      log_level = "info"
      log_file_path = "data/vestaboard.log"
      transport = "local"
    "#;
    let config: Config = toml::from_str(toml_str).expect("Failed to parse TOML");
    assert_eq!(config.transport, Some(TransportType::Local));
    assert_eq!(config.get_transport(), TransportType::Local);
  }

  #[test]
  fn test_transport_toml_parsing_internet() {
    let toml_str = r#"
      log_level = "info"
      log_file_path = "data/vestaboard.log"
      transport = "internet"
    "#;
    let config: Config = toml::from_str(toml_str).expect("Failed to parse TOML");
    assert_eq!(config.transport, Some(TransportType::Internet));
    assert_eq!(config.get_transport(), TransportType::Internet);
  }

  #[test]
  fn test_transport_toml_parsing_missing_defaults_to_local() {
    let toml_str = r#"
      log_level = "info"
      log_file_path = "data/vestaboard.log"
    "#;
    let config: Config = toml::from_str(toml_str).expect("Failed to parse TOML");
    assert_eq!(config.transport, None);
    assert_eq!(config.get_transport(), TransportType::Local);
  }

  #[test]
  fn test_transport_toml_parsing_invalid_value() {
    let toml_str = r#"
      log_level = "info"
      log_file_path = "data/vestaboard.log"
      transport = "wifi"
    "#;
    let result: Result<Config, _> = toml::from_str(toml_str);
    assert!(result.is_err(), "Invalid transport value should fail to parse");
  }

  #[test]
  fn test_config_serializes_transport() {
    let mut config = Config::default();
    config.transport = Some(TransportType::Internet);

    let toml_str = toml::to_string(&config).expect("Failed to serialize config");
    assert!(toml_str.contains("transport = \"internet\""), "Serialized config should contain transport");
  }
}
