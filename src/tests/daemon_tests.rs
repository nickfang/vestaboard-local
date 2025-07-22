#[path = "../daemon.rs"]
mod daemon;

#[cfg(test)]
mod tests {
  use super::*;
  use crate::errors::VestaboardError;
  use crate::scheduler::ScheduledTask;

  use daemon::{execute_task, run_daemon};

  // Note: get_file_mod_time functionality is now tested via ScheduleMonitor tests
  // in scheduler_tests.rs since it was moved to ScheduleMonitor

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
      },
      _ => panic!("Expected WidgetError"),
    }
  }

  #[test]
  #[ignore]
  fn test_run_daemon() {
    // this currently cannot be tested because it runs indefinitely and requires a Ctrl+C signal to stop
    // TODO: refactor to allow testing, or mock the Ctrl+C signal
    let _ = run_daemon();
    assert!(true)
  }
}
