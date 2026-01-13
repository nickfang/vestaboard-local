//! Schedule runner implementation.
//!
//! Handles schedule execution with interactive controls and hot-reload support.
//! This runner skips past-due tasks and waits for the next upcoming task.

use std::collections::HashSet;
use std::time::Duration;

use chrono::Utc;
use crossterm::event::KeyCode;

use crate::api_broker::{handle_message, MessageDestination};
use crate::cli_display::{print_error, print_progress, print_success};
use crate::errors::VestaboardError;
use crate::runner::{ControlFlow, Runner, SCHEDULE_HELP};
use crate::scheduler::{Schedule, ScheduledTask};
use crate::widgets::resolver::execute_widget;
use crate::widgets::widget_utils::error_to_display_message;

/// Schedule runner that handles schedule execution with keyboard controls.
pub struct ScheduleRunner {
  schedule: Schedule,
  executed_task_ids: HashSet<String>,
  dry_run: bool,
}

impl ScheduleRunner {
  /// Create a new schedule runner.
  ///
  /// # Arguments
  /// * `schedule` - The schedule to run
  /// * `dry_run` - If true, display to console instead of Vestaboard
  pub fn new(schedule: Schedule, dry_run: bool) -> Self {
    Self {
      schedule,
      executed_task_ids: HashSet::new(),
      dry_run,
    }
  }

  /// Get the next pending task that is due or in the future.
  ///
  /// Skips past-due tasks and returns the soonest future task that
  /// hasn't been executed yet.
  pub fn next_pending_task(&self) -> Option<&ScheduledTask> {
    let now = Utc::now();

    self
      .schedule
      .tasks
      .iter()
      .filter(|task| !self.executed_task_ids.contains(&task.id))
      .filter(|task| task.time > now)
      .min_by_key(|task| task.time)
  }

  /// Get the next task that is due for execution (time <= now).
  fn next_due_task(&self) -> Option<&ScheduledTask> {
    let now = Utc::now();

    self
      .schedule
      .tasks
      .iter()
      .filter(|task| !self.executed_task_ids.contains(&task.id))
      .filter(|task| task.time <= now)
      .min_by_key(|task| task.time)
  }

  /// Get the time until the next pending task.
  ///
  /// Returns None if there are no pending tasks.
  pub fn time_until_next_task(&self) -> Option<Duration> {
    self.next_pending_task().map(|task| {
      let now = Utc::now();
      let diff = task.time - now;
      // Convert to std::time::Duration, handling negative values
      if diff.num_seconds() > 0 {
        Duration::from_secs(diff.num_seconds() as u64)
      } else {
        Duration::ZERO
      }
    })
  }

  /// Mark a task as executed.
  pub fn mark_executed(&mut self, task_id: &str) {
    self.executed_task_ids.insert(task_id.to_string());
    log::debug!("Marked task {} as executed", task_id);
  }

  /// Check if this runner is in dry-run mode.
  pub fn is_dry_run(&self) -> bool {
    self.dry_run
  }

  /// Reload the schedule with new data.
  ///
  /// This clears the executed set, allowing tasks to re-run if they
  /// become due again (e.g., after a schedule file edit).
  pub fn reload_schedule(&mut self, schedule: Schedule) {
    self.schedule = schedule;
    self.executed_task_ids.clear();
    log::info!("Schedule reloaded, executed set cleared");
  }

  /// Execute a task and send to Vestaboard (or console in dry-run mode).
  async fn execute_task(&mut self, task: &ScheduledTask) -> Result<(), VestaboardError> {
    log::info!("Executing scheduled task: {} ({})", task.widget, task.id);
    print_progress(&format!("Executing task {} ({})...", task.id, task.widget));

    // Execute the widget to get the message
    let message = match execute_widget(&task.widget, &task.input).await {
      Ok(msg) => msg,
      Err(e) => {
        log::error!("Widget '{}' failed: {}", task.widget, e);
        print_error(&format!("Widget {} failed: {}", task.widget, e.to_user_message()));
        error_to_display_message(&e)
      },
    };

    // Send to Vestaboard or console
    let destination = if self.dry_run {
      MessageDestination::Console
    } else {
      MessageDestination::Vestaboard
    };

    match handle_message(message, destination).await {
      Ok(_) => {
        log::info!("Task {} completed successfully", task.id);
        print_success(&format!("Task {} completed", task.id));
      },
      Err(e) => {
        log::error!("Failed to send message: {}", e);
        print_error(&e.to_user_message());
      },
    }

    Ok(())
  }
}

impl Runner for ScheduleRunner {
  fn start(&mut self) {
    log::info!("Schedule runner started with {} tasks", self.schedule.tasks.len());

    let mode = if self.dry_run { "preview" } else { "live" };
    print_progress(&format!("Starting schedule runner ({} tasks, {} mode)...", self.schedule.tasks.len(), mode));

    // Show next pending task info
    if let Some(task) = self.next_pending_task() {
      let local_time = task.time.with_timezone(&chrono::Local::now().timezone());
      let formatted_time = local_time.format("%I:%M %p").to_string();
      println!("Next task: {} at {}", task.widget, formatted_time);
    } else {
      println!("No upcoming tasks in schedule.");
    }
  }

  async fn run_iteration(&mut self) -> Result<ControlFlow, VestaboardError> {
    // Check if any task is due now
    if let Some(task) = self.next_due_task().cloned() {
      self.execute_task(&task).await?;
      self.mark_executed(&task.id);

      // Show next pending task info
      if let Some(next) = self.next_pending_task() {
        let local_time = next.time.with_timezone(&chrono::Local::now().timezone());
        let formatted_time = local_time.format("%I:%M %p").to_string();
        println!("Next task: {} at {}", next.widget, formatted_time);
      } else {
        println!("No more upcoming tasks.");
      }
    }

    Ok(ControlFlow::Continue)
  }

  fn handle_key(&mut self, key: KeyCode) -> ControlFlow {
    match key {
      KeyCode::Char('q') | KeyCode::Char('Q') => {
        log::info!("Quit requested via keyboard");
        ControlFlow::Exit
      },
      KeyCode::Char('?') => {
        println!("\n{}\n", self.help_text());
        ControlFlow::Continue
      },
      _ => ControlFlow::Continue,
    }
  }

  fn help_text(&self) -> &'static str {
    SCHEDULE_HELP
  }

  fn cleanup(&mut self) {
    log::info!("Schedule runner cleanup complete");
  }
}
