//! Tests for the runtime_state module.

use crate::runtime_state::{PlaylistState, RuntimeState};
use chrono::Utc;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_runtime_state_default_values() {
    let state = RuntimeState::default();
    assert_eq!(state.playlist_state, PlaylistState::Stopped);
    assert_eq!(state.playlist_index, 0);
    assert!(state.last_shown_time.is_none());
}

#[test]
fn test_playlist_state_enum_values() {
    assert_ne!(PlaylistState::Stopped, PlaylistState::Running);
    assert_ne!(PlaylistState::Running, PlaylistState::Paused);
    assert_ne!(PlaylistState::Paused, PlaylistState::Stopped);
}

#[test]
fn test_playlist_state_default_is_stopped() {
    let state = PlaylistState::default();
    assert_eq!(state, PlaylistState::Stopped);
}

#[test]
fn test_runtime_state_save_and_load() {
    let mut state = RuntimeState::default();
    state.playlist_state = PlaylistState::Paused;
    state.playlist_index = 5;
    state.last_shown_time = Some(Utc::now());

    let temp_file = NamedTempFile::new().unwrap();
    state.save(temp_file.path());

    let loaded = RuntimeState::load(temp_file.path());
    assert_eq!(loaded.playlist_state, PlaylistState::Paused);
    assert_eq!(loaded.playlist_index, 5);
    assert!(loaded.last_shown_time.is_some());
}

#[test]
fn test_runtime_state_load_missing_file_returns_default() {
    let path = std::path::Path::new("/nonexistent/runtime_state.json");
    let state = RuntimeState::load(path);
    assert_eq!(state.playlist_state, PlaylistState::Stopped);
    assert_eq!(state.playlist_index, 0);
}

#[test]
fn test_runtime_state_load_corrupted_file_returns_default() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "{{invalid json").unwrap();

    let state = RuntimeState::load(temp_file.path());
    assert_eq!(state.playlist_state, PlaylistState::Stopped);
}

#[test]
fn test_runtime_state_load_empty_file_returns_default() {
    let temp_file = NamedTempFile::new().unwrap();
    let state = RuntimeState::load(temp_file.path());
    assert_eq!(state.playlist_state, PlaylistState::Stopped);
    assert_eq!(state.playlist_index, 0);
}

#[test]
fn test_playlist_state_serialization() {
    let state = PlaylistState::Running;
    let json = serde_json::to_string(&state).unwrap();
    assert_eq!(json, "\"Running\"");

    let deserialized: PlaylistState = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, PlaylistState::Running);
}

#[test]
fn test_playlist_state_serialization_all_variants() {
    let variants = [
        (PlaylistState::Stopped, "\"Stopped\""),
        (PlaylistState::Running, "\"Running\""),
        (PlaylistState::Paused, "\"Paused\""),
    ];

    for (state, expected_json) in variants {
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, expected_json);

        let deserialized: PlaylistState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, state);
    }
}

#[test]
fn test_runtime_state_set_index_and_save() {
    let mut state = RuntimeState::default();
    let temp_file = NamedTempFile::new().unwrap();

    state.set_index_and_save(3, temp_file.path());

    assert_eq!(state.playlist_index, 3);
    assert!(state.last_shown_time.is_some());

    // Verify it was saved
    let loaded = RuntimeState::load(temp_file.path());
    assert_eq!(loaded.playlist_index, 3);
}

#[test]
fn test_runtime_state_load_partial_json() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, r#"{{"playlist_index": 5}}"#).unwrap();

    let state = RuntimeState::load(temp_file.path());
    assert_eq!(state.playlist_index, 5);
    assert_eq!(state.playlist_state, PlaylistState::Stopped); // default
}
