#[path = "../daemon.rs"]
mod daemon;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::{ Schedule, ScheduledTask };

    use daemon::{ get_file_mod_time, execute_task, run_daemon };
    use tempfile::NamedTempFile;
    use std::io::{ Write, Seek };

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

    #[test]
    #[ignore]
    fn test_run_daemon() {
        // this currently cannot be tested because it runs indefinitely and requires a Ctrl+C signal to stop
        // TODO: refactor to allow testing, or mock the Ctrl+C signal
        assert!(true)
    }
}
