//! Runtime state persistence for playlist execution.
//!
//! This module provides state persistence across restarts. The design is intentionally
//! best-effort: errors during save/load are logged but don't crash the application.
//! Losing state (resetting to defaults) is preferable to crashing.

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The current state of playlist execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlaylistState {
  /// Playlist is not running
  #[default]
  Stopped,
  /// Playlist is actively rotating items
  Running,
  /// Playlist is paused (position remembered)
  Paused,
}

/// Persisted runtime state for resuming execution across restarts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RuntimeState {
  /// Current playlist execution state
  pub playlist_state: PlaylistState,
  /// Current position in the playlist
  pub playlist_index: usize,
  /// When the last item was displayed
  pub last_shown_time: Option<DateTime<Utc>>,
}

impl RuntimeState {
  /// Load state from file, returning defaults on any error.
  ///
  /// This is intentionally infallible - state persistence is best-effort.
  /// On any error (file not found, corrupted JSON, etc.), we return defaults
  /// and continue. Crashing because of state file corruption would be worse
  /// than losing position.
  pub fn load(path: &Path) -> Self {
    match std::fs::read_to_string(path) {
      Ok(content) if !content.trim().is_empty() => serde_json::from_str(&content).unwrap_or_else(|e| {
        log::warn!("Invalid runtime state JSON, using defaults: {}", e);
        Self::default()
      }),
      Ok(_) => {
        // Empty file
        Self::default()
      },
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
        log::debug!("Runtime state file not found, using defaults");
        Self::default()
      },
      Err(e) => {
        log::warn!("Cannot read runtime state: {}, using defaults", e);
        Self::default()
      },
    }
  }

  /// Save state to file. Errors are logged but not propagated.
  ///
  /// State persistence is best-effort - we don't want to crash if we can't save state.
  pub fn save(&self, path: &Path) {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
      if let Err(e) = std::fs::create_dir_all(parent) {
        log::warn!("Cannot create state directory: {}", e);
        return;
      }
    }

    match serde_json::to_string_pretty(self) {
      Ok(content) => {
        if let Err(e) = std::fs::write(path, content) {
          log::warn!("Cannot save runtime state: {}", e);
        }
      },
      Err(e) => {
        log::warn!("Cannot serialize runtime state: {}", e);
      },
    }
  }

  /// Update index and save (convenience method)
  #[allow(dead_code)]
  pub fn set_index_and_save(&mut self, index: usize, path: &Path) {
    self.playlist_index = index;
    self.last_shown_time = Some(Utc::now());
    self.save(path);
  }
}
