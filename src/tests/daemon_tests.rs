#[path = "../daemon.rs"]
mod daemon;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::{ Schedule, ScheduledTask };
    use crate::errors::VestaboardError;

    use daemon::{ load_schedule, get_file_mod_time, execute_task, run_daemon };
    use chrono::{ DateTime, TimeZone, Utc };
    use tempfile::NamedTempFile;
    use std::path::PathBuf;
    use serde_json::json;
    use std::io::{ Write, Seek };

    fn create_valid_json_content() -> (String, DateTime<Utc>, String) {
        let task_time = Utc.with_ymd_and_hms(2025, 5, 4, 18, 30, 0).unwrap();
        let task_id = "tst1".to_string();
        let schedule = Schedule {
            tasks: vec![ScheduledTask {
                id: task_id.clone(),
                time: task_time,
                widget: "test_widget".to_string(),
                input: json!({"value": "test_input"}),
            }],
        };
        let json_string = serde_json::to_string_pretty(&schedule).unwrap();
        (json_string, task_time, task_id)
    }

    #[test]
    fn save_schedule_test() {
        use daemon::save_schedule;
        use std::io::Read;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();

        // Test saving an empty schedule
        let empty_schedule = Schedule::default();
        let result = save_schedule(&empty_schedule, &path);
        assert!(result.is_ok());

        // Verify the saved content is an empty schedule
        let mut file_content = String::new();
        temp_file.read_to_string(&mut file_content).expect("Failed to read from temp file");
        assert_eq!(file_content, "{\"tasks\":[]}");

        // Test saving a schedule with tasks
        let task1_time = Utc.with_ymd_and_hms(2025, 5, 1, 9, 0, 0).unwrap();
        let task1 = ScheduledTask::new(task1_time, "Weather".to_string(), json!({}));
        let task2_time = Utc.with_ymd_and_hms(2025, 5, 1, 17, 30, 0).unwrap();
        let task2 = ScheduledTask::new(
            task2_time,
            "text".to_string(),
            json!({"message": "Hello, world!"})
        );

        let mut schedule = Schedule::default();
        schedule.add_task(task1);
        schedule.add_task(task2);

        let result = save_schedule(&schedule, &path);
        assert!(result.is_ok());

        // Verify the saved content matches the schedule with tasks
        temp_file.seek(std::io::SeekFrom::Start(0)).expect("Failed to seek to start of file");
        file_content.clear();
        temp_file.read_to_string(&mut file_content).expect("Failed to read from temp file");

        let expected_json = serde_json::to_string(&schedule).unwrap();
        assert_eq!(file_content, expected_json);
    }

    #[test]
    fn test_load_schedule_success() {
        let (json_content, expected_time, expected_id) = create_valid_json_content();
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file, "{}", json_content).expect("Failed to write to temp file");
        temp_file.as_file_mut().flush().expect("Failed to flush temp file");
        println!("Temporary file path: {:?}", temp_file.path());

        let path = temp_file.path().to_path_buf();
        let result = load_schedule(&path);

        assert!(result.is_ok());
        let schedule = result.unwrap();
        assert_eq!(schedule.tasks.len(), 1);
        assert_eq!(schedule.tasks[0].id, expected_id);
        assert_eq!(schedule.tasks[0].widget, "test_widget");
        assert_eq!(schedule.tasks[0].time, expected_time);
        assert_eq!(schedule.tasks[0].input["value"], "test_input");
    }

    #[test]
    fn test_load_schedule_file_not_found() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("non_existent_schedule.json");

        let result = load_schedule(&path);

        assert!(result.is_ok());
        let schedule = result.unwrap();
        assert!(schedule.tasks.is_empty());
    }

    #[test]
    fn test_load_schedule_empty_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        // Write nothing (or just whitespace)
        write!(temp_file, "").expect("Failed to write empty string");
        temp_file.flush().expect("Failed to flush");

        let path = temp_file.path().to_path_buf();
        let result = load_schedule(&path);

        assert!(result.is_ok());
        let schedule = result.unwrap();
        assert!(schedule.tasks.is_empty());
    }

    #[test]
    fn test_load_schedule_whitespace_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file, "  \n  \t  ").expect("Failed to write whitespace");
        temp_file.flush().expect("Failed to flush");

        let path = temp_file.path().to_path_buf();
        let result = load_schedule(&path);

        assert!(result.is_ok());
        let schedule = result.unwrap();
        assert!(schedule.tasks.is_empty()); // Should also return default empty schedule
    }

    #[test]
    fn test_load_schedule_invalid_json() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file, "{{invalid json data, missing quotes}}").expect(
            "Failed to write invalid JSON"
        );
        temp_file.flush().expect("Failed to flush");

        let path = temp_file.path().to_path_buf();
        let result = load_schedule(&path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        if let VestaboardError::JsonError(_) = err {
            assert!(true);
        } else {
            panic!("Expected VestaboardError::JsonError");
        }
    }

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
