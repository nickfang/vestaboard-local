//! Tests for the PlaylistRunner.

use crossterm::event::KeyCode;
use serde_json::json;
use std::time::Instant;
use tempfile::tempdir;

use crate::api::{Transport, TransportType};
use crate::playlist::{Playlist, PlaylistItem};
use crate::runner::playlist_runner::PlaylistRunner;
use crate::runner::{ControlFlow, Runner};
use crate::runtime_state::{PlaylistState, RuntimeState};

/// Create a test transport. Sets env vars if needed.
fn create_test_transport() -> Transport {
  // Set env vars for testing (these are only used if actually sending, which tests don't do)
  if std::env::var("LOCAL_API_KEY").is_err() {
    std::env::set_var("LOCAL_API_KEY", "test-api-key");
  }
  if std::env::var("IP_ADDRESS").is_err() {
    std::env::set_var("IP_ADDRESS", "127.0.0.1");
  }
  Transport::new(TransportType::Local).expect("Failed to create test transport")
}

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
  let transport = create_test_transport();
  let runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

  assert_eq!(runner.current_index(), 0);
}

#[test]
fn test_playlist_runner_starts_at_given_index() {
  let temp_dir = tempdir().unwrap();
  let state_path = temp_dir.path().join("state.json");
  let playlist = create_test_playlist();
  let transport = create_test_transport();
  let runner = PlaylistRunner::new(playlist, state_path, 2, false, true, &transport);

  assert_eq!(runner.current_index(), 2);
}

#[test]
fn test_playlist_runner_initial_state_is_stopped() {
  let temp_dir = tempdir().unwrap();
  let state_path = temp_dir.path().join("state.json");
  let playlist = create_test_playlist();
  let transport = create_test_transport();
  let runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

  assert_eq!(runner.state(), PlaylistState::Stopped);
}

#[test]
fn test_playlist_runner_start_sets_running() {
  let temp_dir = tempdir().unwrap();
  let state_path = temp_dir.path().join("state.json");
  let playlist = create_test_playlist();
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

  runner.start();

  assert_eq!(runner.state(), PlaylistState::Running);
}

#[test]
fn test_playlist_runner_pause_sets_paused() {
  let temp_dir = tempdir().unwrap();
  let state_path = temp_dir.path().join("state.json");
  let playlist = create_test_playlist();
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

  runner.start();
  runner.pause();

  assert_eq!(runner.state(), PlaylistState::Paused);
}

#[test]
fn test_playlist_runner_resume_sets_running() {
  let temp_dir = tempdir().unwrap();
  let state_path = temp_dir.path().join("state.json");
  let playlist = create_test_playlist();
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

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
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

  runner.pause(); // Should not crash

  assert_eq!(runner.state(), PlaylistState::Stopped);
}

#[test]
fn test_playlist_runner_skip_advances_index() {
  let temp_dir = tempdir().unwrap();
  let state_path = temp_dir.path().join("state.json");
  let playlist = create_test_playlist();
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

  runner.start();
  runner.skip_to_next();

  assert_eq!(runner.current_index(), 1);
}

#[test]
fn test_playlist_runner_wraps_at_end() {
  let temp_dir = tempdir().unwrap();
  let state_path = temp_dir.path().join("state.json");
  let playlist = create_test_playlist(); // 3 items
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

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
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, true, true, &transport);

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
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

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
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

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
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

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
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

  runner.start();
  let result = runner.handle_key(KeyCode::Char('q'));

  assert_eq!(result, ControlFlow::Exit);
}

#[test]
fn test_playlist_runner_unknown_key_ignored() {
  let temp_dir = tempdir().unwrap();
  let state_path = temp_dir.path().join("state.json");
  let playlist = create_test_playlist();
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

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
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path.clone(), 0, false, true, &transport);

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
  let transport = create_test_transport();
  let runner = PlaylistRunner::restore_from_state(playlist, state_path, false, true, &transport);

  assert_eq!(runner.current_index(), 2);
}

#[test]
fn test_playlist_runner_help_text() {
  let temp_dir = tempdir().unwrap();
  let state_path = temp_dir.path().join("state.json");
  let playlist = create_test_playlist();
  let transport = create_test_transport();
  let runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);

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
  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 1, false, true, &transport);
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

  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);
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

  let transport = create_test_transport();
  let mut runner = PlaylistRunner::new(playlist, state_path, 0, false, true, &transport);
  runner.start();
  runner.pause();

  // At startup, last_display_time is None, so 'n' should advance
  runner.handle_key(KeyCode::Char('n'));
  assert_eq!(runner.current_index(), 1, "'n' at startup should advance since timer is None");
}
