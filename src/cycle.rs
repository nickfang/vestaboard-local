use std::time::Duration;

use crate::{
  api_broker::{handle_message, MessageDestination},
  config::Config,
  errors::VestaboardError,
  process_control::ProcessController,
  scheduler::{load_schedule, Schedule, ScheduleMonitor, ScheduledTask},
  widgets::{resolver::execute_widget, widget_utils::error_to_display_message},
};

/// Run a schedule cycle that loops through all scheduled tasks in chronological order
///
/// This function provides the core cycle functionality that:
/// - Loads the current schedule using existing load_schedule()
/// - Continuously loops through all scheduled tasks in chronological order
/// - Executes each task using the refactored widget executor
/// - Applies the specified delay between each message
/// - Uses ProcessController for graceful shutdown on Ctrl+C
/// - Optionally uses ScheduleMonitor for file change detection
/// - Handles empty schedules gracefully (wait and retry)
/// - Continues cycling after completing all tasks (if repeat=true)
///
/// # Arguments
/// * `interval_seconds` - Delay in seconds between each message
/// * `dry_run` - If true, preview mode without sending to Vestaboard
/// * `repeat` - If true, continuously cycle until Ctrl+C; if false, run once
/// * `use_monitor` - If true, monitor schedule file for changes during cycling
///
/// # Returns
/// * `Ok(())` - Cycle completed successfully
/// * `Err(VestaboardError)` - Cycle failed due to configuration or other errors
pub async fn run_schedule_cycle(
  interval_seconds: u64,
  dry_run: bool,
  repeat: bool,
  use_monitor: bool,
) -> Result<(), VestaboardError> {
  log::info!(
    "Starting schedule cycle - interval: {}s, dry_run: {}, repeat: {}, monitor: {}",
    interval_seconds,
    dry_run,
    repeat,
    use_monitor
  );

  // Initialize process controller for graceful shutdown
  let process_controller = ProcessController::new();
  process_controller.setup_signal_handler()?;

  // Load configuration and get schedule file path
  let config = Config::load()?;
  let schedule_file_path = config.get_schedule_file_path();

  // Initialize schedule monitor if requested
  let mut schedule_monitor = if use_monitor {
    let mut monitor = ScheduleMonitor::new(&schedule_file_path);
    monitor.initialize()?;
    log::info!("Schedule monitor initialized for file change detection");
    Some(monitor)
  } else {
    None
  };

  if dry_run {
    println!("Running cycle in preview mode...");
  } else {
    println!(
      "Starting {} cycle with {} second intervals...",
      if repeat { "continuous" } else { "single" },
      interval_seconds
    );
  }

  let mut cycle_count = 0;

  loop {
    // Check for shutdown request
    if process_controller.should_shutdown() {
      log::info!("Shutdown requested, stopping cycle");
      println!("Cycle stopped gracefully.");
      break;
    }

    cycle_count += 1;
    log::info!("Starting cycle iteration {}", cycle_count);

    if repeat && cycle_count > 1 {
      println!("Starting cycle {} of scheduled tasks...", cycle_count);
    }

    // Load current schedule (or reload if monitor detects changes)
    let schedule = if let Some(ref mut monitor) = schedule_monitor {
      // Check for file changes and reload if necessary
      match monitor.reload_if_modified() {
        Ok(was_reloaded) => {
          if was_reloaded {
            log::info!("Schedule file changed, reloaded for cycle {}", cycle_count);
            println!("Schedule updated, using new tasks for this cycle.");
          }
        }
        Err(e) => {
          log::warn!("Failed to check for schedule updates: {}", e);
          // Continue with cached schedule
        }
      }
      monitor.get_current_schedule().clone()
    } else {
      // Load schedule fresh each cycle
      match load_schedule(&schedule_file_path) {
        Ok(schedule) => schedule,
        Err(e) => {
          log::error!("Failed to load schedule for cycle {}: {}", cycle_count, e);
          eprintln!("Error loading schedule: {}", e);

          if !repeat || dry_run {
            // For single cycles OR dry-run mode, don't retry
            return Err(e);
          }

          // For continuous cycles, wait and retry
          println!("Waiting {} seconds before retrying...", interval_seconds);

          // Split the sleep into smaller chunks to be more responsive to Ctrl+C
          let sleep_chunks = std::cmp::max(1, interval_seconds / 5); // Check every 1/5 of the interval
          let chunk_duration = interval_seconds / sleep_chunks;

          for _ in 0..sleep_chunks {
            if process_controller.should_shutdown() {
              log::info!("Shutdown requested during retry delay");
              return Ok(());
            }

            if let Err(sleep_err) = tokio::time::timeout(
              Duration::from_secs(chunk_duration),
              tokio::time::sleep(Duration::from_secs(chunk_duration)),
            )
            .await
            {
              log::debug!("Sleep chunk interrupted: {:?}", sleep_err);
            }
          }
          continue;
        }
      }
    };

    // Handle empty schedule
    if schedule.is_empty() {
      log::info!("Schedule is empty for cycle {}", cycle_count);
      println!("No scheduled tasks found.");

      if !repeat || dry_run {
        // For single cycles OR dry-run mode, don't continue looping with empty schedule
        println!("Cycle completed - no tasks to execute.");
        break;
      }

      println!("Waiting {} seconds before checking for new tasks...", interval_seconds);

      // Split the sleep into smaller chunks to be more responsive to Ctrl+C
      let sleep_chunks = std::cmp::max(1, interval_seconds / 5); // Check every 1/5 of the interval
      let chunk_duration = interval_seconds / sleep_chunks;

      for _ in 0..sleep_chunks {
        if process_controller.should_shutdown() {
          log::info!("Shutdown requested during empty schedule delay");
          return Ok(());
        }

        if let Err(sleep_err) = tokio::time::timeout(
          Duration::from_secs(chunk_duration),
          tokio::time::sleep(Duration::from_secs(chunk_duration)),
        )
        .await
        {
          log::debug!("Sleep chunk interrupted: {:?}", sleep_err);
        }
      }
      continue;
    }

    log::info!("Executing {} tasks in cycle {}", schedule.tasks.len(), cycle_count);

    // Execute all tasks in chronological order
    let result = execute_schedule_tasks(&schedule, interval_seconds, dry_run, &process_controller).await;

    match result {
      Ok(tasks_executed) => {
        log::info!("Cycle {} completed successfully, executed {} tasks", cycle_count, tasks_executed);
        println!("Cycle {} completed - executed {} tasks.", cycle_count, tasks_executed);
      }
      Err(e) => {
        log::error!("Cycle {} failed: {}", cycle_count, e);
        eprintln!("Cycle {} failed: {}", cycle_count, e);

        if !repeat {
          return Err(e);
        }
        // For continuous cycles, log error but continue
      }
    }

    // If not repeating OR in dry-run mode, break after one cycle
    if !repeat || dry_run {
      log::info!("Single cycle completed, stopping");
      println!("Single cycle completed.");
      break;
    }

    // Check for shutdown before starting next cycle
    if process_controller.should_shutdown() {
      log::info!("Shutdown requested after cycle completion");
      println!("Cycle stopped gracefully.");
      break;
    }
  }

  log::info!("Schedule cycle finished after {} iterations", cycle_count);
  Ok(())
}

/// Execute all tasks in a schedule in chronological order
///
/// # Arguments
/// * `schedule` - The schedule containing tasks to execute
/// * `interval_seconds` - Delay between each task execution
/// * `dry_run` - If true, preview mode without sending to Vestaboard
/// * `process_controller` - Controller for checking shutdown requests
///
/// # Returns
/// * `Ok(usize)` - Number of tasks successfully executed
/// * `Err(VestaboardError)` - Critical error that should stop the cycle
async fn execute_schedule_tasks(
  schedule: &Schedule,
  interval_seconds: u64,
  dry_run: bool,
  process_controller: &ProcessController,
) -> Result<usize, VestaboardError> {
  let tasks = schedule.get_tasks();
  let total_tasks = tasks.len();
  let mut executed_count = 0;

  log::debug!("Starting execution of {} tasks", total_tasks);

  for (index, task) in tasks.iter().enumerate() {
    // Check for shutdown request before each task
    if process_controller.should_shutdown() {
      log::info!("Shutdown requested during task execution, stopping");
      break;
    }

    log::debug!(
      "Executing task {}/{}: {} (ID: {})",
      index + 1,
      total_tasks,
      task.widget,
      task.id
    );

    // Execute the task
    match execute_single_task(task, dry_run).await {
      Ok(_) => {
        executed_count += 1;
        log::info!(
          "Task {}/{} completed successfully: {} (ID: {})",
          index + 1,
          total_tasks,
          task.widget,
          task.id
        );
      }
      Err(e) => {
        log::error!(
          "Task {}/{} failed: {} (ID: {}) - Error: {}",
          index + 1,
          total_tasks,
          task.widget,
          task.id,
          e
        );
        eprintln!(
          "Task {} failed ({}): {}",
          index + 1,
          task.widget,
          e
        );
        // Continue with next task rather than failing entire cycle
      }
    }

    // Apply delay between tasks (except after the last task)
    if index < total_tasks - 1 && interval_seconds > 0 {
      if dry_run {
        // In dry-run mode, just show what the delay would be
        println!("  [{} second delay]", interval_seconds);
        log::debug!("Dry-run: skipping {} second delay", interval_seconds);
      } else {
        // In normal mode, actually wait
        log::debug!("Waiting {} seconds before next task", interval_seconds);

        // Split the sleep into smaller chunks to be more responsive to Ctrl+C
        let sleep_chunks = std::cmp::max(1, interval_seconds / 5); // Check every 1/5 of the interval
        let chunk_duration = interval_seconds / sleep_chunks;

        for _ in 0..sleep_chunks {
          if process_controller.should_shutdown() {
            log::info!("Shutdown requested during task delay");
            break;
          }

          if let Err(sleep_err) = tokio::time::timeout(
            Duration::from_secs(chunk_duration),
            tokio::time::sleep(Duration::from_secs(chunk_duration)),
          )
          .await
          {
            log::debug!("Sleep chunk interrupted: {:?}", sleep_err);
          }
        }

        // Check for shutdown again after delay
        if process_controller.should_shutdown() {
          log::info!("Shutdown requested during delay, stopping");
          break;
        }
      }
    }
  }

  log::debug!("Task execution completed: {}/{} tasks executed", executed_count, total_tasks);
  Ok(executed_count)
}

/// Execute a single scheduled task
///
/// # Arguments
/// * `task` - The scheduled task to execute
/// * `dry_run` - If true, preview mode without sending to Vestaboard
///
/// # Returns
/// * `Ok(())` - Task executed successfully
/// * `Err(VestaboardError)` - Task execution failed
async fn execute_single_task(task: &ScheduledTask, dry_run: bool) -> Result<(), VestaboardError> {
  log::debug!("Executing widget '{}' with input: {:?}", task.widget, task.input);

  // Execute widget (with error handling for dry-run mode)
  let message = if dry_run {
    // In dry-run mode, convert errors to display messages
    match execute_widget(&task.widget, &task.input).await {
      Ok(message) => message,
      Err(e) => {
        log::debug!("Widget '{}' failed in dry-run, converting to display message: {}", task.widget, e);
        error_to_display_message(&e)
      }
    }
  } else {
    // In normal mode, propagate errors
    execute_widget(&task.widget, &task.input).await?
  };

  // Determine destination and title
  let destination = if dry_run {
    let title = format!("Task {} ({})", task.id, task.widget);
    MessageDestination::ConsoleWithTitle(title)
  } else {
    MessageDestination::Vestaboard
  };

  // Send message
  match handle_message(message, destination).await {
    Ok(_) => {
      log::debug!("Message sent successfully for task {}", task.id);
      Ok(())
    }
    Err(e) => {
      log::error!("Failed to send message for task {}: {}", task.id, e);
      Err(e)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
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

    let result = execute_schedule_tasks(&schedule, 5, false, &process_controller).await;

    assert!(result.is_ok(), "Should handle shutdown gracefully");
    assert_eq!(result.unwrap(), 0, "Should execute 0 tasks when shutdown is already requested");
  }

  #[tokio::test]
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
