use std::{fs, path::PathBuf};

use chrono::{DateTime, Local, Utc};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::widgets::resolver::print_widget_with_timestamp;
use crate::{config::Config, errors::VestaboardError};

pub const CUSTOM_ALPHABET: &[char] = &[
  'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
  't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
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

pub fn save_schedule(schedule: &Schedule, path: &PathBuf) -> Result<(), VestaboardError> {
  log::debug!(
    "Saving schedule with {} tasks to {}",
    schedule.tasks.len(),
    path.display()
  );

  // Save the schedule to the file
  // handle errors appropriately
  match fs::write(path, serde_json::to_string_pretty(schedule).unwrap()) {
    Ok(_) => {
      log::info!("Schedule saved successfully to {}", path.display());
      Ok(())
    },
    Err(e) => {
      log::error!("Failed to save schedule to {}: {}", path.display(), e);
      Err(VestaboardError::io_error(e, "saving schedule to file"))
    },
  }
}

pub fn load_schedule(path: &PathBuf) -> Result<Schedule, VestaboardError> {
  log::debug!("Loading schedule from {}", path.display());
  match fs::read_to_string(&path) {
    Ok(content) => {
      if content.trim().is_empty() {
        log::info!(
          "Schedule file {} is empty, creating new schedule",
          path.display()
        );
        Ok(Schedule::default())
      } else {
        match serde_json::from_str::<Schedule>(&content) {
          Ok(mut schedule) => {
            schedule.tasks.sort_by_key(|task| task.time);
            log::info!(
              "Successfully loaded {} tasks from schedule {}",
              schedule.tasks.len(),
              path.display()
            );
            Ok(schedule)
          },
          Err(e) => {
            log::error!("Failed to parse schedule from {}: {}", path.display(), e);
            Err(VestaboardError::json_error(e, "parsing schedule JSON"))
          },
        }
      }
    },
    Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
      log::info!(
        "Schedule file {} not found, creating new schedule",
        path.display()
      );
      let schedule = Schedule::default();
      match save_schedule(&schedule, path) {
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
      Err(VestaboardError::schedule_error(
        "load_schedule",
        &format!("Failed to read schedule file: {}", e),
      ))
    },
  }
}

pub fn add_task_to_schedule(
  time: DateTime<Utc>,
  widget: String,
  input: Value,
) -> Result<(), VestaboardError> {
  log::info!(
    "Adding task to schedule - time: {}, widget: {}, input: {}",
    time,
    widget,
    serde_json::to_string(&input).unwrap_or_else(|_| "invalid".to_string())
  );

  let config = Config::load()?;
  let schedule_path = config.get_schedule_file_path();
  let mut schedule = load_schedule(&schedule_path)?;

  let task = ScheduledTask::new(time, widget.clone(), input);
  let task_id = task.id.clone();
  schedule.add_task(task);

  match save_schedule(&schedule, &schedule_path) {
    Ok(_) => {
      log::info!(
        "Successfully added task {} for widget '{}'",
        task_id,
        widget
      );
      Ok(())
    },
    Err(e) => {
      log::error!("Failed to save schedule after adding task: {}", e);
      Err(e)
    },
  }
}

pub fn remove_task_from_schedule(id: &str) -> Result<bool, VestaboardError> {
  log::info!("Removing task with ID: {}", id);

  let config = Config::load()?;
  let schedule_path = config.get_schedule_file_path();
  let mut schedule = load_schedule(&schedule_path)?;

  if schedule.get_task(id).is_none() {
    log::warn!("Task with ID {} not found in schedule", id);
    return Ok(false);
  }

  if schedule.remove_task(id) {
    match save_schedule(&schedule, &schedule_path) {
      Ok(_) => {
        log::info!("Successfully removed task with ID {}", id);
        Ok(true)
      },
      Err(e) => {
        log::error!("Failed to save schedule after removing task {}: {}", id, e);
        Err(e)
      },
    }
  } else {
    log::error!("Failed to remove task with ID {}", id);
    Ok(false)
  }
}

pub fn clear_schedule() -> Result<(), VestaboardError> {
  log::info!("Clearing all scheduled tasks");

  let config = Config::load()?;
  let schedule_path = config.get_schedule_file_path();
  let mut schedule = load_schedule(&schedule_path)?;
  let task_count = schedule.tasks.len();

  log::info!("Clearing schedule...");
  schedule.clear();

  match save_schedule(&schedule, &schedule_path) {
    Ok(_) => {
      log::info!("Successfully cleared {} tasks from schedule", task_count);
      Ok(())
    },
    Err(e) => {
      log::error!("Failed to save schedule after clearing: {}", e);
      Err(e)
    },
  }
}

pub fn list_schedule() -> Result<(), VestaboardError> {
  log::debug!("Listing scheduled tasks");

  let config = Config::load()?;
  let schedule_path = config.get_schedule_file_path();
  let schedule = load_schedule(&schedule_path)?;

  log::info!("Displaying {} scheduled tasks", schedule.tasks.len());

  println!("\nScheduled Tasks:");
  println!(
    "{:<6} | {:<22} | {:<15} | {}",
    "ID", "Time (Local)", "Widget", "Input"
  );
  println!("{:-<80}", ""); // Separator line
  if schedule.tasks.is_empty() {
    log::debug!("No scheduled tasks found");
    println!("");
    return Ok(());
  }
  for task in schedule.tasks {
    let local_time = task.time.with_timezone(&Local::now().timezone());
    let formatted_time = local_time.format("%Y.%m.%d %I:%M %p").to_string();
    let input_str =
      serde_json::to_string(&task.input).unwrap_or_else(|_| "Invalid JSON".to_string());
    println!(
      "{:<6} | {:<22} | {:<15} | {}",
      task.id, formatted_time, task.widget, input_str
    );
  }
  println!("{:-<80}", ""); // Footer separator line
  Ok(())
}

pub async fn print_schedule() {
  log::debug!("Running schedule dry run");

  let config = match Config::load() {
    Ok(c) => c,
    Err(e) => {
      log::warn!(
        "Failed to load config for schedule dry run: {}, using defaults",
        e
      );
      Config::default() // Fall back to default config
    },
  };
  let schedule_path = config.get_schedule_file_path();
  let schedule = load_schedule(&schedule_path).unwrap_or_else(|e| {
    log::warn!(
      "Failed to load schedule for dry run: {}, using empty schedule",
      e
    );
    Schedule::default()
  });

  log::info!(
    "Executing dry run for {} scheduled tasks",
    schedule.tasks.len()
  );

  for task in schedule.tasks.iter() {
    log::debug!("Processing task {} (widget: {})", task.id, task.widget);

    // Use the new resolver for preview execution
    print_widget_with_timestamp(&task.widget, &task.input, Some(task.time)).await;
  }

  log::info!("Schedule dry run completed");
}
