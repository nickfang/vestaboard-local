use std::{
  fs,
  path::{Path, PathBuf},
  time::SystemTime,
};

use chrono::{DateTime, Local, Utc};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_broker::{handle_message, MessageDestination};
use crate::cli_display::{print_error, print_progress, print_success, print_warning};
use crate::widgets::resolver::execute_widget;
use crate::widgets::widget_utils;
use crate::{config::Config, errors::VestaboardError};

pub const CUSTOM_ALPHABET: &[char] = &[
  'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
  'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];
pub const ID_LENGTH: usize = 4;

fn generate_task_id() -> String {
  nanoid!(ID_LENGTH, CUSTOM_ALPHABET)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
  #[serde(default = "generate_task_id")]
  pub id: String,
  pub time: DateTime<Utc>,
  pub widget: String,
  pub input: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Schedule {
  #[serde(default)]
  pub tasks: Vec<ScheduledTask>,
}

impl ScheduledTask {
  pub fn new(time: DateTime<Utc>, widget: String, input: Value) -> Self {
    ScheduledTask {
      id: generate_task_id(),
      time,
      widget,
      input,
    }
  }
}

impl Schedule {
  pub fn add_task(&mut self, task: ScheduledTask) {
    let position = self
      .tasks
      .iter()
      .position(|t| t.time > task.time)
      .unwrap_or(self.tasks.len());
    self.tasks.insert(position, task);
  }
  pub fn remove_task(&mut self, id: &str) -> bool {
    let initial_len = self.tasks.len();
    self.tasks.retain(|task| task.id != id);
    self.tasks.len() < initial_len
  }
  pub fn get_tasks(&self) -> &[ScheduledTask] {
    &self.tasks
  }
  pub fn get_task(&self, id: &str) -> Option<&ScheduledTask> {
    self.tasks.iter().find(|task| task.id == id)
  }
  pub fn get_task_mut(&mut self, id: &str) -> Option<&mut ScheduledTask> {
    self.tasks.iter_mut().find(|task| task.id == id)
  }
  pub fn clear(&mut self) {
    self.tasks.clear();
  }
  pub fn is_empty(&self) -> bool {
    self.tasks.is_empty()
  }
}

/// Monitors schedule file for changes and manages schedule reloading
pub struct ScheduleMonitor {
  schedule_file_path: PathBuf,
  last_modified: Option<SystemTime>,
  current_schedule: Schedule,
}

impl ScheduleMonitor {
  /// Create a new schedule monitor for the given file path
  pub fn new<P: AsRef<Path>>(schedule_file_path: P) -> Self {
    Self {
      schedule_file_path: schedule_file_path.as_ref().to_path_buf(),
      last_modified: None,
      current_schedule: Schedule::default(),
    }
  }

  /// Initialize the monitor by loading the current schedule and tracking modification time
  pub fn initialize(&mut self) -> Result<(), VestaboardError> {
    log::info!("Initializing schedule monitor for: {:?}", self.schedule_file_path);

    // Load initial schedule and track modification time
    self.reload_schedule()?;
    Ok(())
  }

  /// Check if the schedule file has been modified since last check
  pub fn check_for_updates(&mut self) -> Result<bool, VestaboardError> {
    let current_mod_time = self.get_file_mod_time()?;

    match self.last_modified {
      Some(last_mod_time) if current_mod_time == last_mod_time => {
        // No change detected
        Ok(false)
      },
      _ => {
        // File has been modified or this is the first check
        log::info!("Schedule file modification detected");
        Ok(true)
      },
    }
  }

  /// Get the current cached schedule
  pub fn get_current_schedule(&self) -> &Schedule {
    &self.current_schedule
  }

  /// Reload the schedule if the file has been modified
  pub fn reload_if_modified(&mut self) -> Result<bool, VestaboardError> {
    if self.check_for_updates()? {
      self.reload_schedule()?;
      Ok(true)
    } else {
      Ok(false)
    }
  }

  /// Force reload the schedule from file
  pub fn reload_schedule(&mut self) -> Result<(), VestaboardError> {
    log::debug!("Reloading schedule from file: {:?}", self.schedule_file_path);

    // Update modification time first
    self.last_modified = Some(self.get_file_mod_time()?);

    // Load the schedule silently - caller manages output
    match load_schedule_silent(&self.schedule_file_path) {
      Ok(schedule) => {
        self.current_schedule = schedule;
        log::info!("Schedule reloaded successfully, {} tasks loaded", self.current_schedule.tasks.len());
        Ok(())
      },
      Err(e) => {
        log::error!("Failed to reload schedule: {}", e);
        // Keep the existing schedule on reload failure
        Err(e)
      },
    }
  }

  /// Get the file modification time, handling various error cases
  fn get_file_mod_time(&self) -> Result<SystemTime, VestaboardError> {
    match fs::metadata(&self.schedule_file_path) {
      Ok(metadata) => match metadata.modified() {
        Ok(modified_time) => {
          log::trace!("File modification time: {:?}", modified_time);
          Ok(modified_time)
        },
        Err(e) => {
          log::warn!("Could not get file modification time: {}", e);
          Err(VestaboardError::io_error(e, &format!("getting mod time for {}", self.schedule_file_path.display())))
        },
      },
      Err(e) => {
        match e.kind() {
          std::io::ErrorKind::NotFound => {
            log::debug!("Schedule file not found: {:?}", self.schedule_file_path);
            // Return a default time for non-existent files
            Ok(SystemTime::UNIX_EPOCH)
          },
          std::io::ErrorKind::PermissionDenied => {
            log::error!("Permission denied accessing schedule file: {:?}", self.schedule_file_path);
            Err(VestaboardError::io_error(
              e,
              &format!("accessing schedule file {}", self.schedule_file_path.display()),
            ))
          },
          _ => {
            log::error!("Error accessing schedule file metadata: {}", e);
            Err(VestaboardError::io_error(
              e,
              &format!("accessing schedule file {}", self.schedule_file_path.display()),
            ))
          },
        }
      },
    }
  }

  /// Get the path to the schedule file being monitored
  pub fn get_schedule_file_path(&self) -> &Path {
    &self.schedule_file_path
  }
}

#[allow(dead_code)]
pub fn save_schedule(schedule: &Schedule, path: &PathBuf) -> Result<(), VestaboardError> {
  save_schedule_internal(schedule, path, false)
}

/// Save schedule without printing progress messages (for internal operations)
pub fn save_schedule_silent(schedule: &Schedule, path: &PathBuf) -> Result<(), VestaboardError> {
  save_schedule_internal(schedule, path, true)
}

fn save_schedule_internal(schedule: &Schedule, path: &PathBuf, silent: bool) -> Result<(), VestaboardError> {
  log::debug!("Saving schedule with {} tasks to {}", schedule.tasks.len(), path.display());

  if !silent {
    print_progress("Saving schedule...");
  }

  match fs::write(path, serde_json::to_string_pretty(schedule).unwrap()) {
    Ok(_) => {
      log::info!("Schedule saved successfully to {}", path.display());
      Ok(())
    },
    Err(e) => {
      log::error!("Failed to save schedule to {}: {}", path.display(), e);
      let error = VestaboardError::io_error(e, &format!("saving schedule to {}", path.display()));
      print_error(&error.to_user_message());
      Err(error)
    },
  }
}

#[allow(dead_code)]
pub fn load_schedule(path: &PathBuf) -> Result<Schedule, VestaboardError> {
  load_schedule_internal(path, false)
}

/// Load schedule without printing progress messages (for internal operations)
pub fn load_schedule_silent(path: &PathBuf) -> Result<Schedule, VestaboardError> {
  load_schedule_internal(path, true)
}

fn load_schedule_internal(path: &PathBuf, silent: bool) -> Result<Schedule, VestaboardError> {
  log::debug!("Loading schedule from {}", path.display());

  match fs::read_to_string(&path) {
    Ok(content) => {
      if content.trim().is_empty() {
        log::info!("Schedule file {} is empty, creating new schedule", path.display());
        Ok(Schedule::default())
      } else {
        match serde_json::from_str::<Schedule>(&content) {
          Ok(mut schedule) => {
            schedule.tasks.sort_by_key(|task| task.time);
            log::info!("Successfully loaded {} tasks from schedule {}", schedule.tasks.len(), path.display());
            Ok(schedule)
          },
          Err(e) => {
            log::error!("Failed to parse schedule from {}: {}", path.display(), e);
            let error = VestaboardError::json_error(e, &format!("parsing schedule from {}", path.display()));
            print_error(&error.to_user_message());
            Err(error)
          },
        }
      }
    },
    Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
      log::info!("Schedule file {} not found, creating new schedule", path.display());
      let schedule = Schedule::default();
      // Save silently since this is an internal auto-create
      match save_schedule_silent(&schedule, path) {
        Ok(_) => {
          log::info!("New schedule created and saved to {}", path.display());
        },
        Err(e) => {
          log::error!("Error saving new schedule to {}: {:?}", path.display(), e);
        },
      }
      Ok(schedule)
    },
    Err(e) => {
      log::error!("Error reading schedule file {}: {}", path.display(), e);
      let error = VestaboardError::io_error(e, &format!("reading schedule from {}", path.display()));
      if !silent {
        print_error(&error.to_user_message());
      }
      Err(error)
    },
  }
}

pub fn add_task_to_schedule(time: DateTime<Utc>, widget: String, input: Value) -> Result<String, VestaboardError> {
  log::info!(
    "Adding task to schedule - time: {}, widget: {}, input: {}",
    time,
    widget,
    serde_json::to_string(&input).unwrap_or_else(|_| "invalid".to_string())
  );

  let config = Config::load_silent()?;
  let schedule_path = config.get_schedule_file_path();
  let mut schedule = load_schedule_silent(&schedule_path)?;

  let task = ScheduledTask::new(time, widget.clone(), input);
  let task_id = task.id.clone();
  schedule.add_task(task);

  match save_schedule_silent(&schedule, &schedule_path) {
    Ok(_) => {
      log::info!("Successfully added task {} for widget '{}'", task_id, widget);
      Ok(task_id)
    },
    Err(e) => {
      log::error!("Failed to save schedule after adding task: {}", e);
      Err(e)
    },
  }
}

pub fn remove_task_from_schedule(id: &str) -> Result<bool, VestaboardError> {
  log::info!("Removing task with ID: {}", id);

  let config = Config::load_silent()?;
  let schedule_path = config.get_schedule_file_path();
  let mut schedule = load_schedule_silent(&schedule_path)?;

  if schedule.get_task(id).is_none() {
    log::warn!("Task with ID {} not found in schedule", id);
    print_warning(&format!("Task {} not found", id));
    return Ok(false);
  }

  if schedule.remove_task(id) {
    match save_schedule_silent(&schedule, &schedule_path) {
      Ok(_) => {
        log::info!("Successfully removed task with ID {}", id);
        print_success(&format!("Task {} removed", id));
        Ok(true)
      },
      Err(e) => {
        log::error!("Failed to save schedule after removing task {}: {}", id, e);
        print_error(&e.to_user_message());
        Err(e)
      },
    }
  } else {
    log::error!("Failed to remove task with ID {}", id);
    Ok(false)
  }
}

pub fn clear_schedule() -> Result<usize, VestaboardError> {
  log::info!("Clearing all scheduled tasks");

  let config = Config::load_silent()?;
  let schedule_path = config.get_schedule_file_path();
  let mut schedule = load_schedule_silent(&schedule_path)?;
  let task_count = schedule.tasks.len();

  log::info!("Clearing schedule...");
  schedule.clear();

  match save_schedule_silent(&schedule, &schedule_path) {
    Ok(_) => {
      log::info!("Successfully cleared {} tasks from schedule", task_count);
      print_success(&format!("Schedule cleared ({} tasks removed)", task_count));
      Ok(task_count)
    },
    Err(e) => {
      log::error!("Failed to save schedule after clearing: {}", e);
      print_error(&e.to_user_message());
      Err(e)
    },
  }
}

pub fn list_schedule() -> Result<(), VestaboardError> {
  log::debug!("Listing scheduled tasks");

  let config = Config::load_silent()?;
  let schedule_path = config.get_schedule_file_path();
  let schedule = load_schedule_silent(&schedule_path)?;

  log::info!("Displaying {} scheduled tasks", schedule.tasks.len());

  if schedule.tasks.is_empty() {
    log::debug!("No scheduled tasks found");
    println!("Schedule is empty");
    return Ok(());
  }

  println!("Scheduled Tasks ({}):", schedule.tasks.len());
  println!("{:<6} | {:<22} | {:<15} | {}", "ID", "Time (Local)", "Widget", "Input");
  println!("{:-<80}", ""); // Separator line
  for task in schedule.tasks {
    let local_time = task.time.with_timezone(&Local::now().timezone());
    let formatted_time = local_time.format("%Y.%m.%d %I:%M %p").to_string();
    let input_str = serde_json::to_string(&task.input).unwrap_or_else(|_| "Invalid JSON".to_string());
    println!("{:<6} | {:<22} | {:<15} | {}", task.id, formatted_time, task.widget, input_str);
  }
  println!("{:-<80}", ""); // Footer separator line
  Ok(())
}

pub async fn preview_schedule() {
  log::debug!("Running schedule preview");

  let config = match Config::load_silent() {
    Ok(c) => c,
    Err(e) => {
      log::warn!("Failed to load config for schedule dry run: {}, using defaults", e);
      Config::default()
    },
  };
  let schedule_path = config.get_schedule_file_path();
  let schedule = load_schedule_silent(&schedule_path).unwrap_or_else(|e| {
    log::warn!("Failed to load schedule for dry run: {}, using empty schedule", e);
    Schedule::default()
  });

  if schedule.tasks.is_empty() {
    println!("Schedule is empty - nothing to preview");
    return;
  }

  println!("Previewing {} scheduled tasks:\n", schedule.tasks.len());

  log::info!("Executing dry run for {} scheduled tasks", schedule.tasks.len());

  for task in schedule.tasks.iter() {
    log::debug!("Processing task {} (widget: {})", task.id, task.widget);

    let local_time = task.time.with_timezone(&Local::now().timezone());
    let formatted_time = local_time.format("%Y.%m.%d %I:%M %p").to_string();

    let message = match execute_widget(&task.widget, &task.input).await {
      Ok(msg) => msg,
      Err(e) => {
        log::error!("Failed to execute widget '{}': {}", task.widget, e);
        widget_utils::error_to_display_message(&e)
      },
    };

    let destination = MessageDestination::ConsoleWithTitle(formatted_time);
    match handle_message(message, destination).await {
      Ok(_) => {},
      Err(e) => {
        log::error!("Failed to handle message for task {}: {}", task.id, e);
        eprintln!("Error handling message for task {}: {}", task.id, e);
      },
    }
  }

  log::info!("Schedule dry run completed");
  println!("\n✓ Preview complete");
}
