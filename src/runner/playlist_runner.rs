//! Playlist runner implementation.
//!
//! Handles playlist execution with interactive controls, state persistence,
//! and widget display.

use std::path::PathBuf;
use std::time::Instant;

use crossterm::event::KeyCode;

use crate::api::Transport;
use crate::cli_display::print_progress;
use crate::errors::VestaboardError;
use crate::playlist::Playlist;
use crate::runner::common::execute_and_send;
use crate::runner::{ControlFlow, Runner, PLAYLIST_HELP};
use crate::runtime_state::{PlaylistState, RuntimeState};

/// Playlist runner that handles playlist execution with keyboard controls.
pub struct PlaylistRunner<'a> {
  playlist: Playlist,
  state: PlaylistState,
  current_index: usize,
  state_path: PathBuf,
  run_once: bool,
  cycle_complete: bool,
  /// Time of last display, used for interval timing.
  /// None means ready to display immediately (at startup or after 'n' pressed).
  pub(crate) last_display_time: Option<Instant>,
  /// Tracks when we paused, for preserving remaining interval time on resume
  paused_at: Option<Instant>,
  dry_run: bool,
  transport: &'a Transport,
}

impl<'a> PlaylistRunner<'a> {
  /// Create a new playlist runner.
  ///
  /// # Arguments
  /// * `playlist` - The playlist to run
  /// * `state_path` - Path to save/load runtime state
  /// * `start_index` - Index to start from (0-based)
  /// * `run_once` - If true, exit after completing one full cycle
  /// * `dry_run` - If true, display to console instead of Vestaboard
  /// * `transport` - The transport to use for sending to Vestaboard
  pub fn new(
    playlist: Playlist,
    state_path: PathBuf,
    start_index: usize,
    run_once: bool,
    dry_run: bool,
    transport: &'a Transport,
  ) -> Self {
    Self {
      playlist,
      state: PlaylistState::Stopped,
      current_index: start_index,
      state_path,
      run_once,
      cycle_complete: false,
      last_display_time: None,
      paused_at: None,
      dry_run,
      transport,
    }
  }

  /// Restore from saved state if available.
  pub fn restore_from_state(
    playlist: Playlist,
    state_path: PathBuf,
    run_once: bool,
    dry_run: bool,
    transport: &'a Transport,
  ) -> Self {
    let saved_state = RuntimeState::load(&state_path);

    let start_index = if saved_state.playlist_index < playlist.len() {
      saved_state.playlist_index
    } else {
      0
    };

    log::info!("Restored playlist state: index={}", start_index);

    Self::new(playlist, state_path, start_index, run_once, dry_run, transport)
  }

  /// Get the current index in the playlist.
  pub fn current_index(&self) -> usize {
    self.current_index
  }

  /// Get the current playlist state.
  pub fn state(&self) -> PlaylistState {
    self.state
  }

  /// Check if the playlist has completed a full cycle (for --once mode).
  pub fn is_complete(&self) -> bool {
    self.cycle_complete
  }

  /// Pause the playlist rotation.
  ///
  /// Records the pause time so remaining interval time can be preserved on resume.
  pub fn pause(&mut self) {
    if self.state == PlaylistState::Running {
      self.state = PlaylistState::Paused;
      self.paused_at = Some(Instant::now());
      self.save_state();
      log::info!("Playlist paused at index {}", self.current_index);
      println!("Paused.");
    }
  }

  /// Resume playlist rotation from paused state.
  ///
  /// Adjusts `last_display_time` to preserve remaining interval time.
  /// For example, if the user paused with 30 seconds left until the next item,
  /// resuming will wait those 30 seconds before displaying the next item.
  pub fn resume(&mut self) {
    if self.state == PlaylistState::Paused {
      self.state = PlaylistState::Running;

      // Preserve remaining interval time by adjusting last_display_time
      // to account for time spent paused
      if let (Some(last), Some(paused)) = (self.last_display_time, self.paused_at) {
        let pause_duration = paused.elapsed();
        self.last_display_time = Some(last + pause_duration);
        log::debug!("Adjusted last_display_time by {:?}", pause_duration);
      }
      self.paused_at = None;

      self.save_state();
      log::info!("Playlist resumed from index {}", self.current_index);
      println!("Resumed.");
    }
  }

  /// Handle the 'n' (next) key press.
  ///
  /// Behavior from user perspective:
  /// - "Next" always means the item that hasn't been shown yet (what index points to)
  /// - While running: trigger immediate display of the current queued item
  /// - While paused (first 'n'): queue immediate display on resume
  /// - While paused (subsequent 'n's): skip to following item and show preview
  fn handle_next_key(&mut self) {
    // If already queued for immediate display (last_display_time is None) and paused,
    // then skip to next item
    if self.last_display_time.is_none() && self.state == PlaylistState::Paused {
      self.skip_to_next();
    } else {
      // Clear timer to trigger immediate display on next iteration
      self.last_display_time = None;
      log::info!("Queued immediate display of item {}", self.current_index);
    }

    // If paused, show preview of what will display on resume
    if self.state == PlaylistState::Paused {
      if let Some(item) = self.playlist.get_item_by_index(self.current_index) {
        println!("Next: {} [{}] - will display on resume", item.widget, item.id);
      }
    } else {
      println!("Showing next item...");
    }
  }

  /// Skip to the next item in the playlist.
  ///
  /// Advances the index and logs the action. Does NOT clear the display timer.
  pub fn skip_to_next(&mut self) {
    self.advance_index();
    log::info!("Skipped to item {}", self.current_index);
    println!("Skipping to next item...");
  }

  /// Advance to the next index (wrapping around).
  fn advance_index(&mut self) {
    if self.playlist.is_empty() {
      return;
    }

    self.current_index = (self.current_index + 1) % self.playlist.len();

    // Check if we completed a full cycle
    if self.current_index == 0 {
      self.cycle_complete = true;
    }
  }

  /// Check if it's time to display the next item.
  fn should_display_next(&self) -> bool {
    match self.state {
      PlaylistState::Running => match self.last_display_time {
        None => true, // First display
        Some(last) => {
          let elapsed = last.elapsed().as_secs();
          elapsed >= self.playlist.interval_seconds
        },
      },
      _ => false,
    }
  }

  /// Save current state to disk.
  fn save_state(&self) {
    let state = RuntimeState {
      playlist_state: self.state,
      playlist_index: self.current_index,
      last_shown_time: Some(chrono::Utc::now()),
    };
    state.save(&self.state_path);
  }

  /// Display the current playlist item.
  async fn display_current_item(&mut self) -> Result<(), VestaboardError> {
    let item = match self.playlist.get_item_by_index(self.current_index) {
      Some(item) => item.clone(),
      None => {
        log::warn!("No current item to display at index {}", self.current_index);
        return Ok(());
      },
    };

    log::info!(
      "Displaying playlist item {}/{}: {} ({})",
      self.current_index + 1,
      self.playlist.len(),
      item.id,
      item.widget
    );
    print_progress(&format!("[{}/{}] Showing {}...", self.current_index + 1, self.playlist.len(), item.widget));

    // Save state BEFORE display (ensures we retry on crash)
    self.save_state();

    let label = format!("Item {}", item.widget);
    // Ignore the result - we want to continue even if sending fails
    let _ = execute_and_send(&item.widget, &item.input, self.dry_run, &label, self.transport).await;

    // Always update display time to maintain interval timing
    self.last_display_time = Some(Instant::now());

    Ok(())
  }
}

impl<'a> Runner for PlaylistRunner<'a> {
  fn start(&mut self) {
    if self.playlist.is_empty() {
      log::warn!("Cannot start empty playlist");
      return;
    }

    self.state = PlaylistState::Running;
    self.cycle_complete = false;
    self.save_state();

    log::info!("Playlist started at index {}/{}", self.current_index + 1, self.playlist.len());

    let mode = if self.dry_run { "preview" } else { "live" };
    print_progress(&format!(
      "Starting playlist ({} items, {} second interval, {} mode)...",
      self.playlist.len(),
      self.playlist.interval_seconds,
      mode
    ));
  }

  async fn run_iteration(&mut self) -> Result<ControlFlow, VestaboardError> {
    // Check if we should exit (--once mode)
    if self.run_once && self.cycle_complete {
      log::info!("Completed one full cycle, stopping (--once mode)");
      println!("Completed one full cycle.");
      self.state = PlaylistState::Stopped;
      return Ok(ControlFlow::Exit);
    }

    // Only display if running and interval has elapsed
    if self.should_display_next() {
      self.display_current_item().await?;
      self.advance_index();
    }

    Ok(ControlFlow::Continue)
  }

  fn handle_key(&mut self, key: KeyCode) -> ControlFlow {
    match key {
      KeyCode::Char('q') | KeyCode::Char('Q') => {
        log::info!("Quit requested via keyboard");
        ControlFlow::Exit
      },
      KeyCode::Char('p') | KeyCode::Char('P') => {
        self.pause();
        ControlFlow::Continue
      },
      KeyCode::Char('r') | KeyCode::Char('R') => {
        self.resume();
        ControlFlow::Continue
      },
      KeyCode::Char('n') | KeyCode::Char('N') => {
        self.handle_next_key();
        ControlFlow::Continue
      },
      KeyCode::Char('?') => {
        println!("\n{}\n", self.help_text());
        ControlFlow::Continue
      },
      _ => ControlFlow::Continue,
    }
  }

  fn help_text(&self) -> &'static str {
    PLAYLIST_HELP
  }

  fn cleanup(&mut self) {
    self.state = PlaylistState::Stopped;
    self.save_state();
    log::info!("Playlist runner cleanup complete");
  }
}
