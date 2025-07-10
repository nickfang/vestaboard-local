use crate::api_broker::{display_message, validate_message_content};
use crate::config::Config;
use crate::datetime::is_or_before;
use crate::errors::VestaboardError;
use crate::scheduler::{load_schedule, Schedule, ScheduledTask};
use crate::widgets::sat_words::get_sat_word;
use crate::widgets::text::{get_text, get_text_from_file};
use crate::widgets::weather::get_weather;
use crate::widgets::widget_utils::error_to_display_message;

use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

#[allow(dead_code)] // Not dead code, but the compiler doesn't know that.
static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);
#[allow(dead_code)] // Not dead code, but the compiler doesn't know that.
const CHECK_INTERVAL_SECONDS: u64 = 3;

pub fn get_file_mod_time(path: &PathBuf) -> Result<SystemTime, VestaboardError> {
  log::trace!("Getting file modification time for: {}", path.display());

  fs::metadata(path)
    .and_then(|meta| meta.modified())
    .map_err(|e| {
      log::error!("Error getting mod time for {}: {}", path.display(), e);
      eprintln!("Error getting mod time for {}: {}", path.display(), e);
      VestaboardError::io_error(e, &format!("getting mod time for {}", path.display()))
    })
}

pub async fn execute_task(task: &ScheduledTask) -> Result<(), VestaboardError> {
  let start_time = std::time::Instant::now();
  log::info!("Executing scheduled task: {} ({})", task.widget, task.id);
  log::debug!("Task details: {:?}", task);

  println!("Executing task: {:?}", task);

  let message_result = match task.widget.as_str() {
    "text" => {
      log::info!("Executing Text widget with input: {:?}", task.input);
      println!("Executing Text widget with input: {:?}", task.input);
      get_text(task.input.as_str().unwrap_or(""))
    },
    "file" => {
      log::info!("Executing File widget with input: {:?}", task.input);
      println!("Executing File widget with input: {:?}", task.input);
      get_text_from_file(PathBuf::from(task.input.as_str().unwrap_or("")))
    },
    "weather" => {
      log::info!("Executing Weather widget");
      println!("Executing Weather widget");
      get_weather().await
    },
    "sat-word" => {
      log::info!("Executing SAT Word widget");
      println!("Executing SAT Word widget");
      get_sat_word()
    },
    _ => {
      let error = VestaboardError::widget_error(
        &task.widget,
        &format!("Unknown widget type: {}", task.widget),
      );
      log::error!("Unknown widget type '{}' in task {}", task.widget, task.id);
      return Err(error);
    },
  };

  let duration = start_time.elapsed();
  let message = match message_result {
    Ok(msg) => {
      log::info!(
        "Widget '{}' completed successfully in {:?}",
        task.widget,
        duration
      );
      msg
    },
    Err(e) => {
      log::error!(
        "Widget '{}' failed after {:?}: {}",
        task.widget,
        duration,
        e
      );
      eprintln!("Widget error: {}", e);
      error_to_display_message(&e)
    },
  };

  // Validate message content before sending
  if let Err(validation_error) = validate_message_content(&message) {
    log::error!(
      "Message validation failed for task {}: {}",
      task.id,
      validation_error
    );
    eprintln!("Validation error: {}", validation_error);
    display_message(error_to_display_message(&VestaboardError::other(
      &validation_error,
    )))
    .await;
    return Ok(()); // Continue daemon operation even after validation error
  }

  log::info!("Sending message to Vestaboard for task {}", task.id);
  display_message(message).await;
  log::info!("Task {} completed successfully", task.id);
  Ok(())
}
// Err(VestaboardError::Other("execute_task() not implemented".to_string()));

pub async fn run_daemon() -> Result<(), VestaboardError> {
  log::info!("Starting Vestaboard daemon");
  println!("Starting daemon...");
  println!("Press Ctrl+C to stop the daemon.");

  // handle ctrl+c
  ctrlc::set_handler(move || {
    log::info!("Ctrl+C received, initiating shutdown");
    println!("\nCtrl+C received, shutting down...");
    SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
  })
  .expect("Error setting Ctrl-C handler");

  let config = Config::load().map_err(|e| {
    eprintln!("Error loading config: {:?}", e);
    e
  })?;
  let schedule_path = config.get_schedule_file_path();
  let check_interval = Duration::from_secs(CHECK_INTERVAL_SECONDS);

  log::info!("Using schedule file: {}", schedule_path.display());
  log::info!("Check interval: {} seconds", CHECK_INTERVAL_SECONDS);

  let mut current_schedule = load_schedule(&schedule_path).unwrap_or_else(|e| {
    log::warn!(
      "Error loading initial schedule: {:?}, using empty schedule",
      e
    );
    eprintln!("Error loading initial schedule: {:?}.", e);
    Schedule::default()
  });

  log::info!(
    "Initial schedule loaded with {} tasks",
    current_schedule.tasks.len()
  );
  println!(
    "Initial schedule loaded with {} tasks.",
    current_schedule.tasks.len()
  );

  let mut last_mod_time = get_file_mod_time(&schedule_path).unwrap_or_else(|e| {
    log::debug!(
      "Could not get initial file mod time: {}, using UNIX_EPOCH",
      e
    );
    SystemTime::UNIX_EPOCH
  });
  let mut executed_task_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

  log::info!("Daemon started successfully, monitoring schedule");
  println!("Daemon started. Monitoring schedule...");

  loop {
    if SHUTDOWN_FLAG.load(Ordering::SeqCst) {
      log::info!("Shutdown flag detected, stopping daemon");
      println!("Daemon shutting down...");
      break;
    }

    log::trace!("Daemon loop iteration starting");

    // Reload schedule if the file has been modified
    match get_file_mod_time(&schedule_path) {
      Ok(current_mod_time) => {
        if current_mod_time > last_mod_time {
          log::info!("Schedule file modified, reloading schedule");
          println!("Schedule file modified. Reloading schedule...");
          match load_schedule(&schedule_path) {
            Ok(new_schedule) => {
              let old_count = current_schedule.tasks.len();
              let new_count = new_schedule.tasks.len();
              current_schedule = new_schedule;
              last_mod_time = current_mod_time;
              log::info!(
                "Successfully reloaded schedule (tasks: {} -> {})",
                old_count,
                new_count
              );
              println!("Successfully reloaded schedule.");
            },
            Err(e) => {
              log::error!("Error reloading schedule: {:?}", e);
              eprintln!("Error reloading schedule: {:?}", e);
            },
          }
        }
      },
      Err(e) => {
        log::debug!("Error getting file modification time: {:?}", e);
        eprintln!("Error getting file modification time: {:?}", e);
      },
    }

    let now = Utc::now();
    let mut tasks_to_execute = Vec::new();

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
