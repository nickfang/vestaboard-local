# Phase 1: Foundation (Non-Breaking)

**Goal**: Add new data structures and shared components without changing existing behavior.

**Prerequisites**: Read [00-overview.md](00-overview.md) first for shared context.

---

## Rust Type Definitions

### PlaylistState Enum

Use a proper Rust enum, not strings:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlaylistState {
    #[default]
    Stopped,
    Running,
    Paused,
}
```

### PlaylistItem Struct

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Re-use the ID generation from scheduler.rs (see src/scheduler.rs:14-22)
use crate::scheduler::{CUSTOM_ALPHABET, ID_LENGTH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    #[serde(default = "generate_item_id")]
    pub id: String,
    pub widget: String,
    pub input: Value,
}

fn generate_item_id() -> String {
    // Use same ID generation as ScheduledTask (nanoid)
    // References: src/scheduler.rs:14-22 for CUSTOM_ALPHABET and ID_LENGTH
    nanoid!(ID_LENGTH, CUSTOM_ALPHABET)
}

impl PlaylistItem {
    /// Create a new playlist item
    /// Note: Widget type is validated at execution time via execute_widget(),
    /// matching the pattern used by ScheduledTask.
    pub fn new(widget: String, input: Value) -> Self {
        Self {
            id: generate_item_id(),
            widget,
            input,
        }
    }
}
```

### Playlist Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    #[serde(default = "default_interval")]
    pub interval_seconds: u64,
    #[serde(default)]
    pub items: Vec<PlaylistItem>,
}

fn default_interval() -> u64 {
    300 // 5 minutes
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            interval_seconds: default_interval(),
            items: Vec::new(),
        }
    }
}

impl Playlist {
    /// Add an item to the playlist
    pub fn add_item(&mut self, item: PlaylistItem) {
        self.items.push(item);
    }

    /// Add an item by widget name
    /// Note: Widget type is validated at execution time via execute_widget(),
    /// matching the pattern used by Schedule.
    pub fn add_widget(&mut self, widget: &str, input: Value) -> String {
        let item = PlaylistItem::new(widget.to_string(), input);
        let id = item.id.clone();
        self.items.push(item);
        id
    }

    pub fn remove_item(&mut self, id: &str) -> bool {
        let len_before = self.items.len();
        self.items.retain(|item| item.id != id);
        self.items.len() < len_before
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get_item(&self, id: &str) -> Option<&PlaylistItem> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn get_item_by_index(&self, index: usize) -> Option<&PlaylistItem> {
        self.items.get(index)
    }

    pub fn find_index_by_id(&self, id: &str) -> Option<usize> {
        self.items.iter().position(|item| item.id == id)
    }

    pub fn validate_interval(&self) -> Result<(), VestaboardError> {
        if self.interval_seconds < 60 {
            return Err(VestaboardError::validation_error(
                "Playlist interval must be at least 60 seconds"
            ));
        }
        Ok(())
    }

    // --- File operations (following scheduler.rs pattern) ---

    /// Load playlist from file (with progress messages)
    pub fn load(path: &Path) -> Result<Self, VestaboardError> {
        Self::load_internal(path, false)
    }

    /// Load playlist without printing progress (for internal operations)
    pub fn load_silent(path: &Path) -> Result<Self, VestaboardError> {
        Self::load_internal(path, true)
    }

    fn load_internal(path: &Path, silent: bool) -> Result<Self, VestaboardError> {
        log::debug!("Loading playlist from {}", path.display());

        match std::fs::read_to_string(path) {
            Ok(content) if content.trim().is_empty() => {
                log::info!("Playlist file is empty, using defaults");
                Ok(Self::default())
            }
            Ok(content) => {
                match serde_json::from_str::<Self>(&content) {
                    Ok(playlist) => {
                        log::info!("Loaded playlist with {} items", playlist.items.len());
                        Ok(playlist)
                    }
                    Err(e) => {
                        log::error!("Failed to parse playlist: {}", e);
                        let error = VestaboardError::json_error(
                            e,
                            &format!("parsing playlist from {}", path.display())
                        );
                        if !silent {
                            print_error(&error.to_user_message());
                        }
                        Err(error)
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::info!("Playlist file not found, creating default");
                let playlist = Self::default();
                // Auto-save on first access (matches scheduler.rs behavior)
                let _ = playlist.save_silent(path);
                Ok(playlist)
            }
            Err(e) => {
                log::error!("Failed to read playlist file: {}", e);
                let error = VestaboardError::io_error(
                    e,
                    &format!("reading playlist from {}", path.display())
                );
                if !silent {
                    print_error(&error.to_user_message());
                }
                Err(error)
            }
        }
    }

    /// Save playlist to file (with progress messages)
    pub fn save(&self, path: &Path) -> Result<(), VestaboardError> {
        self.save_internal(path, false)
    }

    /// Save playlist without printing progress (for internal operations)
    pub fn save_silent(&self, path: &Path) -> Result<(), VestaboardError> {
        self.save_internal(path, true)
    }

    fn save_internal(&self, path: &Path, silent: bool) -> Result<(), VestaboardError> {
        log::debug!("Saving playlist with {} items to {}", self.items.len(), path.display());

        if !silent {
            print_progress("Saving playlist...");
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                let error = VestaboardError::io_error(
                    e,
                    &format!("creating directory for {}", path.display())
                );
                if !silent {
                    print_error(&error.to_user_message());
                }
                return Err(error);
            }
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| VestaboardError::json_error(e, "serializing playlist"))?;

        match std::fs::write(path, content) {
            Ok(_) => {
                log::info!("Playlist saved to {}", path.display());
                if !silent {
                    print_success("Playlist saved.");
                }
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to save playlist: {}", e);
                let error = VestaboardError::io_error(
                    e,
                    &format!("saving playlist to {}", path.display())
                );
                if !silent {
                    print_error(&error.to_user_message());
                }
                Err(error)
            }
        }
    }
}

// Note: Uses print_progress, print_error, print_success from cli_display.rs
// and VestaboardError::io_error, VestaboardError::json_error from errors.rs
```

### RuntimeState Struct

**Design Note**: `load()` returns `Self` directly (not `Result`) because state persistence is best-effort. On any error (file not found, corrupted JSON, etc.), we return defaults and continue. This is intentional - crashing because of state file corruption would be worse than losing position.

```rust
use chrono::{DateTime, Utc};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeState {
    pub playlist_state: PlaylistState,
    pub playlist_index: usize,
    pub last_shown_time: Option<DateTime<Utc>>,
}

impl RuntimeState {
    /// Load state from file, returning defaults on any error.
    /// This is intentionally infallible - state persistence is best-effort.
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) if !content.trim().is_empty() => {
                serde_json::from_str(&content).unwrap_or_else(|e| {
                    log::warn!("Invalid runtime state JSON, using defaults: {}", e);
                    Self::default()
                })
            }
            Ok(_) => {
                // Empty file
                Self::default()
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::debug!("Runtime state file not found, using defaults");
                Self::default()
            }
            Err(e) => {
                log::warn!("Cannot read runtime state: {}, using defaults", e);
                Self::default()
            }
        }
    }

    /// Save state to file. Errors are logged but not propagated.
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
            }
            Err(e) => {
                log::warn!("Cannot serialize runtime state: {}", e);
            }
        }
    }

    /// Update index and save (convenience method)
    pub fn set_index_and_save(&mut self, index: usize, path: &Path) {
        self.playlist_index = index;
        self.last_shown_time = Some(Utc::now());
        self.save(path);
    }
}
```

---

## Playlist State Machine

```
                         start/run
              ┌────────────────────────────────┐
              │                               │
              ▼                               │
         ┌─────────┐      p (pause)      ┌─────────┐
         │ RUNNING │ ──────────────────► │ PAUSED  │
         └─────────┘                     └─────────┘
              │                               │
              │ q (quit)                      │ r (resume)
              │                               │
              ▼                               ▼
         ┌─────────┐                     ┌─────────┐
         │ STOPPED │ ◄─────────────────  │ RUNNING │
         └─────────┘      q (quit)       └─────────┘
```

### State Definitions

| State | Position Remembered | Behavior |
|-------|---------------------|----------|
| **STOPPED** | No (resets to 0) | `run` begins at item 0 |
| **RUNNING** | Yes (current index) | Automatic rotation continues |
| **PAUSED** | Yes (current index) | Waiting for resume; `n` advances position |

### Command Behavior by State

| Command | State | Behavior |
|---------|-------|----------|
| `run` | STOPPED | Start from beginning |
| `run` | RUNNING | "Playlist is running, restarting from beginning." → restarts |
| `run` | PAUSED | "Playlist is paused, restarting from beginning." → starts fresh |
| `run --once` | Any | Run through once, exit at end (don't loop) |
| `run --index N` | Any | Start from index N |
| `run --id X` | Any | Start from item with id X |
| `p` (pause) | RUNNING | Pause, remember position |
| `p` (pause) | PAUSED | No-op, already paused |
| `r` (resume) | PAUSED | Resume from current position |
| `r` (resume) | RUNNING | No-op, already running |
| `n` (next) | RUNNING | Skip to next item immediately |
| `n` (next) | PAUSED | Advance position, stay paused |
| `q` (quit) | Any | Stop and exit |

---

## TDD Implementation Guide

This section provides a **test-driven development (TDD)** approach to implementing the playlist feature. Each phase follows the red-green-refactor cycle:

1. **Red**: Write a failing test that defines expected behavior
2. **Green**: Write the minimum code to make the test pass
3. **Refactor**: Clean up the code while keeping tests green

### TDD Principles for This Project

- **Write tests first** - Before implementing any function, write a test that calls it
- **One test at a time** - Don't write all tests upfront; write one, make it pass, repeat
- **Test behavior, not implementation** - Tests should verify what the code does, not how
- **Run tests frequently** - After every small change, run `cargo test`
- **Commit after each green** - Small, working commits are better than large ones

---

## Phase 1 Checklist

### 1.1 Add Error Variants (estimated: 15 min)

- [x] **Test first**: Add test file `src/tests/playlist_error_tests.rs`
- [x] **Write test**: `test_lock_error_displays_message`
- [x] **Write test**: `test_validation_error_displays_message`
- [x] **Run tests** - should fail (variants don't exist)
- [x] **Implement**: Add `LockError`, `InputError`, `ValidationError` to `errors.rs`
- [x] **Run tests** - should pass
- [x] **Commit**: "Add error variants for playlist feature"

```rust
// src/tests/playlist_error_tests.rs
use crate::errors::VestaboardError;

#[test]
fn test_lock_error_displays_message() {
    let err = VestaboardError::LockError {
        message: "Playlist already running".to_string(),
    };
    assert!(err.to_string().contains("Playlist already running"));
}

#[test]
fn test_validation_error_displays_message() {
    let err = VestaboardError::ValidationError {
        message: "Interval too short".to_string(),
    };
    assert!(err.to_string().contains("Interval too short"));
}

#[test]
fn test_input_error_displays_message() {
    let err = VestaboardError::InputError {
        message: "Not a TTY".to_string(),
    };
    assert!(err.to_string().contains("Not a TTY"));
}
```

### 1.2 Create PlaylistItem and Playlist structs

- [x] **Create test file**: `src/tests/playlist_tests.rs`
- [x] **Add to mod.rs**: `mod playlist_tests;`

**Step 1.2.1: PlaylistItem basics**

- [x] **Write test**: `test_playlist_item_creation`
- [x] **Run test** - fails
- [x] **Create file**: `src/playlist.rs` with `PlaylistItem` struct
- [x] **Run test** - passes
- [x] **Commit**: "Add PlaylistItem struct"

### Test Utilities

Tests need to create `PlaylistItem` instances with known IDs for assertions. Since `PlaylistItem::new()` auto-generates IDs, use these patterns:

```rust
// src/tests/test_utils.rs

use crate::playlist::{Playlist, PlaylistItem};
use serde_json::{json, Value};

/// Create a PlaylistItem with a specific ID for testing
/// Bypasses validation - use only in tests
pub fn test_item(id: &str, widget: &str, input: Value) -> PlaylistItem {
    PlaylistItem {
        id: id.to_string(),
        widget: widget.to_string(),
        input,
    }
}

/// Create a test playlist with weather, text, and sat-word items
pub fn create_test_playlist() -> Playlist {
    let mut playlist = Playlist::default();
    playlist.add_item(test_item("a", "weather", json!(null)));
    playlist.add_item(test_item("b", "text", json!("hello")));
    playlist.add_item(test_item("c", "sat-word", json!(null)));
    playlist
}

/// Create an empty test playlist with a specific interval
pub fn create_test_playlist_with_interval(interval_seconds: u64) -> Playlist {
    Playlist {
        interval_seconds,
        items: Vec::new(),
    }
}
```

**Note**: The `test_item()` helper is for tests that need predictable IDs. In production, `PlaylistItem::new()` auto-generates IDs.

```rust
// src/tests/playlist_tests.rs
use crate::playlist::{Playlist, PlaylistItem};
use crate::tests::test_utils::{test_item, create_test_playlist};
use serde_json::json;

#[test]
fn test_playlist_item_creation() {
    // Using test helper for known ID
    let item = test_item("abc1", "weather", json!(null));
    assert_eq!(item.id, "abc1");
    assert_eq!(item.widget, "weather");
}

#[test]
fn test_playlist_item_creation_generates_id() {
    // Using real constructor (auto-generates ID)
    let item = PlaylistItem::new("weather".to_string(), json!(null));
    assert!(!item.id.is_empty()); // ID is auto-generated
    assert_eq!(item.widget, "weather");
}

#[test]
fn test_playlist_item_serializes_to_json() {
    let item = test_item("abc1", "text", json!("hello world"));
    let serialized = serde_json::to_string(&item).unwrap();
    assert!(serialized.contains("\"widget\":\"text\""));
    assert!(serialized.contains("\"input\":\"hello world\""));
}

#[test]
fn test_playlist_item_deserializes_from_json() {
    // When deserializing, existing IDs are preserved
    let json_str = r#"{"id":"xyz9","widget":"weather","input":null}"#;
    let item: PlaylistItem = serde_json::from_str(json_str).unwrap();
    assert_eq!(item.id, "xyz9");
    assert_eq!(item.widget, "weather");
}

#[test]
fn test_playlist_item_deserializes_without_id_gets_generated() {
    // When deserializing without ID, serde generates one
    let json_str = r#"{"widget":"weather","input":null}"#;
    let item: PlaylistItem = serde_json::from_str(json_str).unwrap();
    assert!(!item.id.is_empty()); // ID should be generated
    assert_eq!(item.widget, "weather");
}
```

**Step 1.2.2: Playlist struct basics**

- [x] **Write test**: `test_playlist_creation_with_defaults`
- [x] **Write test**: `test_playlist_default_interval_is_300`
- [x] **Run tests** - fail
- [x] **Implement**: `Playlist` struct with defaults
- [x] **Run tests** - pass
- [x] **Commit**: "Add Playlist struct with defaults"

```rust
#[test]
fn test_playlist_creation_with_defaults() {
    let playlist = Playlist::default();
    assert!(playlist.items.is_empty());
    assert_eq!(playlist.interval_seconds, 300);
}

#[test]
fn test_playlist_default_interval_is_300() {
    let playlist = Playlist::default();
    assert_eq!(playlist.interval_seconds, 300);
}
```

**Step 1.2.3: Playlist CRUD operations**

- [x] **Write test**: `test_playlist_add_item`
- [x] **Run test** - fails
- [x] **Implement**: `add_item()` method
- [x] **Run test** - passes

```rust
#[test]
fn test_playlist_add_item() {
    let mut playlist = Playlist::default();
    let item = PlaylistItem {
        id: "abc1".to_string(),
        widget: "weather".to_string(),
        input: json!(null),
    };
    playlist.add_item(item);
    assert_eq!(playlist.items.len(), 1);
    assert_eq!(playlist.items[0].widget, "weather");
}

#[test]
fn test_playlist_add_multiple_items_preserves_order() {
    let mut playlist = Playlist::default();
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
    assert_eq!(playlist.items.len(), 3);
    assert_eq!(playlist.items[0].id, "a");
    assert_eq!(playlist.items[1].id, "b");
    assert_eq!(playlist.items[2].id, "c");
}
```

- [x] **Write test**: `test_playlist_remove_item_by_id`
- [x] **Write test**: `test_playlist_remove_nonexistent_returns_false`
- [x] **Run tests** - fail
- [x] **Implement**: `remove_item()` method
- [x] **Run tests** - pass

```rust
#[test]
fn test_playlist_remove_item_by_id() {
    let mut playlist = Playlist::default();
    playlist.add_item(PlaylistItem {
        id: "abc1".to_string(),
        widget: "weather".to_string(),
        input: json!(null),
    });
    playlist.add_item(PlaylistItem {
        id: "def2".to_string(),
        widget: "text".to_string(),
        input: json!("hello"),
    });

    let removed = playlist.remove_item("abc1");
    assert!(removed);
    assert_eq!(playlist.items.len(), 1);
    assert_eq!(playlist.items[0].id, "def2");
}

#[test]
fn test_playlist_remove_nonexistent_returns_false() {
    let mut playlist = Playlist::default();
    playlist.add_item(PlaylistItem {
        id: "abc1".to_string(),
        widget: "weather".to_string(),
        input: json!(null),
    });

    let removed = playlist.remove_item("nonexistent");
    assert!(!removed);
    assert_eq!(playlist.items.len(), 1);
}

#[test]
fn test_playlist_remove_from_empty_returns_false() {
    let mut playlist = Playlist::default();
    assert!(playlist.is_empty());

    let removed = playlist.remove_item("any_id");
    assert!(!removed);
    assert!(playlist.is_empty());
}
```

- [x] **Write test**: `test_playlist_is_empty`
- [x] **Run test** - fails
- [x] **Implement**: `is_empty()` method
- [x] **Run test** - passes

```rust
#[test]
fn test_playlist_is_empty() {
    let playlist = Playlist::default();
    assert!(playlist.is_empty());

    let mut playlist_with_items = Playlist::default();
    playlist_with_items.add_item(PlaylistItem {
        id: "abc1".to_string(),
        widget: "weather".to_string(),
        input: json!(null),
    });
    assert!(!playlist_with_items.is_empty());
}
```

**Step 1.2.4: Interval validation**

- [x] **Write test**: `test_playlist_validate_interval_rejects_under_60`
- [x] **Write test**: `test_playlist_validate_interval_accepts_60_and_above`
- [x] **Run tests** - fail
- [x] **Implement**: `validate_interval()` method
- [x] **Run tests** - pass
- [x] **Commit**: "Add Playlist CRUD and validation"

```rust
#[test]
fn test_playlist_validate_interval_rejects_under_60() {
    let mut playlist = Playlist::default();
    playlist.interval_seconds = 59;
    let result = playlist.validate_interval();
    assert!(result.is_err());
}

#[test]
fn test_playlist_validate_interval_accepts_60() {
    let mut playlist = Playlist::default();
    playlist.interval_seconds = 60;
    let result = playlist.validate_interval();
    assert!(result.is_ok());
}

#[test]
fn test_playlist_validate_interval_accepts_300() {
    let playlist = Playlist::default(); // defaults to 300
    let result = playlist.validate_interval();
    assert!(result.is_ok());
}
```

**Step 1.2.5: File persistence**

- [x] **Write test**: `test_playlist_save_and_load`
- [x] **Write test**: `test_playlist_load_nonexistent_returns_default`
- [x] **Write test**: `test_playlist_load_invalid_json_returns_error`
- [x] **Run tests** - fail
- [x] **Implement**: `save()` and `load()` functions
- [x] **Run tests** - pass
- [x] **Commit**: "Add Playlist file persistence"

```rust
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_playlist_save_and_load() {
    let mut playlist = Playlist::default();
    playlist.interval_seconds = 120;
    playlist.add_item(PlaylistItem {
        id: "abc1".to_string(),
        widget: "weather".to_string(),
        input: json!(null),
    });

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    playlist.save(path).unwrap();
    let loaded = Playlist::load(path).unwrap();

    assert_eq!(loaded.interval_seconds, 120);
    assert_eq!(loaded.items.len(), 1);
    assert_eq!(loaded.items[0].widget, "weather");
}

#[test]
fn test_playlist_load_nonexistent_returns_default() {
    let path = std::path::Path::new("/nonexistent/path/playlist.json");
    let playlist = Playlist::load(path).unwrap();
    assert!(playlist.items.is_empty());
    assert_eq!(playlist.interval_seconds, 300);
}

#[test]
fn test_playlist_load_invalid_json_returns_error() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "not valid json {{{{").unwrap();

    let result = Playlist::load(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_playlist_load_empty_file_returns_default() {
    let temp_file = NamedTempFile::new().unwrap();
    // File exists but is empty

    let playlist = Playlist::load(temp_file.path()).unwrap();
    assert!(playlist.items.is_empty());
}
```

### 1.3 Create RuntimeState

- [x] **Create test file section** in `src/tests/playlist_tests.rs` or separate file
- [x] **Write test**: `test_runtime_state_default_values`
- [x] **Write test**: `test_runtime_state_save_and_load`
- [x] **Write test**: `test_runtime_state_load_missing_file_returns_default`
- [x] **Write test**: `test_runtime_state_load_corrupted_file_returns_default`
- [x] **Run tests** - fail
- [x] **Create file**: `src/runtime_state.rs`
- [x] **Implement**: `RuntimeState` struct with `load()` and `save()`
- [x] **Run tests** - pass
- [x] **Commit**: "Add RuntimeState for playlist persistence"

```rust
// src/tests/runtime_state_tests.rs
use crate::runtime_state::{RuntimeState, PlaylistState};
use chrono::Utc;
use tempfile::NamedTempFile;
use std::io::Write;

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
fn test_runtime_state_save_and_load() {
    let mut state = RuntimeState::default();
    state.playlist_state = PlaylistState::Paused;
    state.playlist_index = 5;
    state.last_shown_time = Some(Utc::now());

    let temp_file = NamedTempFile::new().unwrap();
    state.save(temp_file.path());  // save() is infallible

    let loaded = RuntimeState::load(temp_file.path());  // load() is infallible
    assert_eq!(loaded.playlist_state, PlaylistState::Paused);
    assert_eq!(loaded.playlist_index, 5);
    assert!(loaded.last_shown_time.is_some());
}

#[test]
fn test_runtime_state_load_missing_file_returns_default() {
    let path = std::path::Path::new("/nonexistent/runtime_state.json");
    let state = RuntimeState::load(path);  // Returns default, not error
    assert_eq!(state.playlist_state, PlaylistState::Stopped);
    assert_eq!(state.playlist_index, 0);
}

#[test]
fn test_runtime_state_load_corrupted_file_returns_default() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "{{invalid json").unwrap();

    let state = RuntimeState::load(temp_file.path());  // Returns default on parse error
    assert_eq!(state.playlist_state, PlaylistState::Stopped);
}

#[test]
fn test_playlist_state_serialization() {
    let state = PlaylistState::Running;
    let json = serde_json::to_string(&state).unwrap();
    assert_eq!(json, "\"Running\"");

    let deserialized: PlaylistState = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, PlaylistState::Running);
}
```

### 1.4 Create InstanceLock

- [x] **Create test file**: `src/tests/lock_tests.rs`
- [x] **Write test**: `test_lock_acquires_when_no_existing_lock`
- [x] **Write test**: `test_lock_fails_when_lock_exists_and_pid_running`
- [x] **Write test**: `test_lock_succeeds_when_lock_stale`
- [x] **Write test**: `test_lock_released_on_drop`
- [x] **Run tests** - fail
- [x] **Create file**: `src/runner/lock.rs`
- [x] **Implement**: `InstanceLock` struct
- [x] **Run tests** - pass
- [x] **Commit**: "Add InstanceLock for single-instance enforcement"

```rust
// src/tests/lock_tests.rs
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
    assert!(content.contains("\"mode\":\"playlist\""));
    assert!(content.contains("\"pid\":"));
    assert!(content.contains("\"started_at\":"));
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

    drop(lock1); // Clean up
}

#[test]
fn test_lock_succeeds_when_lock_file_has_dead_pid() {
    let temp_dir = tempdir().unwrap();
    let lock_path = temp_dir.path().join("test.lock");

    // Write a lock file with a PID that doesn't exist
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
```

### 1.6 Create FileMonitor (Generic)

- [ ] **Create test file**: `src/tests/file_monitor_tests.rs`
- [ ] **Write test**: `test_file_monitor_detects_changes`
- [ ] **Write test**: `test_file_monitor_handles_missing_file`
- [ ] **Write test**: `test_file_monitor_handles_invalid_json`
- [ ] **Run tests** - fail
- [ ] **Create file**: `src/file_monitor.rs`
- [ ] **Implement**: `FileMonitor<T>` struct
- [ ] **Run tests** - pass
- [ ] **Commit**: "Add generic FileMonitor"

```rust
// src/tests/file_monitor_tests.rs
use crate::file_monitor::FileMonitor;
use crate::playlist::Playlist;
use serde_json::json;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_file_monitor_loads_initial_data() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, r#"{{"interval_seconds": 120, "items": []}}"#).unwrap();

    let mut monitor: FileMonitor<Playlist> = FileMonitor::new(temp_file.path());
    monitor.initialize().unwrap();

    let playlist = monitor.get_current();
    assert_eq!(playlist.interval_seconds, 120);
}

#[test]
fn test_file_monitor_detects_changes() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    // Write initial content
    fs::write(&path, r#"{"interval_seconds": 120, "items": []}"#).unwrap();

    let mut monitor: FileMonitor<Playlist> = FileMonitor::new(&path);
    monitor.initialize().unwrap();

    // No changes yet
    assert!(!monitor.reload_if_modified().unwrap());

    // Modify the file (need a small delay for filesystem)
    std::thread::sleep(std::time::Duration::from_millis(100));
    fs::write(&path, r#"{"interval_seconds": 300, "items": []}"#).unwrap();

    // Should detect change
    assert!(monitor.reload_if_modified().unwrap());
    assert_eq!(monitor.get_current().interval_seconds, 300);
}

#[test]
fn test_file_monitor_handles_missing_file() {
    let path = std::path::Path::new("/nonexistent/file.json");
    let mut monitor: FileMonitor<Playlist> = FileMonitor::new(path);

    monitor.initialize().unwrap();
    let playlist = monitor.get_current();

    // Should use default values
    assert!(playlist.items.is_empty());
    assert_eq!(playlist.interval_seconds, 300);
}

#[test]
fn test_file_monitor_keeps_old_data_on_read_error() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    // Write valid initial content
    fs::write(&path, r#"{"interval_seconds": 120, "items": []}"#).unwrap();

    let mut monitor: FileMonitor<Playlist> = FileMonitor::new(&path);
    monitor.initialize().unwrap();
    assert_eq!(monitor.get_current().interval_seconds, 120);

    // Corrupt the file
    std::thread::sleep(std::time::Duration::from_millis(100));
    fs::write(&path, "not valid json").unwrap();

    // Reload should fail gracefully, keeping old data
    let _ = monitor.reload_if_modified();
    assert_eq!(monitor.get_current().interval_seconds, 120);
}
```

### 1.7 Create Runner module structure

- [x] **Create directory**: `src/runner/`
- [x] **Create file**: `src/runner/mod.rs`
- [x] **Write test**: `test_control_flow_variants`
- [x] **Implement**: `ControlFlow` enum, `Runner` trait
- [x] **Run tests** - pass
- [x] **Commit**: "Add Runner trait and module structure"

```rust
// In src/runner/mod.rs
pub mod lock;
pub mod keyboard;

use async_trait::async_trait;
use crossterm::event::KeyCode;
use crate::errors::VestaboardError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlow {
    Continue,
    Exit,
}

#[async_trait]
pub trait Runner {
    async fn run_iteration(&mut self) -> Result<(), VestaboardError>;
    fn handle_key(&mut self, key: KeyCode) -> ControlFlow;
    fn help_text(&self) -> &'static str;
    fn cleanup(&mut self);
}

// Test in src/tests/runner_tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_flow_variants() {
        let cont = ControlFlow::Continue;
        let exit = ControlFlow::Exit;
        assert_ne!(cont, exit);
    }
}
```

### 1.8 Create MockInput for testing

- [x] **Write test**: `test_mock_input_provides_keys_in_order`
- [x] **Write test**: `test_mock_input_returns_none_when_exhausted`
- [x] **Implement**: `InputSource` trait and `MockInput` in `src/runner/keyboard.rs`
- [x] **Run tests** - pass
- [x] **Commit**: "Add InputSource trait and MockInput for testing"

```rust
// src/tests/keyboard_tests.rs
use crate::runner::keyboard::{InputSource, MockInput};
use crossterm::event::KeyCode;

#[test]
fn test_mock_input_provides_keys_in_order() {
    let mut mock = MockInput::new(vec![
        KeyCode::Char('p'),
        KeyCode::Char('r'),
        KeyCode::Char('q'),
    ]);

    assert_eq!(mock.next_key(), Some(KeyCode::Char('p')));
    assert_eq!(mock.next_key(), Some(KeyCode::Char('r')));
    assert_eq!(mock.next_key(), Some(KeyCode::Char('q')));
}

#[test]
fn test_mock_input_returns_none_when_exhausted() {
    let mut mock = MockInput::new(vec![KeyCode::Char('q')]);

    assert_eq!(mock.next_key(), Some(KeyCode::Char('q')));
    assert_eq!(mock.next_key(), None);
    assert_eq!(mock.next_key(), None);
}

#[test]
fn test_mock_input_empty_returns_none() {
    let mut mock = MockInput::new(vec![]);
    assert_eq!(mock.next_key(), None);
}
```

### 1.9 Update Config

- [x] **Write test**: `test_config_loads_playlist_path`
- [x] **Write test**: `test_config_defaults_for_new_fields`
- [x] **Add fields**: `playlist_file_path`, `runtime_state_path`, `lock_file_path`
- [x] **Run tests** - pass
- [x] **Commit**: "Add playlist config fields"

```rust
// Add to src/tests/config_tests.rs
#[test]
fn test_config_provides_playlist_file_path() {
    let config = Config::load_silent().unwrap();
    let path = config.get_playlist_file_path();
    assert!(path.to_string_lossy().contains("playlist.json"));
}

#[test]
fn test_config_provides_runtime_state_path() {
    let config = Config::load_silent().unwrap();
    let path = config.get_runtime_state_path();
    assert!(path.to_string_lossy().contains("runtime_state.json"));
}

#[test]
fn test_config_provides_lock_file_path() {
    let config = Config::load_silent().unwrap();
    let path = config.get_lock_file_path();
    assert!(path.to_string_lossy().contains("vestaboard.lock"));
}
```

---

## Phase 1 Definition of Done

**All of the following must be true:**

- [x] `cargo test playlist` - all tests pass
- [x] `cargo test runtime_state` - all tests pass
- [x] `cargo test lock` - all tests pass
- [ ] `cargo test file_monitor` - all tests pass *(FileMonitor deferred to later phase)*
- [x] `cargo test config` - all tests pass (including new tests)
- [x] `cargo build` - compiles without errors
- [x] `cargo clippy` - no errors (dead code warnings expected for foundation code)
- [x] Existing commands (`vbl show`, `vbl schedule`, `vbl daemon`, `vbl cycle`) work unchanged
- [x] All new modules are documented with `///` doc comments
- [ ] Code coverage for new modules > 80%

**Test count checkpoint**: Phase 1 added 52 new tests (playlist: 33, runtime_state: 11, lock: 8).

---

## Next Phase

Continue to [02-phase-2-cli.md](02-phase-2-cli.md) for CLI implementation.
