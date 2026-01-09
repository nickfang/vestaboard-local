//! Playlist runner implementation.
//!
//! Handles playlist execution with interactive controls, state persistence,
//! and widget display.

use std::path::PathBuf;
use std::time::Instant;

use crossterm::event::KeyCode;

use crate::api_broker::{handle_message, MessageDestination};
use crate::cli_display::{print_error, print_progress, print_success};
use crate::errors::VestaboardError;
use crate::playlist::Playlist;
use crate::runner::{ControlFlow, Runner, PLAYLIST_HELP};
use crate::runtime_state::{PlaylistState, RuntimeState};
use crate::widgets::resolver::execute_widget;
use crate::widgets::widget_utils::error_to_display_message;

/// Playlist runner that handles playlist execution with keyboard controls.
pub struct PlaylistRunner {
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
}

impl PlaylistRunner {
    /// Create a new playlist runner.
    ///
    /// # Arguments
    /// * `playlist` - The playlist to run
    /// * `state_path` - Path to save/load runtime state
    /// * `start_index` - Index to start from (0-based)
    /// * `run_once` - If true, exit after completing one full cycle
    /// * `dry_run` - If true, display to console instead of Vestaboard
    pub fn new(
        playlist: Playlist,
        state_path: PathBuf,
        start_index: usize,
        run_once: bool,
        dry_run: bool,
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
        }
    }

    /// Restore from saved state if available.
    pub fn restore_from_state(
        playlist: Playlist,
        state_path: PathBuf,
        run_once: bool,
        dry_run: bool,
    ) -> Self {
        let saved_state = RuntimeState::load(&state_path);

        let start_index = if saved_state.playlist_index < playlist.len() {
            saved_state.playlist_index
        } else {
            0
        };

        log::info!("Restored playlist state: index={}", start_index);

        Self::new(playlist, state_path, start_index, run_once, dry_run)
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
                println!(
                    "Next: {} [{}] - will display on resume",
                    item.widget, item.id
                );
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
                }
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
            }
        };

        log::info!(
            "Displaying playlist item {}/{}: {} ({})",
            self.current_index + 1,
            self.playlist.len(),
            item.id,
            item.widget
        );
        print_progress(&format!(
            "[{}/{}] Showing {}...",
            self.current_index + 1,
            self.playlist.len(),
            item.widget
        ));

        // Save state BEFORE display (ensures we retry on crash)
        self.save_state();

        // Execute widget to generate message
        let message = match execute_widget(&item.widget, &item.input).await {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("Widget '{}' failed: {}", item.widget, e);
                print_error(&format!(
                    "Widget {} failed: {}",
                    item.widget,
                    e.to_user_message()
                ));
                // Continue with error display
                error_to_display_message(&e)
            }
        };

        // Send to Vestaboard or console (dry-run)
        let destination = if self.dry_run {
            MessageDestination::Console
        } else {
            MessageDestination::Vestaboard
        };

        match handle_message(message, destination).await {
            Ok(_) => {
                log::info!("Successfully displayed item {}", item.id);
                self.last_display_time = Some(Instant::now());
                print_success(&format!("Displayed: {}", item.widget));
            }
            Err(e) => {
                log::error!("Failed to send message: {}", e);
                print_error(&e.to_user_message());
                // Don't fail the whole runner - continue to next item
                self.last_display_time = Some(Instant::now());
            }
        }

        Ok(())
    }
}

impl Runner for PlaylistRunner {
    fn start(&mut self) {
        if self.playlist.is_empty() {
            log::warn!("Cannot start empty playlist");
            return;
        }

        self.state = PlaylistState::Running;
        self.cycle_complete = false;
        self.save_state();

        log::info!(
            "Playlist started at index {}/{}",
            self.current_index + 1,
            self.playlist.len()
        );

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
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.pause();
                ControlFlow::Continue
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.resume();
                ControlFlow::Continue
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.handle_next_key();
                ControlFlow::Continue
            }
            KeyCode::Char('?') => {
                println!("\n{}\n", self.help_text());
                ControlFlow::Continue
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playlist::PlaylistItem;
    use serde_json::json;
    use tempfile::tempdir;

    fn create_test_playlist() -> Playlist {
        let mut playlist = Playlist::default();
        playlist.interval_seconds = 60;
        playlist.add_item(PlaylistItem {
            id: "a".to_string(),
            widget: "weather".to_string(),
            input: json!(null),
        });
        playlist.add_item(PlaylistItem {
            id: "b".to_string(),
            widget: "text".to_string(),
            input: json!("hello"),
        });
        playlist.add_item(PlaylistItem {
            id: "c".to_string(),
            widget: "sat-word".to_string(),
            input: json!(null),
        });
        playlist
    }

    #[test]
    fn test_playlist_runner_starts_at_index_zero() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        assert_eq!(runner.current_index(), 0);
    }

    #[test]
    fn test_playlist_runner_starts_at_given_index() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let runner = PlaylistRunner::new(playlist, state_path, 2, false, true);

        assert_eq!(runner.current_index(), 2);
    }

    #[test]
    fn test_playlist_runner_initial_state_is_stopped() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        assert_eq!(runner.state(), PlaylistState::Stopped);
    }

    #[test]
    fn test_playlist_runner_start_sets_running() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        runner.start();

        assert_eq!(runner.state(), PlaylistState::Running);
    }

    #[test]
    fn test_playlist_runner_pause_sets_paused() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        runner.start();
        runner.pause();

        assert_eq!(runner.state(), PlaylistState::Paused);
    }

    #[test]
    fn test_playlist_runner_resume_sets_running() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        runner.start();
        runner.pause();
        runner.resume();

        assert_eq!(runner.state(), PlaylistState::Running);
    }

    #[test]
    fn test_playlist_runner_pause_when_stopped_is_noop() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        runner.pause(); // Should not crash

        assert_eq!(runner.state(), PlaylistState::Stopped);
    }

    #[test]
    fn test_playlist_runner_skip_advances_index() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        runner.start();
        runner.skip_to_next();

        assert_eq!(runner.current_index(), 1);
    }

    #[test]
    fn test_playlist_runner_wraps_at_end() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist(); // 3 items
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        runner.start();
        runner.skip_to_next(); // 0 -> 1
        runner.skip_to_next(); // 1 -> 2
        runner.skip_to_next(); // 2 -> 0 (wrap)

        assert_eq!(runner.current_index(), 0);
    }

    #[test]
    fn test_playlist_runner_once_mode_sets_complete() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist(); // 3 items
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, true, true);

        runner.start();
        assert!(!runner.is_complete());

        runner.skip_to_next(); // 0 -> 1
        assert!(!runner.is_complete());

        runner.skip_to_next(); // 1 -> 2
        assert!(!runner.is_complete());

        runner.skip_to_next(); // 2 -> 0, cycle complete
        assert!(runner.is_complete());
    }

    #[test]
    fn test_playlist_runner_p_key_pauses() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        runner.start();
        let result = runner.handle_key(KeyCode::Char('p'));

        assert_eq!(result, ControlFlow::Continue);
        assert_eq!(runner.state(), PlaylistState::Paused);
    }

    #[test]
    fn test_playlist_runner_r_key_resumes() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        runner.start();
        runner.pause();
        let result = runner.handle_key(KeyCode::Char('r'));

        assert_eq!(result, ControlFlow::Continue);
        assert_eq!(runner.state(), PlaylistState::Running);
    }

    #[test]
    fn test_playlist_runner_n_key_does_not_advance_while_running() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        runner.start();
        let result = runner.handle_key(KeyCode::Char('n'));

        // 'n' while running should NOT advance - just trigger immediate display
        assert_eq!(result, ControlFlow::Continue);
        assert_eq!(runner.current_index(), 0);
    }

    #[test]
    fn test_playlist_runner_q_key_exits() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        runner.start();
        let result = runner.handle_key(KeyCode::Char('q'));

        assert_eq!(result, ControlFlow::Exit);
    }

    #[test]
    fn test_playlist_runner_unknown_key_ignored() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        runner.start();
        let initial_state = runner.state();
        let initial_index = runner.current_index();

        let result = runner.handle_key(KeyCode::Char('x'));

        assert_eq!(result, ControlFlow::Continue);
        assert_eq!(runner.state(), initial_state);
        assert_eq!(runner.current_index(), initial_index);
    }

    #[test]
    fn test_playlist_runner_saves_state() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let mut runner = PlaylistRunner::new(playlist, state_path.clone(), 0, false, true);

        runner.start();
        runner.skip_to_next();
        runner.pause();

        // State should be saved
        let state = RuntimeState::load(&state_path);
        assert_eq!(state.playlist_index, 1);
        assert_eq!(state.playlist_state, PlaylistState::Paused);
    }

    #[test]
    fn test_playlist_runner_restores_state() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");

        // Pre-create state
        let mut state = RuntimeState::default();
        state.playlist_index = 2;
        state.playlist_state = PlaylistState::Paused;
        state.save(&state_path);

        let playlist = create_test_playlist();
        let runner = PlaylistRunner::restore_from_state(playlist, state_path, false, true);

        assert_eq!(runner.current_index(), 2);
    }

    #[test]
    fn test_playlist_runner_help_text() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist();
        let runner = PlaylistRunner::new(playlist, state_path, 0, false, true);

        let help = runner.help_text();
        assert!(help.contains("p"));
        assert!(help.contains("r"));
        assert!(help.contains("n"));
        assert!(help.contains("q"));
    }

    /// Test that pressing 'n' advances by exactly one item, not two.
    ///
    /// Scenario: Playlist has [A(0), B(1), C(2)]. After displaying A, index is at 1.
    /// User presses 'n' to see B immediately.
    ///
    /// Expected: Index should stay at 1 so that run_iteration displays B.
    /// Bug: If skip_to_next() advances, index becomes 2 and we skip B entirely.
    #[test]
    fn test_pressing_n_should_not_skip_current_item() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist(); // [A, B, C]

        // Start at index 1 (simulating: we just displayed A at index 0, advanced to 1)
        // Now the user wants to see B (the current item) immediately by pressing 'n'
        let mut runner = PlaylistRunner::new(playlist, state_path, 1, false, true);
        runner.start();

        // User presses 'n' to see the current item (B) immediately
        runner.handle_key(KeyCode::Char('n'));

        // The index should still be 1 so that run_iteration will display B
        // If it advanced to 2, we'd skip B and display C instead
        assert_eq!(
            runner.current_index(),
            1,
            "Pressing 'n' should NOT advance the index; it should trigger immediate display of the current item"
        );
    }

    /// Test that pressing 'n' while paused:
    /// - If last_display_time is None (ready for immediate display): advance to next
    /// - If last_display_time is Some (waiting for interval): clear timer for immediate display
    #[test]
    fn test_pressing_n_while_paused_with_timer_clears_timer() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist(); // [A, B, C]

        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);
        runner.start();

        // Simulate having displayed an item (sets last_display_time to Some)
        runner.last_display_time = Some(Instant::now());

        runner.pause();

        // First 'n' while paused with timer set: should clear timer, not advance
        runner.handle_key(KeyCode::Char('n'));
        assert_eq!(runner.current_index(), 0, "First 'n' should clear timer, not advance");

        // Second 'n' while paused with timer cleared: should advance
        runner.handle_key(KeyCode::Char('n'));
        assert_eq!(runner.current_index(), 1, "Second 'n' should advance to next item");

        // Third 'n' while paused: timer still cleared, should advance again
        runner.handle_key(KeyCode::Char('n'));
        assert_eq!(runner.current_index(), 2, "Third 'n' should advance again");
    }

    /// Test that pressing 'n' while paused with no prior display advances immediately.
    /// At startup, last_display_time is None (ready to display), so 'n' should advance.
    #[test]
    fn test_pressing_n_while_paused_at_startup_advances() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("state.json");
        let playlist = create_test_playlist(); // [A, B, C]

        let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true);
        runner.start();
        runner.pause();

        // At startup, last_display_time is None, so 'n' should advance
        runner.handle_key(KeyCode::Char('n'));
        assert_eq!(runner.current_index(), 1, "'n' at startup should advance since timer is None");
    }
}
