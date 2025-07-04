#[cfg(test)]
mod tests {
    use crate::vblconfig::VblConfig;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    // Import the logging macros - they're exported at crate level
    use crate::{
        log_api_error, log_api_request, log_api_response, log_widget_error, log_widget_start,
        log_widget_success,
    };

    #[test]
    fn test_logging_initialization() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let log_file_path = temp_dir.path().join("test.log");

        // Create a custom config for testing
        let config_content = format!(
            r#"log_level = "debug"
log_file_path = "{}"
console_log_level = "info""#,
            log_file_path.display(),
        );

        let config_dir = temp_dir.path().join("data");
        fs::create_dir_all(&config_dir).expect("Failed to create config dir");
        let config_path = config_dir.join("vblconfig.toml");
        fs::write(&config_path, config_content).expect("Failed to write config");

        // Test that init_logging works (this is a basic smoke test since
        // we can't easily test the actual logging output in a unit test)
        // The real test would be an integration test

        // For now, just test that VblConfig can be loaded
        let config = VblConfig::load();
        assert!(config.is_ok(), "Config should load successfully");
    }

    #[test]
    fn test_vbl_config_default() {
        let config = VblConfig::default();
        assert_eq!(config.log_level, crate::vblconfig::DEFAULT_LOG_LEVEL);
        assert_eq!(
            config.log_file_path,
            crate::vblconfig::DEFAULT_LOG_FILE_PATH,
        );
        assert_eq!(
            config.console_log_level,
            Some(crate::vblconfig::DEFAULT_CONSOLE_LOG_LEVEL.to_string()),
        );
    }

    #[test]
    fn test_vbl_config_log_level_parsing() {
        let config = VblConfig::default();

        // Test valid log levels
        assert_eq!(config.parse_log_level("error"), log::LevelFilter::Error);
        assert_eq!(config.parse_log_level("warn"), log::LevelFilter::Warn);
        assert_eq!(config.parse_log_level("info"), log::LevelFilter::Info);
        assert_eq!(config.parse_log_level("debug"), log::LevelFilter::Debug);
        assert_eq!(config.parse_log_level("trace"), log::LevelFilter::Trace);
        assert_eq!(config.parse_log_level("off"), log::LevelFilter::Off);

        // Test invalid log level defaults to info
        assert_eq!(config.parse_log_level("invalid"), log::LevelFilter::Info);
    }

    #[test]
    fn test_vbl_config_paths() {
        let config = VblConfig {
            log_level: "debug".to_string(),
            log_file_path: "custom/path/log.txt".to_string(),
            console_log_level: Some("warn".to_string()),
            schedule_file_path: Some("custom/schedule.json".to_string()),
            schedule_backup_path: Some("custom/backup.json".to_string()),
        };

        assert_eq!(config.get_log_level(), log::LevelFilter::Debug);
        assert_eq!(config.get_console_log_level(), log::LevelFilter::Warn);
        assert_eq!(
            config.get_log_file_path(),
            PathBuf::from("custom/path/log.txt"),
        );
        assert_eq!(
            config.get_schedule_file_path(),
            PathBuf::from("custom/schedule.json"),
        );
        assert_eq!(
            config.get_schedule_backup_path(),
            PathBuf::from("custom/backup.json"),
        );
    }

    #[test]
    fn test_logging_macros_exist() {
        // Test that our logging macros are available
        // These are compile-time tests - if the macros don't exist, this won't compile

        // We can't easily test the actual logging output in unit tests,
        // but we can verify the macros compile correctly
        let widget = "test_widget";
        let input = "test_input";
        let duration = std::time::Duration::from_millis(100);
        let error = "test_error";

        // These should compile without errors:
        log_widget_start!(widget, input);
        log_widget_success!(widget, duration);
        log_widget_error!(widget, error, duration);
        log_api_request!("GET", "http://example.com");
        log_api_response!("200", duration);
        log_api_error!(error, duration);
    }

    #[test]
    fn test_log_timestamp_format() {
        use tempfile::NamedTempFile;

        // Create a temporary log file
        let temp_log = NamedTempFile::new().expect("Failed to create temp log file");
        let log_path = temp_log.path().to_string_lossy().to_string();

        // Create a custom config that uses our temp log file
        let config_content = format!(
            r#"log_level = "info"
log_file_path = "{}"
console_log_level = "info""#,
            log_path,
        );

        // Save original config if it exists
        let config_path = std::path::PathBuf::from(crate::vblconfig::CONFIG_FILE_PATH);
        let original_config = if config_path.exists() {
            Some(std::fs::read_to_string(&config_path).ok())
        } else {
            None
        };

        // Create test config
        std::fs::create_dir_all("data").ok();
        std::fs::write(&config_path, config_content).expect("Failed to write test config");

        // Initialize logging with our test config
        let init_result = crate::logging::init_logging();

        // Restore original config
        if let Some(Some(orig)) = original_config {
            std::fs::write(&config_path, orig).ok();
        } else {
            std::fs::remove_file(&config_path).ok();
        }

        assert!(init_result.is_ok(), "Logging initialization should succeed");

        // Generate a log entry
        log::info!("Test log entry for timestamp validation");

        // Wait a bit for the log to be written
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Read the log file content
        let log_content =
            std::fs::read_to_string(&log_path).expect("Should be able to read log file");

        // Verify the timestamp format
        // Local time format should be: "YYYY-MM-DD HH:MM:SS.sss" (no UTC suffix)
        // UTC format would be: "YYYY-MM-DD HH:MM:SS.sss UTC"

        let lines: Vec<&str> = log_content.lines().collect();
        assert!(
            !lines.is_empty(),
            "Log file should contain at least one line",
        );

        let last_line = lines.last().expect("Should have at least one log line");

        // Check that the line contains our test message
        assert!(
            last_line.contains("Test log entry for timestamp validation"),
            "Log should contain our test message",
        );

        // Check timestamp format: should start with date-time pattern and NOT contain "UTC"
        // Pattern: YYYY-MM-DD HH:MM:SS.mmm [LEVEL]
        let timestamp_pattern =
            regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3} \[INFO\]")
                .expect("Regex should compile");

        assert!(
            timestamp_pattern.is_match(last_line),
            "Log line should start with local time format (YYYY-MM-DD HH:MM:SS.mmm [LEVEL]), got: {}",
            last_line,
        );

        // Most importantly, verify it does NOT contain "UTC"
        assert!(
            !last_line.contains("UTC"),
            "Log timestamp should NOT contain 'UTC' (should be local time), got: {}",
            last_line,
        );
    }
}
