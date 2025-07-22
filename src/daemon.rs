use crate::api_broker::{handle_message, MessageDestination};
use crate::config::Config;
use crate::datetime::is_or_before;
use crate::errors::VestaboardError;
use crate::process_control::ProcessController;
use crate::scheduler::{ScheduleMonitor, ScheduledTask};
use crate::widgets::resolver::execute_widget;

use chrono::Utc;
use std::thread;
use std::time::Duration;

const CHECK_INTERVAL_SECONDS: u64 = 3;

pub async fn execute_task(task: &ScheduledTask) -> Result<(), VestaboardError> {
  log::info!("Executing scheduled task: {} ({})", task.widget, task.id);
  log::debug!("Task details: {:?}", task);

  println!("Executing task: {:?}", task);

  // Execute the widget to get the raw message. Dry-run is handled in the scheduler.
  let message = execute_widget(&task.widget, &task.input).await?;

  match handle_message(message.clone(), MessageDestination::Vestaboard).await {
    Ok(_) => {},
    Err(e) => {
      log::error!("Failed to send message to Vestaboard: {}", e);
      eprintln!("Error sending message to Vestaboard: {}", e);
    },
  }
  log::info!("Task {} completed successfully", task.id);
  Ok(())
}
// Err(VestaboardError::Other("execute_task() not implemented".to_string()));

pub async fn run_daemon() -> Result<(), VestaboardError> {
  log::info!("Starting Vestaboard daemon");
  println!("Starting daemon...");

  // Create and setup process controller for graceful shutdown
  let process_controller = ProcessController::new();
  process_controller.setup_signal_handler().map_err(|e| {
    eprintln!("Error setting up signal handler: {:?}", e);
    e
  })?;

  let config = Config::load().map_err(|e| {
    eprintln!("Error loading config: {:?}", e);
    e
  })?;
  let schedule_path = config.get_schedule_file_path();
  let check_interval = Duration::from_secs(CHECK_INTERVAL_SECONDS);

  log::info!("Using schedule file: {}", schedule_path.display());
  log::info!("Check interval: {} seconds", CHECK_INTERVAL_SECONDS);

  // Initialize schedule monitor
  let mut schedule_monitor = ScheduleMonitor::new(&schedule_path);

  // Initialize the monitor (loads initial schedule)
  if let Err(e) = schedule_monitor.initialize() {
    log::warn!("Failed to initialize schedule monitor: {}", e);
    eprintln!("Error loading initial schedule: {:?}.", e);
    // Continue running even if initial schedule load fails
  }

  log::info!(
    "Initial schedule loaded with {} tasks",
    schedule_monitor.get_current_schedule().tasks.len()
  );
  println!(
    "Initial schedule loaded with {} tasks.",
    schedule_monitor.get_current_schedule().tasks.len()
  );

  let mut executed_task_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

  log::info!("Daemon started successfully, monitoring schedule");
  println!("Daemon started. Monitoring schedule...");

  loop {
    if process_controller.should_shutdown() {
      log::info!("Shutdown request detected, stopping daemon");
      println!("Daemon shutting down...");
      break;
    }

    log::trace!("Daemon loop iteration starting");

    // Check for schedule file updates
    match schedule_monitor.reload_if_modified() {
      Ok(true) => {
        log::info!("Schedule file updated and reloaded");
        println!("Successfully reloaded schedule.");
      }
      Ok(false) => {
        log::trace!("No schedule file changes detected");
      }
      Err(e) => {
        log::error!("Error checking for schedule updates: {}", e);
        eprintln!("Error getting file modification time: {:?}", e);
        // Continue running even if file monitoring fails
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
      log::info!(
        "Found {} task(s) ready for execution",
        tasks_to_execute.len()
      );
      match execute_task(task).await {
        Ok(_) => {
          log::info!(
            "Task execution successful, marking {} task(s) as executed",
            tasks_to_execute.len()
          );
          for task in &tasks_to_execute {
            executed_task_ids.insert(task.id.clone());
          }
        },
        Err(e) => {
          log::error!("Error executing task {}: {:?}", task.id, e);
          eprintln!("Error executing task: {:?}", e);
          // In daemon mode, we continue running even after task execution errors
          // The error should have been displayed on the Vestaboard by execute_task
        },
      }
    } else {
      log::trace!("No tasks ready for execution");
    }

    log::trace!("Daemon sleeping for {:?}", check_interval);
    thread::sleep(check_interval);
  }

  log::info!("Daemon shutdown complete");
  println!("Shutdown complete.");
  Ok(())
}
