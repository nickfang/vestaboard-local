use crate::api_broker::{ handle_message, MessageDestination };
use crate::cli_display::{ print_error, print_progress, print_success, print_warning };
use crate::config::Config;
use crate::datetime::is_or_before;
use crate::errors::VestaboardError;
use crate::process_control::ProcessController;
use crate::scheduler::{ ScheduleMonitor, ScheduledTask };
use crate::widgets::resolver::execute_widget;

use chrono::Utc;
use std::thread;
use std::time::Duration;

pub async fn execute_task(task: &ScheduledTask) -> Result<(), VestaboardError> {
  log::info!("Executing scheduled task: {} ({})", task.widget, task.id);
  log::debug!("Task details: {:?}", task);

  print_progress(&format!("Executing task {} ({})...", task.id, task.widget));

  // Execute the widget to get the raw message. Dry-run is handled in the scheduler.
  let message = execute_widget(&task.widget, &task.input).await?;

  // send_codes() will print "Sending message to Vestaboard..." so we don't need to print it here
  match handle_message(message.clone(), MessageDestination::Vestaboard).await {
    Ok(_) => {}
    Err(e) => {
      log::error!("Failed to send message to Vestaboard: {}", e);
      print_error(&e.to_user_message());
    }
  }
  log::info!("Task {} completed successfully", task.id);
  print_success(&format!("Task {} completed successfully", task.id));
  Ok(())
}
// Err(VestaboardError::Other("execute_task() not implemented".to_string()));

pub async fn run_daemon() -> Result<(), VestaboardError> {
  log::info!("Starting Vestaboard daemon");

  // Deprecation warning
  print_warning("'vbl daemon' is deprecated. Use 'vbl schedule run' instead.");

  print_progress("Starting Vestaboard daemon...");

  // Create and setup process controller for graceful shutdown
  let process_controller = ProcessController::new();
  process_controller.setup_signal_handler().map_err(|e| {
    print_error(&format!("Error setting up signal handler: {:?}", e));
    e
  })?;

  let config = Config::load_silent().map_err(|e| {
    print_error(&e.to_user_message());
    e
  })?;

  let schedule_path = config.get_schedule_file_path();
  let check_interval_seconds = config.get_check_interval_seconds();
  let check_interval = Duration::from_secs(check_interval_seconds);

  log::info!("Using schedule file: {}", schedule_path.display());
  log::info!("Check interval: {} seconds", check_interval_seconds);

  let mut schedule_monitor = ScheduleMonitor::new(&schedule_path);

  // Initialize the monitor (loads initial schedule)
  let initial_task_count = match schedule_monitor.initialize() {
    Ok(_) => {
      let count = schedule_monitor.get_current_schedule().tasks.len();
      log::info!("Initial schedule loaded with {} tasks", count);
      count
    }
    Err(e) => {
      log::warn!("Failed to initialize schedule monitor: {}", e);
      print_warning(&format!("Could not load schedule: {}", e.to_user_message()));
      0
    }
  };

  let mut executed_task_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

  log::info!("Daemon started successfully, monitoring schedule");
  print_success(
    &format!(
      "Daemon started ({} tasks, checking every {}s)",
      initial_task_count,
      check_interval_seconds
    )
  );

  loop {
    if process_controller.should_shutdown() {
      log::info!("Shutdown request detected, stopping daemon");
      print_progress("Shutting down daemon...");
      break;
    }

    log::trace!("Daemon loop iteration starting");

    // Check for schedule file updates
    match schedule_monitor.reload_if_modified() {
      Ok(true) => {
        log::info!("Schedule file updated and reloaded");
        let task_count = schedule_monitor.get_current_schedule().tasks.len();
        print_success(&format!("Schedule reloaded ({} tasks)", task_count));
      }
      Ok(false) => {
        log::trace!("No schedule file changes detected");
      }
      Err(e) => {
        log::error!("Error checking for schedule updates: {}", e);
        print_warning(&e.to_user_message());
      }
    }

    let now = Utc::now();
    let mut tasks_to_execute = Vec::new();
    let current_schedule = schedule_monitor.get_current_schedule();

    for task in &current_schedule.tasks {
      if is_or_before(task.time, now) && !executed_task_ids.contains(&task.id) {
        tasks_to_execute.push(task.clone());
      }
    }

    if let Some(task) = tasks_to_execute.last() {
      log::info!("Found {} task(s) ready for execution", tasks_to_execute.len());
      match execute_task(task).await {
        Ok(_) => {
          log::info!(
            "Task execution successful, marking {} task(s) as executed",
            tasks_to_execute.len()
          );
          for task in &tasks_to_execute {
            executed_task_ids.insert(task.id.clone());
          }
        }
        Err(e) => {
          log::error!("Error executing task {}: {:?}", task.id, e);
          print_error(&format!("Error executing task {}: {}", task.id, e.to_user_message()));
          // In daemon mode, we continue running even after task execution errors
          // The error should have been displayed on the Vestaboard by execute_task
        }
      }
    } else {
      log::trace!("No tasks ready for execution");
    }

    log::trace!("Daemon sleeping for {:?}", check_interval);
    thread::sleep(check_interval);
  }

  log::info!("Daemon shutdown complete");
  print_success("Daemon shutdown complete");
  Ok(())
}
