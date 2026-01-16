//! Schedule runner implementation.
//!
//! Handles schedule execution with interactive controls and hot-reload support.
//! This runner skips past-due tasks and waits for the next upcoming task.

use std::collections::HashSet;
use std::time::Duration;

use chrono::Utc;
use crossterm::event::KeyCode;

use crate::api::Transport;
use crate::cli_display::print_progress;
use crate::errors::VestaboardError;
use crate::runner::common::execute_and_send;
use crate::runner::{ControlFlow, Runner, SCHEDULE_HELP};
use crate::scheduler::{Schedule, ScheduledTask};

/// Schedule runner that handles schedule execution with keyboard controls.
pub struct ScheduleRunner<'a> {
  schedule: Schedule,
  executed_task_ids: HashSet<String>,
  dry_run: bool,
  transport: &'a Transport,
}

impl<'a> ScheduleRunner<'a> {
  /// Create a new schedule runner.
  ///
  /// # Arguments
  /// * `schedule` - The schedule to run
  /// * `dry_run` - If true, display to console instead of Vestaboard
  /// * `transport` - The transport to use for sending to Vestaboard
  pub fn new(schedule: Schedule, dry_run: bool, transport: &'a Transport) -> Self {
    Self {
      schedule,
      executed_task_ids: HashSet::new(),
      dry_run,
      transport,
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

    let label = format!("Task {}", task.id);
    // Ignore the result - we want to continue even if sending fails
    let _ = execute_and_send(&task.widget, &task.input, self.dry_run, &label, self.transport).await;

    Ok(())
  }
}

impl<'a> Runner for ScheduleRunner<'a> {
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
