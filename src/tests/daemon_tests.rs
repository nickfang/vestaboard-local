#[path = "../daemon.rs"]
mod daemon;

use daemon::{ load_schedule, get_file_mod_time, execute_task, run_daemon };
use std::path::PathBuf;

use crate::{ errors::VestaboardError, scheduler::ScheduledTask };

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::{ Schedule, ScheduledTask };
    use crate::errors::VestaboardError;
    use std::path::PathBuf;

    #[test]
    fn test_load_schedule() {
        let path = PathBuf::from("non_existent_schedule.json");
        let result = load_schedule(&path);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e, VestaboardError::Other("load_schedule() not implemented.".to_string()));
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
