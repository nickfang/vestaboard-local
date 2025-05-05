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
    use std::io::Write;

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
        let path = PathBuf::from("non_existent_file.json");
        let result = get_file_mod_time(&path);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(
                e,
                VestaboardError::Other("get_file_mod_time() not implemented".to_string())
            );
        }
    }

    #[test]
    fn test_execute_task() {
        let task = ScheduledTask {
            id: "test_task".to_string(),
            time: chrono::Utc::now(),
            widget: "TestWidget".to_string(),
            input: serde_json::Value::String("test_input".to_string()),
        };
        let result = execute_task(&task);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e, VestaboardError::Other("execute_task() not implemented".to_string()));
        }
    }

    #[test]
    fn test_run_daemon() {
        // this currently cannot be tested because it runs indefinitely and requires a Ctrl+C signal to stop
        // TODO: refactor to allow testing, or mock the Ctrl+C signal
        assert!(true)
    }
}
