//! Runner framework for playlist and schedule execution.
//!
//! This module provides shared infrastructure for running playlists and schedules
//! with keyboard controls, instance locking, and graceful shutdown.

pub mod common;
pub mod keyboard;
pub mod lock;
pub mod playlist_runner;
pub mod schedule_runner;

use crossterm::event::KeyCode;

use crate::errors::VestaboardError;

/// Help text for playlist runner keyboard controls
pub const PLAYLIST_HELP: &str = "\
Playlist Controls:
  p - Pause rotation
  r - Resume rotation
  n - Show next item now
  q - Quit
  ? - Show this help";

/// Help text for schedule runner keyboard controls
pub const SCHEDULE_HELP: &str = "\
Schedule Controls:
  q - Quit
  ? - Show this help";

/// Control flow decision after handling an event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlow {
  /// Continue running
  Continue,
  /// Exit the runner
  Exit,
}

/// Common trait for playlist and schedule runners
pub trait Runner: Send {
  /// Called once when the runner starts
  fn start(&mut self);

  /// Run one iteration of the runner (check if work needs to be done, do it)
  /// Returns quickly if nothing to do (non-blocking)
  async fn run_iteration(&mut self) -> Result<ControlFlow, VestaboardError>;

  /// Handle a keyboard input, return whether to continue
  fn handle_key(&mut self, key: KeyCode) -> ControlFlow;

  /// Get help text for keyboard controls
  fn help_text(&self) -> &'static str;

  /// Called on graceful shutdown
  fn cleanup(&mut self);
}
