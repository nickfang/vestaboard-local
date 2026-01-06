//! Tests for the runner/lock module.

use crate::runner::lock::InstanceLock;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_lock_acquires_when_no_existing_lock() {
    let temp_dir = tempdir().unwrap();
    let lock_path = temp_dir.path().join("test.lock");

    let lock = InstanceLock::acquire_at("playlist", &lock_path);
    assert!(lock.is_ok());
    assert!(lock_path.exists());
}

#[test]
fn test_lock_file_contains_correct_data() {
    let temp_dir = tempdir().unwrap();
    let lock_path = temp_dir.path().join("test.lock");

    let _lock = InstanceLock::acquire_at("playlist", &lock_path).unwrap();

    let content = fs::read_to_string(&lock_path).unwrap();
    // Pretty-printed JSON has spaces after colons
    assert!(content.contains(r#""mode": "playlist""#));
    assert!(content.contains(r#""pid":"#));
    assert!(content.contains(r#""started_at":"#));
}

#[test]
fn test_lock_released_on_drop() {
    let temp_dir = tempdir().unwrap();
    let lock_path = temp_dir.path().join("test.lock");

    {
        let _lock = InstanceLock::acquire_at("playlist", &lock_path).unwrap();
        assert!(lock_path.exists());
    }
    // Lock dropped here

    assert!(!lock_path.exists());
}

#[test]
fn test_lock_fails_when_already_held() {
    let temp_dir = tempdir().unwrap();
    let lock_path = temp_dir.path().join("test.lock");

    let lock1 = InstanceLock::acquire_at("playlist", &lock_path).unwrap();
    let lock2 = InstanceLock::acquire_at("schedule", &lock_path);

    assert!(lock2.is_err());
    let err = lock2.unwrap_err();
    assert!(err.to_string().contains("already running"));

    drop(lock1);
}

#[test]
fn test_lock_succeeds_when_lock_file_has_dead_pid() {
    let temp_dir = tempdir().unwrap();
    let lock_path = temp_dir.path().join("test.lock");

    // Write a lock file with a PID that almost certainly doesn't exist
    let fake_lock = r#"{"mode":"playlist","pid":999999999,"started_at":"2025-01-01T00:00:00Z"}"#;
    fs::write(&lock_path, fake_lock).unwrap();

    // Should succeed because PID is not running
    let lock = InstanceLock::acquire_at("schedule", &lock_path);
    assert!(lock.is_ok());
}

#[test]
fn test_lock_succeeds_when_lock_file_corrupted() {
    let temp_dir = tempdir().unwrap();
    let lock_path = temp_dir.path().join("test.lock");

    // Write corrupted lock file
    fs::write(&lock_path, "not valid json").unwrap();

    // Should succeed (treat as stale)
    let lock = InstanceLock::acquire_at("playlist", &lock_path);
    assert!(lock.is_ok());
}

#[test]
fn test_lock_path_getter() {
    let temp_dir = tempdir().unwrap();
    let lock_path = temp_dir.path().join("test.lock");

    let lock = InstanceLock::acquire_at("playlist", &lock_path).unwrap();
    assert_eq!(lock.path(), &lock_path);
}

#[test]
fn test_lock_creates_parent_directory() {
    let temp_dir = tempdir().unwrap();
    let nested_path = temp_dir.path().join("nested").join("dir").join("test.lock");

    // Parent directory doesn't exist yet
    assert!(!nested_path.parent().unwrap().exists());

    let lock = InstanceLock::acquire_at("playlist", &nested_path);
    assert!(lock.is_ok());
    assert!(nested_path.exists());
}
