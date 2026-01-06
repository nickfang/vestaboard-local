//! Runner framework for playlist and schedule execution.
//!
//! This module provides shared infrastructure for running playlists and schedules
//! with keyboard controls, instance locking, and graceful shutdown.

pub mod keyboard;
pub mod lock;

/// Help text for playlist runner keyboard controls
pub const PLAYLIST_HELP: &str = "\
Playlist Controls:
  p - Pause rotation
  r - Resume rotation
  n - Skip to next item
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_flow_variants() {
        let cont = ControlFlow::Continue;
        let exit = ControlFlow::Exit;
        assert_ne!(cont, exit);
    }

    #[test]
    fn test_playlist_help_contains_expected_keys() {
        assert!(PLAYLIST_HELP.contains("p"));
        assert!(PLAYLIST_HELP.contains("r"));
        assert!(PLAYLIST_HELP.contains("n"));
        assert!(PLAYLIST_HELP.contains("q"));
        assert!(PLAYLIST_HELP.contains("?"));
    }

    #[test]
    fn test_schedule_help_contains_expected_keys() {
        assert!(SCHEDULE_HELP.contains("q"));
        assert!(SCHEDULE_HELP.contains("?"));
    }
}
