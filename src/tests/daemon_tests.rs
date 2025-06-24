#[path = "../daemon.rs"]
mod daemon;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::{ Schedule, ScheduledTask };
    use crate::errors::VestaboardError;

    use daemon::{ get_file_mod_time, execute_task, run_daemon };
    use tempfile::NamedTempFile;
    use std::io::{ Write, Seek };
    use std::path::PathBuf;

    #[test]
    fn test_get_file_mod_time() {
        let earlier_time = std::time::SystemTime::now() - std::time::Duration::from_secs(60);
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file, "test content").expect("Failed to write to temp file");
        temp_file.as_file_mut().flush().expect("Failed to flush temp file");

        let path = temp_file.path().to_path_buf();
        let result = get_file_mod_time(&path);

        assert!(result.is_ok());
        let mod_time = result.unwrap();
        assert!(mod_time <= std::time::SystemTime::now());
        assert!(mod_time > earlier_time);
    }

    #[test]
    fn test_get_file_mod_time_error_context() {
        let non_existent_path = PathBuf::from("/this/path/does/not/exist");
        let result = get_file_mod_time(&non_existent_path);

        assert!(result.is_err());
        let error = result.unwrap_err();

        // Check that it's an IO error with proper context
        let error_msg = format!("{}", error);
        match error {
            VestaboardError::IOError { context, .. } => {
                assert!(context.contains("getting mod time for"));
                assert!(context.contains("/this/path/does/not/exist"));
            }
            _ => panic!("Expected IOError with context"),
        }

        // Check display formatting includes context
        assert!(error_msg.contains("IO Error"));
        assert!(error_msg.contains("getting mod time for"));
    }

    #[tokio::test]
    #[ignore]
    // figure out how to test without sending to vestaboard
    async fn test_execute_task() {
        let task = ScheduledTask {
            id: "test_task".to_string(),
            time: chrono::Utc::now(),
            widget: "weather".to_string(),
            input: serde_json::Value::String("".to_string()),
        };
        let result = execute_task(&task);
        assert!(result.await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_task_unknown_widget_error() {
        let task = ScheduledTask {
            id: "test_task".to_string(),
            time: chrono::Utc::now(),
            widget: "unknown_widget".to_string(),
            input: serde_json::Value::String("test".to_string()),
        };

        let result = execute_task(&task).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            VestaboardError::WidgetError { widget, message } => {
                assert_eq!(widget, "unknown_widget");
                assert!(message.contains("Unknown widget type"));
            }
            _ => panic!("Expected WidgetError"),
        }
    }

    #[test]
    #[ignore]
    fn test_run_daemon() {
        // this currently cannot be tested because it runs indefinitely and requires a Ctrl+C signal to stop
        // TODO: refactor to allow testing, or mock the Ctrl+C signal
        assert!(true)
    }
}
