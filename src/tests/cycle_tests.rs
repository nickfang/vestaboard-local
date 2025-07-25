
#[cfg(test)]
mod tests {
  use super::*;
  use crate::cycle::{execute_single_task, execute_schedule_tasks};
  use crate::process_control::ProcessController;
use crate::scheduler::{Schedule, ScheduledTask};
  use chrono::Utc;
  use serde_json::json;
  use std::time::Duration;

  /// Helper function to create a test scheduled task
  fn create_test_task(widget: &str, input: serde_json::Value) -> ScheduledTask {
    ScheduledTask::new(Utc::now(), widget.to_string(), input)
  }

  /// Helper function to create a test schedule with multiple tasks
  fn create_test_schedule() -> Schedule {
    let mut schedule = Schedule::default();

    // Add some test tasks in non-chronological order to test sorting
    // Use lowercase text as per Vestaboard requirements
    schedule.add_task(create_test_task("text", json!("hello world")));
    schedule.add_task(create_test_task("clear", json!(null)));
    schedule.add_task(create_test_task("text", json!("goodbye")));

    schedule
  }

  #[tokio::test]
  async fn test_execute_single_task_text_dry_run() {
    let task = create_test_task("text", json!("test message"));

    let result = execute_single_task(&task, true).await;
    assert!(result.is_ok(), "Text task should execute successfully in dry-run mode");
  }

  #[tokio::test]
  async fn test_execute_single_task_clear_dry_run() {
    let task = create_test_task("clear", json!(null));

    let result = execute_single_task(&task, true).await;
    assert!(result.is_ok(), "Clear task should execute successfully in dry-run mode");
  }

  #[tokio::test]
  async fn test_execute_single_task_invalid_widget() {
    let task = create_test_task("invalid-widget", json!("test"));

    let result = execute_single_task(&task, true).await;
    // In dry-run mode, invalid widgets should still "succeed" by showing error message
    assert!(result.is_ok(), "Invalid widget should be handled gracefully in dry-run mode");
  }

  #[tokio::test]
  async fn test_execute_schedule_tasks_empty_schedule() {
    let schedule = Schedule::default();
    let process_controller = ProcessController::new();

    let result = execute_schedule_tasks(&schedule, 1, true, &process_controller).await;
    assert!(result.is_ok(), "Empty schedule should execute successfully");
    assert_eq!(result.unwrap(), 0, "Empty schedule should execute 0 tasks");
  }

  #[tokio::test]
  async fn test_execute_schedule_tasks_with_tasks() {
    let schedule = create_test_schedule();
    let process_controller = ProcessController::new();

    let result = execute_schedule_tasks(&schedule, 0, true, &process_controller).await;
    assert!(result.is_ok(), "Schedule with tasks should execute successfully");
    assert_eq!(result.unwrap(), 3, "Should execute all 3 tasks");
  }

  #[tokio::test]
  async fn test_execute_schedule_tasks_with_shutdown() {
    let schedule = create_test_schedule();
    let process_controller = ProcessController::new();

    // Request shutdown immediately
    process_controller.request_shutdown();

    let result = execute_schedule_tasks(&schedule, 0, true, &process_controller).await;
    assert!(result.is_ok(), "Should handle shutdown gracefully");
    assert_eq!(result.unwrap(), 0, "Should execute 0 tasks when shutdown requested");
  }

  #[tokio::test]
  async fn test_execute_schedule_tasks_with_delay() {
    let mut schedule = Schedule::default();
    schedule.add_task(create_test_task("text", json!("first")));
    schedule.add_task(create_test_task("text", json!("second")));

    let process_controller = ProcessController::new();
    let start_time = std::time::Instant::now();

    // Use a small delay to avoid making tests too slow
    // In dry-run mode, delays should be skipped so this should be fast
    let result = execute_schedule_tasks(&schedule, 1, true, &process_controller).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok(), "Schedule with delay should execute successfully");
    assert_eq!(result.unwrap(), 2, "Should execute both tasks");
    // In dry-run mode, should NOT wait - should complete quickly
    assert!(elapsed < Duration::from_secs(1), "Should skip delays in dry-run mode");
  }

  #[tokio::test]
  #[ignore]
  // This is ignored because it actually sends to the vestaboard.  Need to figure out how to mock the api calls.
  async fn test_execute_schedule_tasks_with_delay_normal_mode() {
    let mut schedule = Schedule::default();
    schedule.add_task(create_test_task("text", json!("first")));
    schedule.add_task(create_test_task("text", json!("second")));

    let process_controller = ProcessController::new();
    let start_time = std::time::Instant::now();

    // Use a small delay - in normal mode (dry_run=false) this should actually wait
    let result = execute_schedule_tasks(&schedule, 1, false, &process_controller).await;
    let elapsed = start_time.elapsed();

    // Note: This test will fail the tasks due to API calls, but that's expected
    // We just want to verify the delay behavior
    assert!(result.is_ok(), "Schedule with delay should execute successfully");
    // Should take at least 1 second due to delay between tasks in normal mode
    assert!(elapsed >= Duration::from_secs(1), "Should respect delay between tasks in normal mode");
  }

  #[tokio::test]
  async fn test_dry_run_never_repeats() {
    // Test that dry-run mode always runs only once, even with repeat=true
    let schedule = create_test_schedule();
    let process_controller = ProcessController::new();

    // This simulates what would happen in run_schedule_cycle with dry_run=true
    // The key test is that we only execute once
    let result = execute_schedule_tasks(&schedule, 0, true, &process_controller).await;

    assert!(result.is_ok(), "Dry-run should execute successfully");
    assert_eq!(result.unwrap(), 3, "Should execute all 3 tasks in dry-run");

    // In actual run_schedule_cycle, the dry_run check would prevent looping
    // This test verifies that the task execution part works correctly
  }

  #[tokio::test]
  async fn test_shutdown_during_task_execution() {
    let schedule = create_test_schedule();
    let process_controller = ProcessController::new();

    // Request shutdown before starting
    process_controller.request_shutdown();

    let result = execute_schedule_tasks(&schedule, 5, true, &process_controller).await;

    assert!(result.is_ok(), "Should handle shutdown gracefully");
    assert_eq!(result.unwrap(), 0, "Should execute 0 tasks when shutdown is already requested");
  }

  #[tokio::test]
  #[ignore]
  async fn test_shutdown_responsiveness_with_chunked_sleep() {
    let mut schedule = Schedule::default();
    schedule.add_task(create_test_task("text", json!("first")));
    schedule.add_task(create_test_task("text", json!("second")));

    let process_controller = ProcessController::new();

    // Start execution in a background task
    let controller_clone = process_controller.clone();
    let task_handle = tokio::spawn(async move {
      execute_schedule_tasks(&schedule, 10, false, &controller_clone).await
    });

    // Wait a bit, then request shutdown
    tokio::time::sleep(Duration::from_millis(100)).await;
    process_controller.request_shutdown();

    // The task should complete quickly due to shutdown request
    let start_time = std::time::Instant::now();
    let result = task_handle.await.unwrap();
    let elapsed = start_time.elapsed();

    assert!(result.is_ok(), "Should handle shutdown gracefully");
    // Should complete much faster than the 10 second delay due to shutdown
    assert!(elapsed < Duration::from_secs(5), "Should respond to shutdown quickly");
  }

  #[tokio::test]
  async fn test_empty_schedule_handling() {
    let schedule = Schedule::default();
    let process_controller = ProcessController::new();

    let result = execute_schedule_tasks(&schedule, 10, true, &process_controller).await;

    assert!(result.is_ok(), "Empty schedule should be handled gracefully");
    assert_eq!(result.unwrap(), 0, "Empty schedule should execute 0 tasks");
  }

  #[tokio::test]
  async fn test_partial_task_failure() {
    let mut schedule = Schedule::default();
    schedule.add_task(create_test_task("text", json!("valid text")));
    schedule.add_task(create_test_task("invalid-widget", json!("test")));
    schedule.add_task(create_test_task("clear", json!(null)));

    let process_controller = ProcessController::new();

    // In dry-run mode, all tasks should "succeed" (errors converted to display messages)
    let result = execute_schedule_tasks(&schedule, 0, true, &process_controller).await;

    assert!(result.is_ok(), "Should handle mixed valid/invalid tasks in dry-run");
    assert_eq!(result.unwrap(), 3, "All tasks should 'succeed' in dry-run mode");
  }

  // Note: Integration tests for run_schedule_cycle would require:
  // - File system mocking for schedule loading
  // - Process controller mocking for signal handling
  // - More complex test setup
  // These would be better placed in integration tests or end-to-end tests
}
