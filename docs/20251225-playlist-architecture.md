# Playlist Feature Architecture

## Overview

This document defines the architecture for the Playlist feature, which enables interval-based rotation of content on the Vestaboard. It addresses the conceptual separation between time-triggered schedules and interval-based content rotation, defines behavior specifications, and outlines the migration path from the current implementation.

**This document is self-contained** - it includes all decisions, implementation details, and code patterns needed to implement the feature.

---

## Problem Statement

### Current State

The application currently has three execution modes:

1. **Show mode** (`vbl show`): One-shot message display
2. **Daemon mode** (`vbl daemon`): Executes tasks at their scheduled times
3. **Cycle mode** (`vbl cycle`): Rotates through tasks at fixed intervals

The cycle mode reuses `schedule.json` but ignores the time fields, creating semantic confusion:
- Users schedule tasks with specific times, but cycle mode discards this information
- Two different execution semantics share one data model
- The relationship between daemon and cycle modes is unclear

### Target State

- **Schedule**: Time-triggered task execution (when to show something)
- **Playlist**: Interval-based content rotation (what to rotate through)
- **Mutual exclusivity**: Only one can run at a time (no collision handling needed)
- **No daemon concept**: Replace with `vbl schedule run` and `vbl playlist run`

---

## Module Structure

### New Modules

```
src/
├── runner/                      # NEW: Execution runner framework
│   ├── mod.rs                   # Runner trait, shared utilities, re-exports
│   ├── lock.rs                  # Instance lock (prevents multiple runs)
│   ├── keyboard.rs              # Keyboard input handling (crossterm)
│   ├── schedule_runner.rs       # vbl schedule run implementation
│   └── playlist_runner.rs       # vbl playlist run implementation
├── playlist.rs                  # NEW: Playlist data model + CRUD operations
├── runtime_state.rs             # NEW: Runtime state persistence
├── file_monitor.rs              # REFACTOR: Extract generic monitor from ScheduleMonitor
└── ... (existing modules unchanged)
```

### Layer Mapping

Following the existing layer structure from `DEVELOPMENT_GUIDE.md`:

| Layer | New Modules | Responsibility |
|-------|-------------|----------------|
| **UI Layer** | CLI additions in `cli_setup.rs` | Parse playlist/schedule subcommands |
| **Execution Layer** | `runner/*`, `playlist.rs`, `runtime_state.rs` | Run playlists/schedules, manage state |
| **Widgets Module** | (unchanged) | Content generation via resolver |
| **Translation Layer** | (unchanged) | Message-to-code conversion |
| **Communication Layer** | (unchanged) | Vestaboard API calls |

---

## Command Structure

### Design Principle

All commands related to a feature are grouped under that feature's namespace. This provides discoverability - typing `vbl playlist --help` shows everything you can do with playlists.

### Full Command Reference

```bash
# One-shot display
vbl show text "hello"
vbl show weather
vbl show sat-word
vbl show jokes
vbl show file <path>
vbl show clear

# Schedule management
vbl schedule add "2025-01-15 08:00" text "good morning"
vbl schedule add "2025-01-15 18:00" weather
vbl schedule list
vbl schedule remove <id>
vbl schedule clear
vbl schedule preview              # Dry-run all scheduled items

# Schedule execution
vbl schedule run                  # Long-running: execute at scheduled times

# Playlist management
vbl playlist add weather
vbl playlist add text "welcome"
vbl playlist add sat-word
vbl playlist list
vbl playlist remove <id>
vbl playlist clear
vbl playlist interval 300         # Set rotation interval (seconds)
vbl playlist preview              # Dry-run all playlist items

# Playlist execution
vbl playlist run                  # Long-running: rotate through items (loops forever)
vbl playlist run --once           # Run through playlist once, then exit
vbl playlist run --index 3        # Start from index 3
vbl playlist run --id abc1        # Start from item with id "abc1"
```

### Deprecated Commands

The following commands will be removed:

```bash
vbl daemon              # Use: vbl schedule run
vbl cycle               # Use: vbl playlist run
vbl cycle repeat        # Use: vbl playlist run
```

---

## Interactive Keyboard Controls

Both `vbl schedule run` and `vbl playlist run` are foreground processes with interactive keyboard controls.

### Playlist Controls

```
$ vbl playlist run
Running playlist (5 min interval). Press ? for help.

[14:32:01] weather
[14:37:01] text "welcome"
?
  p - pause    r - resume    n - next    q - quit    ? - help
[14:42:01] sat-word
p
Paused. Press r to resume, q to quit.
r
Resumed.
n
[14:42:15] jokes (skipped ahead)
q
Playlist stopped.
$
```

| Key | Action | Description |
|-----|--------|-------------|
| `p` | Pause | Stop rotation, remember position |
| `r` | Resume | Continue from paused position |
| `n` | Next | Skip to next item immediately |
| `q` | Quit | Exit cleanly |
| `?` | Help | Show available commands |

### Schedule Controls

```
$ vbl schedule run
Running schedule. Press ? for help.

Waiting for: 18:00 weather
[18:00:00] weather
Waiting for: 22:00 text "goodnight"
q
Schedule stopped.
$
```

| Key | Action | Description |
|-----|--------|-------------|
| `q` | Quit | Exit cleanly |
| `?` | Help | Show available commands |

**Note**: Schedule does not have pause/resume. If you need to stop, quit and restart. Pausing a schedule creates complexity around missed tasks that is not worth the added functionality.

---

## Execution Model

### Foreground Processes (v1)

Both `vbl schedule run` and `vbl playlist run` are **foreground processes** that require an attached terminal:

- Run in the terminal where the command is executed
- Keyboard controls work via stdin
- Ctrl+C triggers graceful shutdown (same as pressing `q`)
- For persistence, use `screen`, `tmux`, or `nohup`

**Future consideration**: A proper daemon/service architecture with socket-based IPC may be added in a future version if headless operation becomes a priority.

### Instance Prevention (Lock File)

Only one instance of `run` can be active at a time. A lock file prevents multiple instances:

**File**: `data/vestaboard.lock`

```json
{
  "mode": "playlist",
  "pid": 12345,
  "started_at": "2025-01-15T14:32:01Z"
}
```

| Action | Lock State | Result |
|--------|------------|--------|
| `vbl playlist run` | No lock | Creates lock, starts playlist |
| `vbl schedule run` | No lock | Creates lock, starts schedule |
| `vbl playlist run` | Lock exists (playlist) | Error: "Playlist already running (PID 12345, started 14:32:01)" |
| `vbl schedule run` | Lock exists (playlist) | Error: "Playlist is running. Stop it first." |
| Process exits | Lock exists | Lock file removed |

**Stale lock detection**: If the PID in the lock file is no longer running, the lock is considered stale and will be overwritten.

**Lock file error handling**:

| Error | Behavior |
|-------|----------|
| Can't create lock file (permissions) | Fatal error: "Cannot create lock file: permission denied" |
| Can't read existing lock file | Treat as no lock (overwrite) with warning |
| Lock file exists, PID still running | Error with helpful message showing PID and start time |
| Lock file exists, PID not running | Stale lock, overwrite and continue |

### Cross-Terminal Control

**Data commands** modify files and work from any terminal:

```
Terminal 1:                         Terminal 2:
$ vbl playlist run                  $ vbl playlist add weather
Running playlist...                 Added weather to playlist.
[file change detected, reloads]
[14:32] weather                     $ vbl playlist remove abc1
                                    Removed abc1 from playlist.
[file change detected, reloads]
```

The running process watches for file changes (via `FileMonitor`) and automatically reloads.

**Process commands** are keyboard-only in the original terminal:

| Command Type | Examples | Cross-Terminal? |
|--------------|----------|-----------------|
| Data commands | `add`, `remove`, `list`, `clear`, `interval`, `preview` | Yes (via file) |
| Process commands | `run`, `pause`, `resume`, `next`, `quit` | No (keyboard only) |

If you try to run a process command from another terminal:

```bash
$ vbl playlist pause
Error: Playlist is running in another terminal.
       Use keyboard controls (press 'p') in that terminal to pause.
```

### Empty Playlist/Schedule

If `run` is executed with no items:

```bash
$ vbl playlist run
No items in playlist. Add items with 'vbl playlist add <widget>'.
$
```

The command shows a message and exits immediately. It does not wait or loop.

### Preview Behavior

`vbl playlist preview` and `vbl schedule preview` show all items **without delays**:

```bash
$ vbl playlist preview
Previewing 3 playlist items:

[1/3] weather
┌──────────────────────┐
│ SUNNY 72°F           │
│ ...                  │
└──────────────────────┘

[2/3] text "welcome"
┌──────────────────────┐
│ WELCOME              │
│ ...                  │
└──────────────────────┘

[3/3] sat-word
┌──────────────────────┐
│ EPHEMERAL            │
│ ...                  │
└──────────────────────┘

Preview complete.
$
```

---

## Mutual Exclusivity

### Design Decision

Schedule and Playlist cannot run simultaneously. Only one can be active at a time.

### Rationale

- **Simplicity**: No collision handling, priority system, or arbitration needed
- **Clear mental model**: One thing controls the board at a time
- **Easier debugging**: You always know what's controlling the display

### Behavior

| Action | Current State | Result |
|--------|---------------|--------|
| `vbl playlist run` | Nothing running | Playlist starts |
| `vbl schedule run` | Nothing running | Schedule starts |
| `vbl playlist run` | Schedule running | Error: "Schedule is running. Stop it first." |
| `vbl schedule run` | Playlist running | Error: "Playlist is running. Stop it first." |

---

## Data Models

### Schedule (existing, unchanged)

**File**: `data/schedule.json`

```json
{
  "tasks": [
    {
      "id": "abc1",
      "time": "2025-01-15T08:00:00Z",
      "widget": "text",
      "input": "good morning"
    },
    {
      "id": "def2",
      "time": "2025-01-15T18:00:00Z",
      "widget": "weather",
      "input": null
    }
  ]
}
```

### Playlist (new)

**File**: `data/playlist.json`

```json
{
  "interval_seconds": 300,
  "items": [
    {
      "id": "item1",
      "widget": "weather",
      "input": null
    },
    {
      "id": "item2",
      "widget": "text",
      "input": "welcome to our office"
    },
    {
      "id": "item3",
      "widget": "sat-word",
      "input": null
    }
  ]
}
```

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `interval_seconds` | u64 | Time between rotations (minimum: 60, default: 300) |
| `items` | array | Playlist entries (executed in order) |
| `items[].id` | string | Unique identifier (auto-generated, same as schedule) |
| `items[].widget` | string | Widget type to execute |
| `items[].input` | Value | Widget input (widget-specific) |

**Note**: No `enabled` flag, no `mode` (sequential only for v1), no `active_window`. These are deferred to future versions.

### Runtime State (new)

**File**: `data/runtime_state.json`

```json
{
  "playlist_state": "Paused",
  "playlist_index": 3,
  "last_shown_time": "2025-01-15T14:32:00Z"
}
```

**Purpose**: Persists state across restarts for continuity.

| Field | Type | Description |
|-------|------|-------------|
| `playlist_state` | enum | `Stopped`, `Running`, or `Paused` (capitalized for serde) |
| `playlist_index` | usize | Current position in playlist |
| `last_shown_time` | DateTime<Utc> | When last message was displayed |

**When state is saved**: On every rotation, **before** each item is displayed. This ensures that if the process crashes during display or API call, the state reflects the item that was about to be shown, and on restart, we'll retry that item rather than skip it.

**State save timing**:
1. Advance index to next item
2. Save state (index + timestamp)
3. Display item on Vestaboard
4. Wait for interval

This order ensures we never "lose" an item display due to a crash. In the worst case, an item might be shown twice (if crash happens after display but before interval), which is acceptable.

**Note**: This is not a high-precision timing application. Saving on rotation is sufficient.

**State file error handling**:

| Error | Behavior |
|-------|----------|
| File doesn't exist | Create with defaults, continue |
| Can't write (disk full, permissions) | Log warning, continue without saving (don't crash) |
| Invalid JSON on load | Reset to defaults, log warning |

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
              ┌─────────────────────────────────┐
              │                                 │
              ▼                                 │
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

## Schedule Execution Behavior

### Overdue Messages: Never Execute

**Rule**: When schedule starts (or restarts), skip all past-due tasks. Wait for the next upcoming task.

```
Schedule:
  - 08:00 "good morning"
  - 12:00 "lunch time"
  - 18:00 "dinner time"

Schedule starts at 14:30:
  - Skip 08:00 (past)
  - Skip 12:00 (past)
  - Wait for 18:00
```

**Rationale**:
- Stale messages (hours or days old) are not useful
- Eliminates "catch up" behavior after restart
- Simple and predictable

### Clock Handling

| Context | Clock Type | Rationale |
|---------|------------|-----------|
| Schedule comparison | Wall clock (UTC) | Schedules specify wall clock times |
| Playlist intervals | Monotonic (`Instant`) | Intervals should be consistent elapsed time |

**Important**: `Instant` cannot be serialized. For state persistence:
- Save `last_shown_time` as `DateTime<Utc>` (wall clock)
- On resume, calculate elapsed time since `last_shown_time`
- If elapsed >= interval, show next item immediately
- Otherwise, wait for remaining time

**Backward clock jump for schedule**: If the clock jumps backward and a previously-executed task becomes "due" again, it will re-execute. This is acceptable - it indicates something unusual happened with the system clock, and re-showing a message is not harmful.

---

## Runner Framework

### Runner Trait

Both schedule and playlist runners share common patterns. Define a trait:

```rust
use async_trait::async_trait;
use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlow {
    Continue,
    Exit,
}

#[async_trait]
pub trait Runner {
    /// Called once when the runner starts
    fn start(&mut self);

    /// Run one iteration of the runner (check if work needs to be done, do it)
    /// Returns quickly if nothing to do (non-blocking)
    async fn run_iteration(&mut self) -> Result<(), VestaboardError>;

    /// Handle a keyboard input, return whether to continue
    fn handle_key(&mut self, key: KeyCode) -> ControlFlow;

    /// Get help text for keyboard controls
    fn help_text(&self) -> &'static str;

    /// Called on graceful shutdown
    fn cleanup(&mut self);
}
```

### Help Text Constants

Define the help text as constants so they're consistent and testable:

```rust
/// Help text for playlist runner
pub const PLAYLIST_HELP: &str = "\
Playlist Controls:
  p - Pause rotation
  r - Resume rotation
  n - Skip to next item
  q - Quit
  ? - Show this help";

/// Help text for schedule runner
pub const SCHEDULE_HELP: &str = "\
Schedule Controls:
  q - Quit
  ? - Show this help";

// Implementation example for PlaylistRunner
impl Runner for PlaylistRunner {
    fn help_text(&self) -> &'static str {
        PLAYLIST_HELP
    }
    // ... other methods
}

// Implementation example for ScheduleRunner
impl Runner for ScheduleRunner {
    fn help_text(&self) -> &'static str {
        SCHEDULE_HELP
    }
    // ... other methods
}
```

### Common Runner Setup

The main loop uses the existing `ProcessController` pattern from `process_control.rs` for signal handling, combined with keyboard input and runner iterations. This matches the established patterns in `daemon.rs` and `cycle.rs`.

**Important**: This design follows the existing ProcessController API exactly - it uses `should_shutdown()` for polling, not channels.

```rust
use crate::process_control::ProcessController;
use crate::runner::{lock::InstanceLock, keyboard::KeyboardListener};
use crate::cli_display::{print_progress, print_success};
use std::time::Duration;

pub async fn run_with_keyboard<R: Runner>(
    mut runner: R,
    mode: &str,  // "playlist" or "schedule"
) -> Result<(), VestaboardError> {
    log::info!("Starting {} runner", mode);

    // Acquire exclusive lock (prevents multiple instances)
    let _lock = InstanceLock::acquire(mode)?;

    // Setup keyboard listener (spawns background thread)
    let mut keyboard = KeyboardListener::new()?;

    // Setup Ctrl+C handler using existing ProcessController pattern
    // Note: ProcessController.setup_signal_handler() returns Result<(), VestaboardError>
    // and uses should_shutdown() for polling - NOT channels
    let process_controller = ProcessController::new();
    process_controller.setup_signal_handler()?;

    // Show initial help
    println!("Press ? for help.");

    // Start the runner
    runner.start();
    print_progress(&format!("Running {}...", mode));

    // Main loop - polling pattern matching existing daemon.rs/cycle.rs
    loop {
        // Priority 1: Check for shutdown signal (Ctrl+C)
        if process_controller.should_shutdown() {
            log::info!("Shutdown requested, stopping {}", mode);
            println!("\nShutting down...");
            break;
        }

        // Priority 2: Check for keyboard input (non-blocking)
        if let Some(key) = keyboard.try_recv() {
            match runner.handle_key(key) {
                ControlFlow::Continue => {}
                ControlFlow::Exit => {
                    log::info!("User requested exit via keyboard");
                    break;
                }
            }
        }

        // Priority 3: Run one iteration of the runner
        // This may display a message if interval has elapsed
        if let Err(e) = runner.run_iteration().await {
            log::error!("Runner error: {}", e);
            eprintln!("Error: {}", e);
            // Continue running unless fatal - matches cycle.rs behavior
        }

        // Small sleep to prevent busy-looping (matches daemon.rs pattern)
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Cleanup
    runner.cleanup();
    print_success(&format!("{} stopped.", mode));

    // Lock is automatically released when _lock is dropped (RAII)
    Ok(())
}
```

### KeyboardListener Implementation

Uses `std::sync::mpsc` with a background thread, providing a non-blocking `try_recv()` method that integrates well with the polling loop:

```rust
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct KeyboardListener {
    receiver: Receiver<KeyCode>,
    _handle: JoinHandle<()>,
}

impl KeyboardListener {
    pub fn new() -> Result<Self, VestaboardError> {
        // Check if stdin is a TTY (required for interactive mode)
        if !atty::is(atty::Stream::Stdin) {
            return Err(VestaboardError::input_error(
                "Interactive mode requires a terminal. Stdin is not a TTY."
            ));
        }

        let (sender, receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            loop {
                // Poll for keyboard events with short timeout
                if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                    if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                        if sender.send(code).is_err() {
                            break; // Receiver dropped, exit thread
                        }
                    }
                }
            }
        });

        log::debug!("KeyboardListener started");
        Ok(Self { receiver, _handle: handle })
    }

    /// Non-blocking check for keyboard input
    pub fn try_recv(&mut self) -> Option<KeyCode> {
        match self.receiver.try_recv() {
            Ok(key) => Some(key),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                log::warn!("Keyboard input thread disconnected");
                None
            }
        }
    }
}
```

### Instance Lock Implementation

**Important**: This implementation uses OS-level file locking (`flock` on Unix, `LockFile` on Windows) to prevent race conditions. The JSON data inside the lock file is for informational purposes only - the actual locking is done by the OS.

```rust
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
struct LockData {
    mode: String,
    pid: u32,
    started_at: DateTime<Utc>,
}

pub struct InstanceLock {
    path: PathBuf,
    file: File,  // Keep file open to maintain OS lock (flock on Unix, LockFileEx on Windows)
}

impl InstanceLock {
    /// Acquire an exclusive lock using OS-level file locking.
    /// This is atomic and race-condition free.
    pub fn acquire(mode: &str) -> Result<Self, VestaboardError> {
        Self::acquire_at(mode, &PathBuf::from("data/vestaboard.lock"))
    }

    /// Acquire lock at a specific path (useful for testing)
    pub fn acquire_at(mode: &str, path: &PathBuf) -> Result<Self, VestaboardError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| VestaboardError::lock_error(
                &format!("Cannot create lock directory: {}", e)
            ))?;
        }

        // Open or create the lock file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|e| VestaboardError::lock_error(
                &format!("Cannot open lock file: {}", e)
            ))?;

        // Try to acquire exclusive lock (non-blocking)
        if !Self::try_exclusive_lock(&file)? {
            // Lock is held by another process - read the lock data for error message
            let content = fs::read_to_string(path).unwrap_or_default();
            if let Ok(lock_data) = serde_json::from_str::<LockData>(&content) {
                return Err(VestaboardError::lock_error(&format!(
                    "{} already running (PID {}, started {})",
                    lock_data.mode,
                    lock_data.pid,
                    lock_data.started_at.format("%H:%M:%S")
                )));
            } else {
                return Err(VestaboardError::lock_error(
                    "Another instance is already running"
                ));
            }
        }

        // We have the lock - write our info to the file
        let lock_data = LockData {
            mode: mode.to_string(),
            pid: std::process::id(),
            started_at: Utc::now(),
        };

        let content = serde_json::to_string_pretty(&lock_data)
            .map_err(|e| VestaboardError::lock_error(
                &format!("Cannot serialize lock: {}", e)
            ))?;

        // Truncate and write (we hold the lock, so this is safe)
        let mut file = file;
        file.set_len(0).ok();
        file.seek(SeekFrom::Start(0)).ok();
        file.write_all(content.as_bytes()).map_err(|e| VestaboardError::lock_error(
            &format!("Cannot write lock file: {}", e)
        ))?;
        file.flush().ok();

        Ok(Self { path: path.clone(), file })
    }

    #[cfg(unix)]
    fn try_exclusive_lock(file: &File) -> Result<bool, VestaboardError> {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        // LOCK_EX = exclusive lock, LOCK_NB = non-blocking
        let result = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
        if result == 0 {
            Ok(true)  // Lock acquired
        } else {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::WouldBlock {
                Ok(false)  // Lock held by another process
            } else {
                Err(VestaboardError::lock_error(&format!("flock failed: {}", err)))
            }
        }
    }

    #[cfg(windows)]
    fn try_exclusive_lock(file: &File) -> Result<bool, VestaboardError> {
        use std::os::windows::io::AsRawHandle;
        use winapi::um::fileapi::LockFileEx;
        use winapi::um::minwinbase::{LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY, OVERLAPPED};
        use std::mem::zeroed;

        let handle = file.as_raw_handle();
        let mut overlapped: OVERLAPPED = unsafe { zeroed() };

        let result = unsafe {
            LockFileEx(
                handle as *mut _,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                1,  // Lock 1 byte
                0,
                &mut overlapped,
            )
        };

        if result != 0 {
            Ok(true)  // Lock acquired
        } else {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(33) {  // ERROR_LOCK_VIOLATION
                Ok(false)  // Lock held by another process
            } else {
                Err(VestaboardError::lock_error(&format!("LockFileEx failed: {}", err)))
            }
        }
    }

    // Note: is_pid_running() is NOT needed because OS-level flock/LockFile
    // automatically releases the lock when the process dies. Stale lock
    // detection is handled by the OS, not by PID checking.
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        // Lock is automatically released when file is closed (on drop)
        // Remove the lock file for cleanliness
        if let Err(e) = fs::remove_file(&self.path) {
            log::warn!("Cannot remove lock file: {}", e);
        }
    }
}
```

### Input Source Trait and MockInput

The `InputSource` trait abstracts keyboard input for testing. `MockInput` is available in both unit tests and integration tests:

```rust
// For testing - abstract input source
pub trait InputSource: Send {
    fn try_recv(&mut self) -> Option<KeyCode>;
}

impl InputSource for KeyboardListener {
    fn try_recv(&mut self) -> Option<KeyCode> {
        self.try_recv()
    }
}

// MockInput for tests - NOT cfg(test) so it's available for integration tests
// Use feature flag or always include since it has no runtime cost
pub struct MockInput {
    keys: std::collections::VecDeque<KeyCode>,
}

impl MockInput {
    pub fn new(keys: Vec<KeyCode>) -> Self {
        Self {
            keys: keys.into(),
        }
    }

    pub fn with_keys(keys: &[KeyCode]) -> Self {
        Self {
            keys: keys.iter().copied().collect(),
        }
    }
}

impl InputSource for MockInput {
    fn try_recv(&mut self) -> Option<KeyCode> {
        self.keys.pop_front()
    }
}
```

**Note**: `MockInput` is NOT marked `#[cfg(test)]` because it's needed for integration tests that run in non-test compilation contexts. It has zero runtime cost when not used.

### PlaylistRunner Implementation

This is the core runner that integrates with existing infrastructure. **Critical**: Uses `execute_widget()` from `widgets/resolver.rs` and `handle_message()` from `api_broker.rs` - the same functions used by `cycle.rs` and `daemon.rs`.

```rust
// src/runner/playlist_runner.rs

use std::path::PathBuf;
use std::time::Instant;

use async_trait::async_trait;
use crossterm::event::KeyCode;

use crate::api_broker::{handle_message, MessageDestination};
use crate::cli_display::{print_error, print_progress, print_success};
use crate::errors::VestaboardError;
use crate::playlist::{Playlist, PlaylistItem, PlaylistState};
use crate::runner::{ControlFlow, Runner, PLAYLIST_HELP};
use crate::runtime_state::RuntimeState;
use crate::widgets::resolver::execute_widget;
use crate::widgets::widget_utils::error_to_display_message;

pub struct PlaylistRunner {
    playlist: Playlist,
    state: PlaylistState,
    current_index: usize,
    state_path: PathBuf,
    run_once: bool,
    cycle_complete: bool,
    last_display_time: Option<Instant>,
}

impl PlaylistRunner {
    pub fn new(playlist: Playlist, state_path: PathBuf, start_index: usize, run_once: bool) -> Self {
        Self {
            playlist,
            state: PlaylistState::Stopped,
            current_index: start_index,
            state_path,
            run_once,
            cycle_complete: false,
            last_display_time: None,
        }
    }

    /// Restore from saved state if available
    pub fn restore_from_state(playlist: Playlist, state_path: &PathBuf) -> Self {
        let saved_state = RuntimeState::load(state_path);

        let start_index = if saved_state.playlist_index < playlist.items.len() {
            saved_state.playlist_index
        } else {
            0
        };

        log::info!("Restored playlist state: index={}", start_index);

        Self::new(playlist, state_path.clone(), start_index, false)
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn current_item(&self) -> Option<&PlaylistItem> {
        self.playlist.get_item_by_index(self.current_index)
    }

    pub fn state(&self) -> PlaylistState {
        self.state
    }

    pub fn pause(&mut self) {
        if self.state == PlaylistState::Running {
            self.state = PlaylistState::Paused;
            log::info!("Playlist paused at index {}", self.current_index);
            println!("Paused.");
        }
    }

    pub fn resume(&mut self) {
        if self.state == PlaylistState::Paused {
            self.state = PlaylistState::Running;
            log::info!("Playlist resumed from index {}", self.current_index);
            println!("Resumed.");
        }
    }

    pub fn skip_to_next(&mut self) {
        self.advance_index();
        // Reset last_display_time so the item displays immediately on next iteration
        self.last_display_time = None;
        log::info!("Skipped to item {}", self.current_index);
        println!("Skipping to next item...");
    }

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

    fn should_display_next(&self) -> bool {
        match self.state {
            PlaylistState::Running => {
                match self.last_display_time {
                    None => true, // First display
                    Some(last) => {
                        let elapsed = last.elapsed().as_secs();
                        elapsed >= self.playlist.interval_seconds
                    }
                }
            }
            _ => false,
        }
    }

    fn save_state(&self) {
        let state = RuntimeState {
            playlist_state: self.state,  // PlaylistState is Copy
            playlist_index: self.current_index,
            last_shown_time: Some(chrono::Utc::now()),
        };
        state.save(&self.state_path);
    }

    /// Display the current playlist item on the Vestaboard
    /// Uses execute_widget() and handle_message() - same as cycle.rs
    async fn display_current_item(&mut self) -> Result<(), VestaboardError> {
        let item = match self.current_item() {
            Some(item) => item.clone(),
            None => {
                log::warn!("No current item to display");
                return Ok(());
            }
        };

        log::info!("Displaying playlist item: {} ({})", item.id, item.widget);
        print_progress(&format!("Showing {}...", item.widget));

        // Save state BEFORE display (ensures we retry on crash)
        self.save_state();

        // Execute widget to generate message - uses existing execute_widget()
        let message = match execute_widget(&item.widget, &item.input).await {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("Widget '{}' failed: {}", item.widget, e);
                print_error(&format!("Widget {} failed: {}", item.widget, e.to_user_message()));

                // Continue with error display (matches cycle.rs behavior)
                error_to_display_message(&e)
            }
        };

        // Send to Vestaboard - uses existing handle_message()
        match handle_message(message, MessageDestination::Vestaboard).await {
            Ok(_) => {
                log::info!("Successfully displayed item {}", item.id);
                self.last_display_time = Some(Instant::now());
                print_success(&format!("Displayed: {}", item.widget));
            }
            Err(e) => {
                log::error!("Failed to send message: {}", e);
                print_error(&e.to_user_message());
                // Don't fail the whole runner - continue to next item
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Runner for PlaylistRunner {
    fn start(&mut self) {
        if self.playlist.is_empty() {
            log::warn!("Cannot start empty playlist");
            return;
        }

        self.state = PlaylistState::Running;
        self.cycle_complete = false;
        log::info!("Playlist started at index {}", self.current_index);
        print_progress(&format!(
            "Starting playlist ({} items, {} second interval)...",
            self.playlist.len(),
            self.playlist.interval_seconds
        ));
    }

    async fn run_iteration(&mut self) -> Result<(), VestaboardError> {
        // Check if we should exit (--once mode)
        if self.run_once && self.cycle_complete {
            log::info!("Completed one full cycle, stopping (--once mode)");
            println!("Completed one full cycle.");
            self.state = PlaylistState::Stopped;
            return Ok(());
        }

        // Only display if running and interval has elapsed
        if self.should_display_next() {
            self.display_current_item().await?;
            self.advance_index();
        }

        Ok(())
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
                self.skip_to_next();
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
        self.save_state();
        log::info!("Playlist runner cleanup complete");
    }
}
```

**Key Integration Points**:
1. **`execute_widget()`** - from `widgets/resolver.rs`, generates the message content
2. **`handle_message()`** - from `api_broker.rs`, sends to Vestaboard or console
3. **`error_to_display_message()`** - from `widget_utils.rs`, converts errors to displayable messages
4. **`RuntimeState.save()`** - persists state before display (crash recovery)

### File Monitor (Generalized)

Extract from `ScheduleMonitor` to be generic:

```rust
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::de::DeserializeOwned;

pub struct FileMonitor<T> {
    path: PathBuf,
    last_modified: Option<SystemTime>,
    current_data: T,
}

impl<T: DeserializeOwned + Default + Clone> FileMonitor<T> {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            last_modified: None,
            current_data: T::default(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), VestaboardError> {
        self.reload()?;
        Ok(())
    }

    pub fn reload_if_modified(&mut self) -> Result<bool, VestaboardError> {
        let current_mod_time = self.get_mod_time()?;

        match self.last_modified {
            Some(last) if current_mod_time == last => Ok(false),
            _ => {
                self.reload()?;
                Ok(true)
            }
        }
    }

    pub fn reload(&mut self) -> Result<(), VestaboardError> {
        self.last_modified = Some(self.get_mod_time()?);

        match fs::read_to_string(&self.path) {
            Ok(content) if !content.trim().is_empty() => {
                // Parse JSON, but keep existing data on parse error (resilient behavior)
                match serde_json::from_str(&content) {
                    Ok(data) => self.current_data = data,
                    Err(e) => {
                        log::warn!("Parse error in {}, keeping existing data: {}",
                            self.path.display(), e);
                        // Keep existing data - don't update self.current_data
                    }
                }
            }
            Ok(_) => {
                // Empty file
                self.current_data = T::default();
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.current_data = T::default();
            }
            Err(e) => {
                log::error!("Error reading file: {}", e);
                // Keep existing data on error
            }
        }

        Ok(())
    }

    pub fn get_current(&self) -> &T {
        &self.current_data
    }

    fn get_mod_time(&self) -> Result<SystemTime, VestaboardError> {
        match fs::metadata(&self.path) {
            Ok(meta) => Ok(meta.modified().unwrap_or(SystemTime::UNIX_EPOCH)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(SystemTime::UNIX_EPOCH)
            }
            Err(e) => {
                // Use existing io_error pattern from errors.rs
                Err(VestaboardError::io_error(
                    e,
                    &format!("accessing {}", self.path.display())
                ))
            }
        }
    }
}
```

---

## Error Handling

### New Error Variants

Add to `VestaboardError` in `src/errors.rs`. **Note**: This project does NOT use `thiserror` - it manually implements `Display`. Follow the existing pattern:

```rust
// Add these variants to the enum in src/errors.rs
pub enum VestaboardError {
    // ... existing variants ...

    LockError {
        message: String,
    },
    InputError {
        message: String,
    },
    ValidationError {
        message: String,
    },
}

// Add to the Display impl
impl std::fmt::Display for VestaboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ... existing arms ...

            VestaboardError::LockError { message } => {
                write!(f, "Lock Error: {}", message)
            },
            VestaboardError::InputError { message } => {
                write!(f, "Input Error: {}", message)
            },
            VestaboardError::ValidationError { message } => {
                write!(f, "Validation Error: {}", message)
            },
        }
    }
}

// Add to the PartialEq impl
impl PartialEq for VestaboardError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // ... existing arms ...

            (
                VestaboardError::LockError { message: m1 },
                VestaboardError::LockError { message: m2 },
            ) => m1 == m2,
            (
                VestaboardError::InputError { message: m1 },
                VestaboardError::InputError { message: m2 },
            ) => m1 == m2,
            (
                VestaboardError::ValidationError { message: m1 },
                VestaboardError::ValidationError { message: m2 },
            ) => m1 == m2,
            _ => false,
        }
    }
}

// Add convenience constructors (following existing pattern)
impl VestaboardError {
    // ... existing constructors ...

    pub fn lock_error(message: &str) -> Self {
        VestaboardError::LockError {
            message: message.to_string(),
        }
    }

    pub fn input_error(message: &str) -> Self {
        VestaboardError::InputError {
            message: message.to_string(),
        }
    }

    pub fn validation_error(message: &str) -> Self {
        VestaboardError::ValidationError {
            message: message.to_string(),
        }
    }
}

// Add to to_user_message() method in errors.rs
impl VestaboardError {
    pub fn to_user_message(&self) -> String {
        match self {
            // ... existing arms ...

            VestaboardError::LockError { message } => message.clone(),
            VestaboardError::InputError { message } => message.clone(),
            VestaboardError::ValidationError { message } => message.clone(),
        }
    }
}
```

### Update error_to_display_message()

Add handling for the new error types in `src/widgets/widget_utils.rs`:

```rust
// Add to the error_to_display_message() function in widget_utils.rs
pub fn error_to_display_message(error: &VestaboardError) -> Vec<String> {
    match error {
        // ... existing arms ...

        VestaboardError::LockError { message } => {
            format_error_with_header(
                &if message.len() > 40 { format!("{}...", &message[..37]) } else { message.clone() },
                "lock error"
            )
        },
        VestaboardError::InputError { message } => {
            format_error_with_header(
                &if message.len() > 40 { format!("{}...", &message[..37]) } else { message.clone() },
                "input error"
            )
        },
        VestaboardError::ValidationError { message } => {
            format_error_with_header(
                &if message.len() > 40 { format!("{}...", &message[..37]) } else { message.clone() },
                "invalid"
            )
        },
    }
}
```

### Widget Failures

**Standardized behavior across all execution modes** (matches existing `cycle.rs` behavior):

1. `log::error!("Widget '{}' failed: {}", widget, error)` - Always log
2. `eprintln!("Widget {} failed: {}", widget, error)` - Always print to console
3. Display error message on Vestaboard using `error_to_display_message()` - so board shows something
4. Continue to next item - Don't stop the playlist/schedule

**Rationale**: Following the existing pattern in `cycle.rs`, widget errors are displayed on the Vestaboard (using `error_to_display_message()` from `widget_utils.rs`) so the user knows something went wrong, then execution continues.

### File Errors

**Startup behavior** (fail fast):

| Error | Behavior |
|-------|----------|
| File doesn't exist | Create empty file, log info, continue with empty list |
| Permission denied | Fatal error, exit with clear message |
| Invalid JSON | Fatal error, exit - user must fix the file |
| Wrong schema | Best-effort parse with defaults, log warning |

**Runtime behavior** (be resilient):

| Error | Behavior |
|-------|----------|
| File deleted while running | Continue with in-memory version, log warning |
| File modified with bad JSON | Keep using old version, log error, don't crash |

---

## Manual Messages (`vbl show`)

### Independence

`vbl show` and `vbl schedule run` / `vbl playlist run` operate independently:

- **No communication** between them
- **No IPC** (inter-process communication)
- If both send at similar times, **last one wins** (Vestaboard shows whatever was sent last)
- This is **the user's responsibility** to manage

### Behavior

```
┌─────────────────┐         ┌─────────────────────────┐
│   vbl show      │         │ vbl schedule/playlist   │
│                 │         │         run             │
│  Sends message  │         │                         │
│  directly to    │────────►│  Sends messages at      │
│  Vestaboard     │    ▲    │  scheduled/interval     │
└─────────────────┘    │    │  times                  │
                       │    └─────────────────────────┘
                       │
                       │ Both write to same
                       │ Vestaboard independently
                       ▼
              ┌─────────────────┐
              │   Vestaboard    │
              │                 │
              │ Shows whatever  │
              │ was sent last   │
              └─────────────────┘
```

**Note**: There are no `--pause-playlist` or `--stop-playlist` flags on `vbl show`. Without IPC, these cannot work. If you need to pause the playlist and show a message:

```bash
# In the terminal running playlist, press 'p' to pause
# In another terminal:
vbl show text "important message"
# When done, press 'r' in the playlist terminal to resume
```

---

## Configuration

### Config File Updates

**File**: `data/vblconfig.toml`

```toml
# Existing settings
log_level = "info"
log_file_path = "data/vestaboard.log"
console_log_level = "info"
schedule_file_path = "data/schedule.json"
schedule_backup_path = "data/schedule_backup.json"

# New settings
playlist_file_path = "data/playlist.json"
runtime_state_path = "data/runtime_state.json"
lock_file_path = "data/vestaboard.lock"

[playlist]
default_interval_seconds = 300
minimum_interval_seconds = 60
```

### Config Struct Updates

**File**: `src/config.rs`

The following fields and methods need to be added to the `Config` struct:

```rust
// Add to Config struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    // ... existing fields ...

    pub playlist_file_path: Option<String>,
    pub runtime_state_path: Option<String>,
    pub lock_file_path: Option<String>,
    pub playlist: Option<PlaylistConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PlaylistConfig {
    #[serde(default = "default_interval")]
    pub default_interval_seconds: u64,
    #[serde(default = "minimum_interval")]
    pub minimum_interval_seconds: u64,
}

fn default_interval() -> u64 { 300 }
fn minimum_interval() -> u64 { 60 }

// Add to Config impl
impl Config {
    // ... existing methods ...

    pub fn get_playlist_file_path(&self) -> PathBuf {
        PathBuf::from(
            self.playlist_file_path
                .as_deref()
                .unwrap_or("data/playlist.json")
        )
    }

    pub fn get_runtime_state_path(&self) -> PathBuf {
        PathBuf::from(
            self.runtime_state_path
                .as_deref()
                .unwrap_or("data/runtime_state.json")
        )
    }

    pub fn get_lock_file_path(&self) -> PathBuf {
        PathBuf::from(
            self.lock_file_path
                .as_deref()
                .unwrap_or("data/vestaboard.lock")
        )
    }

    pub fn get_playlist_config(&self) -> PlaylistConfig {
        self.playlist.clone().unwrap_or_default()
    }
}
```

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

## Phase 1: Foundation (Non-Breaking)

**Goal**: Add new data structures and shared components without changing existing behavior.

**Duration estimate**: This phase establishes core building blocks.

### Phase 1 Checklist

#### 1.1 Add Error Variants (estimated: 15 min)

- [ ] **Test first**: Add test file `src/tests/playlist_error_tests.rs`
- [ ] **Write test**: `test_lock_error_displays_message`
- [ ] **Write test**: `test_validation_error_displays_message`
- [ ] **Run tests** - should fail (variants don't exist)
- [ ] **Implement**: Add `LockError`, `InputError`, `ValidationError` to `errors.rs`
- [ ] **Run tests** - should pass
- [ ] **Commit**: "Add error variants for playlist feature"

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

#### 1.2 Create PlaylistItem and Playlist structs

- [ ] **Create test file**: `src/tests/playlist_tests.rs`
- [ ] **Add to mod.rs**: `mod playlist_tests;`

**Step 1.2.1: PlaylistItem basics**

- [ ] **Write test**: `test_playlist_item_creation`
- [ ] **Run test** - fails
- [ ] **Create file**: `src/playlist.rs` with `PlaylistItem` struct
- [ ] **Run test** - passes
- [ ] **Commit**: "Add PlaylistItem struct"

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

- [ ] **Write test**: `test_playlist_creation_with_defaults`
- [ ] **Write test**: `test_playlist_default_interval_is_300`
- [ ] **Run tests** - fail
- [ ] **Implement**: `Playlist` struct with defaults
- [ ] **Run tests** - pass
- [ ] **Commit**: "Add Playlist struct with defaults"

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

- [ ] **Write test**: `test_playlist_add_item`
- [ ] **Run test** - fails
- [ ] **Implement**: `add_item()` method
- [ ] **Run test** - passes

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

- [ ] **Write test**: `test_playlist_remove_item_by_id`
- [ ] **Write test**: `test_playlist_remove_nonexistent_returns_false`
- [ ] **Run tests** - fail
- [ ] **Implement**: `remove_item()` method
- [ ] **Run tests** - pass

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

- [ ] **Write test**: `test_playlist_remove_from_empty_returns_false`
- [ ] **Write test**: `test_playlist_is_empty`
- [ ] **Run test** - fails
- [ ] **Implement**: `is_empty()` method
- [ ] **Run test** - passes

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

- [ ] **Write test**: `test_playlist_validate_interval_rejects_under_60`
- [ ] **Write test**: `test_playlist_validate_interval_accepts_60_and_above`
- [ ] **Run tests** - fail
- [ ] **Implement**: `validate_interval()` method
- [ ] **Run tests** - pass
- [ ] **Commit**: "Add Playlist CRUD and validation"

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

- [ ] **Write test**: `test_playlist_save_and_load`
- [ ] **Write test**: `test_playlist_load_nonexistent_returns_default`
- [ ] **Write test**: `test_playlist_load_invalid_json_returns_error`
- [ ] **Run tests** - fail
- [ ] **Implement**: `save()` and `load()` functions
- [ ] **Run tests** - pass
- [ ] **Commit**: "Add Playlist file persistence"

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

#### 1.3 Create RuntimeState

- [ ] **Create test file section** in `src/tests/playlist_tests.rs` or separate file
- [ ] **Write test**: `test_runtime_state_default_values`
- [ ] **Write test**: `test_runtime_state_save_and_load`
- [ ] **Write test**: `test_runtime_state_load_missing_file_returns_default`
- [ ] **Write test**: `test_runtime_state_load_corrupted_file_returns_default`
- [ ] **Run tests** - fail
- [ ] **Create file**: `src/runtime_state.rs`
- [ ] **Implement**: `RuntimeState` struct with `load()` and `save()`
- [ ] **Run tests** - pass
- [ ] **Commit**: "Add RuntimeState for playlist persistence"

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

#### 1.4 Create InstanceLock

- [ ] **Create test file**: `src/tests/lock_tests.rs`
- [ ] **Write test**: `test_lock_acquires_when_no_existing_lock`
- [ ] **Write test**: `test_lock_fails_when_lock_exists_and_pid_running`
- [ ] **Write test**: `test_lock_succeeds_when_lock_stale`
- [ ] **Write test**: `test_lock_released_on_drop`
- [ ] **Run tests** - fail
- [ ] **Create file**: `src/runner/lock.rs`
- [ ] **Implement**: `InstanceLock` struct
- [ ] **Run tests** - pass
- [ ] **Commit**: "Add InstanceLock for single-instance enforcement"

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

#### 1.6 Create FileMonitor (Generic)

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

#### 1.7 Create Runner module structure

- [ ] **Create directory**: `src/runner/`
- [ ] **Create file**: `src/runner/mod.rs`
- [ ] **Write test**: `test_control_flow_variants`
- [ ] **Implement**: `ControlFlow` enum, `Runner` trait
- [ ] **Run tests** - pass
- [ ] **Commit**: "Add Runner trait and module structure"

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

#### 1.8 Create MockInput for testing

- [ ] **Write test**: `test_mock_input_provides_keys_in_order`
- [ ] **Write test**: `test_mock_input_returns_none_when_exhausted`
- [ ] **Implement**: `InputSource` trait and `MockInput` in `src/runner/keyboard.rs`
- [ ] **Run tests** - pass
- [ ] **Commit**: "Add InputSource trait and MockInput for testing"

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

#### 1.9 Update Config

- [ ] **Write test**: `test_config_loads_playlist_path`
- [ ] **Write test**: `test_config_defaults_for_new_fields`
- [ ] **Add fields**: `playlist_file_path`, `runtime_state_path`, `lock_file_path`
- [ ] **Run tests** - pass
- [ ] **Commit**: "Add playlist config fields"

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

### Phase 1 Definition of Done

**All of the following must be true:**

- [ ] `cargo test playlist` - all tests pass
- [ ] `cargo test runtime_state` - all tests pass
- [ ] `cargo test lock` - all tests pass
- [ ] `cargo test file_monitor` - all tests pass
- [ ] `cargo test config` - all tests pass (including new tests)
- [ ] `cargo build` - compiles without errors
- [ ] `cargo clippy` - no warnings
- [ ] Existing commands (`vbl show`, `vbl schedule`, `vbl daemon`, `vbl cycle`) work unchanged
- [ ] All new modules are documented with `///` doc comments
- [ ] Code coverage for new modules > 80%

**Test count checkpoint**: Phase 1 should add approximately 40-50 new tests.

---

## Phase 2: Playlist CLI

**Goal**: Add playlist management commands.

### Phase 2 Checklist

#### 2.1 Add Playlist CLI structure

- [ ] **Write test**: `test_cli_parses_playlist_add`
- [ ] **Write test**: `test_cli_parses_playlist_list`
- [ ] **Write test**: `test_cli_parses_playlist_remove`
- [ ] **Write test**: `test_cli_parses_playlist_clear`
- [ ] **Write test**: `test_cli_parses_playlist_interval`
- [ ] **Write test**: `test_cli_parses_playlist_preview`
- [ ] **Run tests** - fail
- [ ] **Implement**: Add `Playlist` command variants to `cli_setup.rs`
- [ ] **Run tests** - pass
- [ ] **Commit**: "Add playlist CLI command parsing"

```rust
// Add to src/tests/cli_setup_tests.rs
use crate::cli_setup::{Cli, Command, PlaylistArgs};
use clap::Parser;

#[test]
fn test_cli_parses_playlist_add_weather() {
    let cli = Cli::parse_from(["vbl", "playlist", "add", "weather"]);
    match cli.command {
        Command::Playlist { action: PlaylistArgs::Add { widget, input } } => {
            assert_eq!(widget, "weather");
            assert!(input.is_empty());
        }
        _ => panic!("Expected Playlist Add command"),
    }
}

#[test]
fn test_cli_parses_playlist_add_text_with_input() {
    let cli = Cli::parse_from(["vbl", "playlist", "add", "text", "hello", "world"]);
    match cli.command {
        Command::Playlist { action: PlaylistArgs::Add { widget, input } } => {
            assert_eq!(widget, "text");
            assert_eq!(input, vec!["hello", "world"]);
        }
        _ => panic!("Expected Playlist Add command"),
    }
}

#[test]
fn test_cli_parses_playlist_list() {
    let cli = Cli::parse_from(["vbl", "playlist", "list"]);
    match cli.command {
        Command::Playlist { action: PlaylistArgs::List } => {}
        _ => panic!("Expected Playlist List command"),
    }
}

#[test]
fn test_cli_parses_playlist_remove() {
    let cli = Cli::parse_from(["vbl", "playlist", "remove", "abc1"]);
    match cli.command {
        Command::Playlist { action: PlaylistArgs::Remove { id } } => {
            assert_eq!(id, "abc1");
        }
        _ => panic!("Expected Playlist Remove command"),
    }
}

#[test]
fn test_cli_parses_playlist_clear() {
    let cli = Cli::parse_from(["vbl", "playlist", "clear"]);
    match cli.command {
        Command::Playlist { action: PlaylistArgs::Clear } => {}
        _ => panic!("Expected Playlist Clear command"),
    }
}

#[test]
fn test_cli_parses_playlist_interval() {
    let cli = Cli::parse_from(["vbl", "playlist", "interval", "120"]);
    match cli.command {
        Command::Playlist { action: PlaylistArgs::Interval { seconds } } => {
            assert_eq!(seconds, 120);
        }
        _ => panic!("Expected Playlist Interval command"),
    }
}

#[test]
fn test_cli_parses_playlist_preview() {
    let cli = Cli::parse_from(["vbl", "playlist", "preview"]);
    match cli.command {
        Command::Playlist { action: PlaylistArgs::Preview } => {}
        _ => panic!("Expected Playlist Preview command"),
    }
}
```

#### 2.2 Wire up playlist commands to main.rs

- [ ] **Write integration test**: `test_playlist_add_creates_item`
- [ ] **Write integration test**: `test_playlist_list_shows_items`
- [ ] **Write integration test**: `test_playlist_remove_deletes_item`
- [ ] **Implement**: Handle `Command::Playlist` in `main.rs`
- [ ] **Run tests** - pass
- [ ] **Manual test**: Run `vbl playlist add weather`, `vbl playlist list`
- [ ] **Commit**: "Wire up playlist CLI commands"

```rust
// src/tests/playlist_integration_tests.rs
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_playlist_add_and_list_integration() {
    let temp_dir = tempdir().unwrap();
    let playlist_path = temp_dir.path().join("playlist.json");

    // This would need environment setup to point to temp playlist
    // For now, these are more like acceptance test descriptions

    // vbl playlist add weather
    // vbl playlist list
    // Verify output contains "weather"
}
```

### Phase 2 Definition of Done

- [ ] `cargo test cli_setup` - all tests pass (including new playlist tests)
- [ ] `vbl playlist add weather` - adds item, shows confirmation
- [ ] `vbl playlist add text "hello world"` - adds text item
- [ ] `vbl playlist list` - shows all items with IDs
- [ ] `vbl playlist remove <id>` - removes item
- [ ] `vbl playlist clear` - removes all items
- [ ] `vbl playlist interval 120` - sets interval
- [ ] `vbl playlist preview` - shows all items (dry-run)
- [ ] `vbl playlist --help` - shows help for all subcommands
- [ ] Error handling: invalid widget type shows helpful message

**Test count checkpoint**: Phase 2 should add approximately 15-20 new tests.

---

## Phase 3: Playlist Execution

**Goal**: Implement `vbl playlist run` with interactive controls.

### Phase 3 Checklist

#### 3.1 Create PlaylistRunner struct

- [ ] **Write test**: `test_playlist_runner_creation`
- [ ] **Write test**: `test_playlist_runner_advances_index`
- [ ] **Write test**: `test_playlist_runner_wraps_at_end`
- [ ] **Run tests** - fail
- [ ] **Create file**: `src/runner/playlist_runner.rs`
- [ ] **Implement**: Basic `PlaylistRunner` struct
- [ ] **Run tests** - pass

```rust
// src/tests/playlist_runner_tests.rs
use crate::runner::playlist_runner::PlaylistRunner;
use crate::playlist::{Playlist, PlaylistItem};
use crate::runner::keyboard::MockInput;
use serde_json::json;

fn create_test_playlist() -> Playlist {
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
    playlist
}

#[test]
fn test_playlist_runner_starts_at_index_zero() {
    let playlist = create_test_playlist();
    let runner = PlaylistRunner::new(playlist);
    assert_eq!(runner.current_index(), 0);
}

#[test]
fn test_playlist_runner_advances_index() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);

    runner.advance();
    assert_eq!(runner.current_index(), 1);

    runner.advance();
    assert_eq!(runner.current_index(), 2);
}

#[test]
fn test_playlist_runner_wraps_at_end() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);

    runner.advance(); // 0 -> 1
    runner.advance(); // 1 -> 2
    runner.advance(); // 2 -> 0 (wrap)

    assert_eq!(runner.current_index(), 0);
}

#[test]
fn test_playlist_runner_current_item() {
    let playlist = create_test_playlist();
    let runner = PlaylistRunner::new(playlist);

    let item = runner.current_item().unwrap();
    assert_eq!(item.widget, "weather");
}
```

#### 3.2 Implement state machine

- [ ] **Write test**: `test_playlist_runner_pause_sets_state`
- [ ] **Write test**: `test_playlist_runner_resume_from_paused`
- [ ] **Write test**: `test_playlist_runner_pause_when_already_paused_is_noop`
- [ ] **Implement**: `pause()`, `resume()` methods
- [ ] **Run tests** - pass

```rust
#[test]
fn test_playlist_runner_initial_state_is_stopped() {
    let playlist = create_test_playlist();
    let runner = PlaylistRunner::new(playlist);
    assert_eq!(runner.state(), PlaylistState::Stopped);
}

#[test]
fn test_playlist_runner_start_sets_running() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);
    runner.start();
    assert_eq!(runner.state(), PlaylistState::Running);
}

#[test]
fn test_playlist_runner_pause_sets_paused() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);
    runner.start();
    runner.pause();
    assert_eq!(runner.state(), PlaylistState::Paused);
}

#[test]
fn test_playlist_runner_resume_sets_running() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);
    runner.start();
    runner.pause();
    runner.resume();
    assert_eq!(runner.state(), PlaylistState::Running);
}

#[test]
fn test_playlist_runner_pause_when_stopped_is_noop() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);
    runner.pause(); // Should not crash
    assert_eq!(runner.state(), PlaylistState::Stopped);
}

#[test]
fn test_playlist_runner_next_advances_and_stays_paused() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);
    runner.start();
    runner.pause();

    let initial_index = runner.current_index();
    runner.skip_to_next();

    assert_eq!(runner.current_index(), initial_index + 1);
    assert_eq!(runner.state(), PlaylistState::Paused);
}
```

#### 3.3 Implement keyboard handling

- [ ] **Write test**: `test_playlist_runner_handles_p_key`
- [ ] **Write test**: `test_playlist_runner_handles_r_key`
- [ ] **Write test**: `test_playlist_runner_handles_n_key`
- [ ] **Write test**: `test_playlist_runner_handles_q_key`
- [ ] **Write test**: `test_playlist_runner_handles_question_mark`
- [ ] **Implement**: `handle_key()` method using `Runner` trait
- [ ] **Run tests** - pass

```rust
use crate::runner::{Runner, ControlFlow};
use crossterm::event::KeyCode;

#[test]
fn test_playlist_runner_p_key_pauses() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);
    runner.start();

    let result = runner.handle_key(KeyCode::Char('p'));

    assert_eq!(result, ControlFlow::Continue);
    assert_eq!(runner.state(), PlaylistState::Paused);
}

#[test]
fn test_playlist_runner_r_key_resumes() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);
    runner.start();
    runner.pause();

    let result = runner.handle_key(KeyCode::Char('r'));

    assert_eq!(result, ControlFlow::Continue);
    assert_eq!(runner.state(), PlaylistState::Running);
}

#[test]
fn test_playlist_runner_n_key_advances() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);
    runner.start();

    let initial_index = runner.current_index();
    let result = runner.handle_key(KeyCode::Char('n'));

    assert_eq!(result, ControlFlow::Continue);
    assert_eq!(runner.current_index(), initial_index + 1);
}

#[test]
fn test_playlist_runner_q_key_exits() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);
    runner.start();

    let result = runner.handle_key(KeyCode::Char('q'));

    assert_eq!(result, ControlFlow::Exit);
}

#[test]
fn test_playlist_runner_question_mark_returns_continue() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);

    let result = runner.handle_key(KeyCode::Char('?'));

    assert_eq!(result, ControlFlow::Continue);
    // Help text should be printed (we'd verify stdout in integration test)
}

#[test]
fn test_playlist_runner_unknown_key_ignored() {
    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::new(playlist);
    runner.start();

    let initial_state = runner.state();
    let initial_index = runner.current_index();

    let result = runner.handle_key(KeyCode::Char('x'));

    assert_eq!(result, ControlFlow::Continue);
    assert_eq!(runner.state(), initial_state);
    assert_eq!(runner.current_index(), initial_index);
}
```

#### 3.4 Add --once, --index, --id flags

- [ ] **Write test**: `test_playlist_run_once_exits_after_cycle`
- [ ] **Write test**: `test_playlist_run_index_starts_at_position`
- [ ] **Write test**: `test_playlist_run_id_starts_at_item`
- [ ] **Write test**: `test_playlist_run_id_not_found_errors`
- [ ] **Update CLI**: Add flags to `playlist run` command
- [ ] **Implement**: Flag handling in `PlaylistRunner`
- [ ] **Run tests** - pass

```rust
#[test]
fn test_playlist_runner_with_start_index() {
    let playlist = create_test_playlist();
    let runner = PlaylistRunner::with_start_index(playlist, 2);
    assert_eq!(runner.current_index(), 2);
}

#[test]
fn test_playlist_runner_with_start_id() {
    let playlist = create_test_playlist();
    let runner = PlaylistRunner::with_start_id(playlist, "b").unwrap();
    assert_eq!(runner.current_index(), 1);
}

#[test]
fn test_playlist_runner_with_start_id_not_found() {
    let playlist = create_test_playlist();
    let result = PlaylistRunner::with_start_id(playlist, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_playlist_runner_once_mode_stops_after_full_cycle() {
    let playlist = create_test_playlist(); // 3 items
    let mut runner = PlaylistRunner::new_once(playlist);
    runner.start();

    // Advance through all items
    runner.advance(); // 0 -> 1
    assert!(!runner.is_complete());

    runner.advance(); // 1 -> 2
    assert!(!runner.is_complete());

    runner.advance(); // 2 -> done
    assert!(runner.is_complete());
}
```

#### 3.5 Add state persistence

- [ ] **Write test**: `test_playlist_runner_saves_state_on_advance`
- [ ] **Write test**: `test_playlist_runner_restores_state_on_creation`
- [ ] **Implement**: Integration with `RuntimeState`
- [ ] **Run tests** - pass

```rust
#[test]
fn test_playlist_runner_saves_state() {
    let temp_dir = tempdir().unwrap();
    let state_path = temp_dir.path().join("state.json");

    let playlist = create_test_playlist();
    let mut runner = PlaylistRunner::with_state_path(playlist, &state_path);
    runner.start();
    runner.advance();
    runner.pause();

    // State should be saved
    let state = RuntimeState::load(&state_path);  // load() is infallible
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
    state.save(&state_path);  // save() is infallible

    let playlist = create_test_playlist();
    let runner = PlaylistRunner::restore_from_state(playlist, &state_path);

    assert_eq!(runner.current_index(), 2);
}
```

#### 3.6 Wire up to CLI and test end-to-end

- [ ] **Write test**: `test_cli_parses_playlist_run`
- [ ] **Write test**: `test_cli_parses_playlist_run_once`
- [ ] **Write test**: `test_cli_parses_playlist_run_index`
- [ ] **Write test**: `test_cli_parses_playlist_run_id`
- [ ] **Run tests** - fail
- [ ] **Implement**: Add `Run` variant to `PlaylistArgs` in `cli_setup.rs`
- [ ] **Run tests** - pass

```rust
// Add to src/tests/cli_setup_tests.rs

#[test]
fn test_cli_parses_playlist_run() {
    let cli = Cli::parse_from(["vbl", "playlist", "run"]);
    match cli.command {
        Command::Playlist { action: PlaylistArgs::Run { once, index, id } } => {
            assert!(!once);
            assert!(index.is_none());
            assert!(id.is_none());
        }
        _ => panic!("Expected Playlist Run command"),
    }
}

#[test]
fn test_cli_parses_playlist_run_once() {
    let cli = Cli::parse_from(["vbl", "playlist", "run", "--once"]);
    match cli.command {
        Command::Playlist { action: PlaylistArgs::Run { once, .. } } => {
            assert!(once);
        }
        _ => panic!("Expected Playlist Run command with --once"),
    }
}

#[test]
fn test_cli_parses_playlist_run_index() {
    let cli = Cli::parse_from(["vbl", "playlist", "run", "--index", "3"]);
    match cli.command {
        Command::Playlist { action: PlaylistArgs::Run { index, .. } } => {
            assert_eq!(index, Some(3));
        }
        _ => panic!("Expected Playlist Run command with --index"),
    }
}

#[test]
fn test_cli_parses_playlist_run_id() {
    let cli = Cli::parse_from(["vbl", "playlist", "run", "--id", "abc1"]);
    match cli.command {
        Command::Playlist { action: PlaylistArgs::Run { id, .. } } => {
            assert_eq!(id, Some("abc1".to_string()));
        }
        _ => panic!("Expected Playlist Run command with --id"),
    }
}

#[test]
fn test_cli_playlist_run_index_and_id_mutually_exclusive() {
    // Should fail to parse when both are provided
    let result = Cli::try_parse_from(["vbl", "playlist", "run", "--index", "3", "--id", "abc1"]);
    assert!(result.is_err());
}
```

- [ ] **Implement**: Handle `Command::Playlist { action: PlaylistArgs::Run { .. } }` in `main.rs`

```rust
// Add to main.rs in the match on Command

Command::Playlist { action } => match action {
    // ... existing arms ...

    PlaylistArgs::Run { once, index, id } => {
        // Load config and playlist
        let config = Config::load()?;
        let playlist_path = config.get_playlist_file_path();
        let playlist = Playlist::load(&playlist_path)?;

        if playlist.is_empty() {
            println!("No items in playlist. Add items with 'vbl playlist add <widget>'.");
            return Ok(());
        }

        // Determine start position
        let start_index = if let Some(id) = id {
            playlist.find_index_by_id(&id).ok_or_else(|| {
                VestaboardError::validation_error(&format!("Item '{}' not found in playlist", id))
            })?
        } else if let Some(idx) = index {
            if idx >= playlist.len() {
                return Err(VestaboardError::validation_error(&format!(
                    "Index {} is out of bounds (playlist has {} items)",
                    idx,
                    playlist.len()
                )));
            }
            idx
        } else {
            0
        };

        // Create and run the playlist runner
        let state_path = config.get_runtime_state_path();
        let runner = PlaylistRunner::new(
            playlist,
            state_path,
            start_index,
            once,  // run_once mode
        );

        run_with_keyboard(runner, "playlist").await?;
        Ok(())
    }
},
```

- [ ] **Write integration test**: End-to-end playlist run (uses mock Vestaboard)
- [ ] **Manual test**: Full interactive session
- [ ] **Commit**: "Implement playlist run with interactive controls"

```rust
// src/tests/playlist_integration_tests.rs

/// Integration test that runs a playlist with mock components
#[tokio::test]
async fn test_playlist_run_end_to_end_with_mocks() {
    use crate::runner::keyboard::MockInput;
    use crossterm::event::KeyCode;

    let temp_dir = tempdir().unwrap();

    // Setup playlist file
    let playlist_path = temp_dir.path().join("playlist.json");
    let playlist = Playlist {
        interval_seconds: 1, // Fast for testing (below minimum, but allowed in test)
        items: vec![
            PlaylistItem::new("text".to_string(), json!("item 1")),
            PlaylistItem::new("text".to_string(), json!("item 2")),
        ],
    };
    playlist.save(&playlist_path).unwrap();

    // Setup mock input that will press 'q' after a short delay
    let mock_input = MockInput::new(vec![
        KeyCode::Char('n'),  // Skip to next
        KeyCode::Char('p'),  // Pause
        KeyCode::Char('r'),  // Resume
        KeyCode::Char('q'),  // Quit
    ]);

    // Create runner with test configuration
    let state_path = temp_dir.path().join("state.json");
    let mut runner = PlaylistRunner::new_with_input(
        playlist,
        state_path.clone(),
        0,      // start_index
        false,  // run_once
        Box::new(mock_input),
    );

    // Run should complete without error
    runner.start();

    // Simulate a few iterations
    for _ in 0..4 {
        let flow = runner.handle_next_key();
        if flow == ControlFlow::Exit {
            break;
        }
    }

    // Verify state was saved
    let state = RuntimeState::load(&state_path);
    assert!(state.playlist_index <= 2);  // Should have advanced
}
```

### Phase 3 Definition of Done

- [ ] `cargo test playlist_runner` - all tests pass
- [ ] `vbl playlist run` - starts playlist rotation
- [ ] Press `p` - pauses rotation
- [ ] Press `r` - resumes rotation
- [ ] Press `n` - skips to next item
- [ ] Press `q` - exits cleanly
- [ ] Press `?` - shows help
- [ ] `vbl playlist run --once` - exits after one cycle
- [ ] `vbl playlist run --index 2` - starts at index 2
- [ ] `vbl playlist run --id abc1` - starts at item abc1
- [ ] Ctrl+C - exits cleanly (same as `q`)
- [ ] State persists across restarts
- [ ] Cannot run two instances simultaneously (lock file works)

**Test count checkpoint**: Phase 3 should add approximately 30-40 new tests.

---

## Phase 4: Schedule Refactoring

**Goal**: Replace `vbl daemon` with `vbl schedule run`.

### Phase 4 Checklist

#### 4.1 Create ScheduleRunner

- [ ] **Write test**: `test_schedule_runner_skips_past_tasks`
- [ ] **Write test**: `test_schedule_runner_waits_for_next_task`
- [ ] **Write test**: `test_schedule_runner_q_key_exits`
- [ ] **Run tests** - fail
- [ ] **Create file**: `src/runner/schedule_runner.rs`
- [ ] **Implement**: `ScheduleRunner` struct
- [ ] **Run tests** - pass

```rust
// src/tests/schedule_runner_tests.rs
use crate::runner::schedule_runner::ScheduleRunner;
use crate::scheduler::{Schedule, ScheduledTask};
use chrono::{Utc, Duration};
use serde_json::json;

fn create_test_schedule() -> Schedule {
    let now = Utc::now();
    Schedule {
        tasks: vec![
            ScheduledTask {
                id: "past".to_string(),
                time: now - Duration::hours(2),
                widget: "weather".to_string(),
                input: json!(null),
            },
            ScheduledTask {
                id: "future".to_string(),
                time: now + Duration::hours(1),
                widget: "text".to_string(),
                input: json!("hello"),
            },
        ],
    }
}

#[test]
fn test_schedule_runner_identifies_next_task() {
    let schedule = create_test_schedule();
    let runner = ScheduleRunner::new(schedule);

    let next = runner.next_pending_task();
    assert!(next.is_some());
    assert_eq!(next.unwrap().id, "future");
}

#[test]
fn test_schedule_runner_skips_past_tasks() {
    let now = Utc::now();
    let schedule = Schedule {
        tasks: vec![
            ScheduledTask {
                id: "past1".to_string(),
                time: now - Duration::hours(2),
                widget: "weather".to_string(),
                input: json!(null),
            },
            ScheduledTask {
                id: "past2".to_string(),
                time: now - Duration::hours(1),
                widget: "text".to_string(),
                input: json!("hello"),
            },
        ],
    };

    let runner = ScheduleRunner::new(schedule);
    let next = runner.next_pending_task();

    assert!(next.is_none()); // All tasks in past
}

#[test]
fn test_schedule_runner_q_key_exits() {
    let schedule = create_test_schedule();
    let mut runner = ScheduleRunner::new(schedule);

    let result = runner.handle_key(KeyCode::Char('q'));
    assert_eq!(result, ControlFlow::Exit);
}

#[test]
fn test_schedule_runner_help_text() {
    let schedule = create_test_schedule();
    let runner = ScheduleRunner::new(schedule);

    let help = runner.help_text();
    assert!(help.contains("q"));
    assert!(help.contains("quit"));
}
```

#### 4.2 Add schedule run CLI command

- [ ] **Write test**: `test_cli_parses_schedule_run`
- [ ] **Implement**: Add `schedule run` to CLI
- [ ] **Wire up** in `main.rs`
- [ ] **Run tests** - pass

```rust
#[test]
fn test_cli_parses_schedule_run() {
    let cli = Cli::parse_from(["vbl", "schedule", "run"]);
    match cli.command {
        Command::Schedule { action: ScheduleArgs::Run } => {}
        _ => panic!("Expected Schedule Run command"),
    }
}
```

#### 4.3 Add deprecation warnings

- [ ] **Write test**: `test_daemon_command_shows_deprecation`
- [ ] **Implement**: Add warning to `vbl daemon` output
- [ ] **Run tests** - pass
- [ ] **Commit**: "Add schedule run command with daemon deprecation"

```rust
#[test]
fn test_daemon_command_output_contains_deprecation() {
    // Integration test - run vbl daemon and capture stderr
    // Verify it contains "deprecated" and "vbl schedule run"
}
```

### Phase 4 Definition of Done

- [ ] `cargo test schedule_runner` - all tests pass
- [ ] `vbl schedule run` - works like daemon but with keyboard controls
- [ ] `vbl daemon` - still works but shows deprecation warning
- [ ] Press `q` - exits cleanly
- [ ] Press `?` - shows help
- [ ] Past tasks are skipped on startup
- [ ] Schedule changes are hot-reloaded

**Test count checkpoint**: Phase 4 should add approximately 15-20 new tests.

---

## Phase 5: Cleanup (Breaking)

**Goal**: Remove deprecated code.

### Phase 5 Checklist

- [ ] **Remove**: `vbl daemon` command from CLI
- [ ] **Remove**: `vbl cycle` and `vbl cycle repeat` commands
- [ ] **Delete**: `src/daemon.rs`
- [ ] **Delete**: `src/cycle.rs`
- [ ] **Update**: README and documentation
- [ ] **Run**: Full test suite
- [ ] **Manual test**: All remaining commands
- [ ] **Commit**: "Remove deprecated daemon and cycle commands"

### Phase 5 Definition of Done

- [ ] `vbl daemon` - command not found
- [ ] `vbl cycle` - command not found
- [ ] No references to `daemon.rs` or `cycle.rs` in codebase
- [ ] README updated with new commands
- [ ] `cargo test` - all tests pass
- [ ] No orphaned code (run `cargo clippy` and check for dead code warnings)

---

## Test Summary

| Phase | New Test Files | Approximate Test Count |
|-------|---------------|------------------------|
| Phase 1 | `playlist_tests.rs`, `runtime_state_tests.rs`, `lock_tests.rs`, `file_monitor_tests.rs`, `keyboard_tests.rs` | 35-45 |
| Phase 2 | Additions to `cli_setup_tests.rs`, `playlist_integration_tests.rs` | 15-20 |
| Phase 3 | `playlist_runner_tests.rs` | 30-40 |
| Phase 4 | `schedule_runner_tests.rs` | 15-20 |
| Phase 5 | (removals) | 0 |
| **Total** | | **100-130 new tests** |

**Running tests by module:**

```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test playlist
cargo test runtime_state
cargo test lock
cargo test file_monitor
cargo test runner

# Run with output
cargo test -- --nocapture

# Run single test
cargo test test_playlist_add_item
```

---

## Current Code Issues to Address

During implementation, the following issues from the current codebase should be resolved:

| Issue | Location | Resolution |
|-------|----------|------------|
| Compile error | `daemon.rs:79` | Fix undefined `CHECK_INTERVAL_SECONDS` reference |
| Inconsistent sleep | `daemon.rs:143` | Use `tokio::time::sleep` not `thread::sleep` |
| Ignored tests | `cycle_tests.rs` | Use `InputSource` trait for mocking |
| Widget error handling | Various | Standardize using pattern in Error Handling section |
| ScheduleMonitor specific | `scheduler.rs` | Replace with generic `FileMonitor<Schedule>` |

---

## Dependencies to Add

```toml
# Cargo.toml additions
[dependencies]
crossterm = "0.27"      # Terminal input handling
atty = "0.2"            # TTY detection
async-trait = "0.1"     # Async trait support

[target.'cfg(unix)'.dependencies]
libc = "0.2"            # flock() for atomic file locking, kill() for PID checking

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = [
    "processthreadsapi",  # OpenProcess for PID checking
    "handleapi",          # CloseHandle
    "winnt",              # PROCESS_QUERY_LIMITED_INFORMATION
    "fileapi",            # LockFileEx for atomic file locking
    "minwinbase",         # OVERLAPPED, LOCKFILE_* constants
] }
```

**Note**: Most dependencies are already in the project (crossterm may already be used for clearing terminal, check Cargo.toml). Only add what's missing.

---

## Out of Scope (Future Versions)

The following features are explicitly deferred:

| Feature | Rationale |
|---------|-----------|
| Random/shuffle playlist mode | Adds complexity; sequential is sufficient for v1 |
| Weighted playlist items | Requires random mode |
| Active time windows | Can be added later if needed |
| Web UI | Separate design document needed |
| Daemon/service mode with socket IPC | Foreground with keyboard controls is simpler for v1; daemon mode can be added if headless operation becomes a priority |
| Simultaneous schedule + playlist | Mutual exclusivity is simpler |
| Cross-terminal process control | Process commands (pause/resume/next) work via keyboard only; no IPC needed for v1 |

---

## Testing Strategy

### Unit Tests

| Module | Test Focus |
|--------|------------|
| `playlist.rs` | CRUD operations, file parsing, validation, interval limits |
| `runtime_state.rs` | State persistence, corruption recovery, default handling |
| `runner/lock.rs` | Lock acquisition, stale lock detection, cleanup |
| `runner/keyboard.rs` | Key parsing (use MockInput) |
| `file_monitor.rs` | Change detection, reload behavior, error handling |

### Integration Tests

| Scenario | Validation |
|----------|------------|
| Playlist rotation | Items cycle correctly at specified interval |
| Pause/resume | State preserved correctly |
| Skip (next) | Advances to next item immediately |
| Quit | Clean exit, state saved, lock released |
| Start from index/id | Begins at correct position |
| File changes | Hot-reload of playlist works |
| Lock contention | Second instance fails with clear error |
| Stale lock | Old lock from dead process is overwritten |

### Additional Test Coverage (Edge Cases)

These tests ensure robustness for edge cases not covered by the basic tests:

```rust
// File: src/tests/edge_case_tests.rs

// === Concurrent Access Tests ===

#[tokio::test]
async fn test_lock_concurrent_acquisition_only_one_succeeds() {
    use std::sync::Arc;
    use tokio::sync::Barrier;

    let temp_dir = tempdir().unwrap();
    let lock_path = temp_dir.path().join("test.lock");
    let barrier = Arc::new(Barrier::new(2));

    let path1 = lock_path.clone();
    let barrier1 = barrier.clone();
    let handle1 = tokio::spawn(async move {
        barrier1.wait().await;
        InstanceLock::acquire_at("playlist", &path1)
    });

    let path2 = lock_path.clone();
    let barrier2 = barrier.clone();
    let handle2 = tokio::spawn(async move {
        barrier2.wait().await;
        InstanceLock::acquire_at("schedule", &path2)
    });

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    // Exactly one should succeed
    let successes = [result1.is_ok(), result2.is_ok()];
    assert_eq!(successes.iter().filter(|&&x| x).count(), 1);
}

// === File Permission Tests ===

#[test]
#[cfg(unix)]
fn test_lock_fails_on_readonly_directory() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir().unwrap();
    let readonly_dir = temp_dir.path().join("readonly");
    fs::create_dir(&readonly_dir).unwrap();
    fs::set_permissions(&readonly_dir, fs::Permissions::from_mode(0o444)).unwrap();

    let lock_path = readonly_dir.join("test.lock");
    let result = InstanceLock::acquire_at("playlist", &lock_path);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot"));

    // Cleanup: restore permissions
    fs::set_permissions(&readonly_dir, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn test_playlist_save_fails_gracefully_on_permission_error() {
    // Playlist operations should not crash on save failures
    let playlist = Playlist::default();
    // Try to save to an invalid path - should not panic
    let result = playlist.save(&PathBuf::from("/nonexistent/deeply/nested/playlist.json"));
    // Should return error, not panic
    assert!(result.is_err());
}

// === Playlist Modification During Run ===

#[test]
fn test_playlist_index_adjusts_when_item_removed_before_current() {
    // If we're at index 3 and item 1 is removed, we should adjust
    let mut playlist = Playlist::default();
    for i in 0..5 {
        playlist.add_item(PlaylistItem::new(
            "text".to_string(),
            json!(format!("item {}", i)),
        ));
    }

    let current_index = 3;
    let removed_id = playlist.items[1].id.clone();

    // Remove item before current index
    playlist.remove_item(&removed_id);

    // After removal, the effective current item is now at index 2
    let adjusted_index = if current_index > 1 { current_index - 1 } else { current_index };
    assert_eq!(adjusted_index, 2);
}

// === Empty State Tests ===

#[test]
fn test_runtime_state_load_handles_empty_json_object() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), "{}").unwrap();

    let state = RuntimeState::load(temp_file.path());
    assert_eq!(state.playlist_state, PlaylistState::Stopped);
    assert_eq!(state.playlist_index, 0);
}

#[test]
fn test_runtime_state_load_handles_partial_json() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), r#"{"playlist_index": 5}"#).unwrap();

    let state = RuntimeState::load(temp_file.path());
    assert_eq!(state.playlist_index, 5);
    assert_eq!(state.playlist_state, PlaylistState::Stopped); // default
}
```

### Manual Testing Checklist

- [ ] Add items to playlist, verify they appear in list
- [ ] Run playlist, observe rotation at correct interval
- [ ] Press `p` to pause, verify rotation stops
- [ ] Press `r` to resume, verify rotation continues from same position
- [ ] Press `n` to skip, verify next item shows immediately
- [ ] Press `?` to see help
- [ ] Press `q` to quit, verify clean exit
- [ ] Press Ctrl+C, verify same as `q`
- [ ] Restart playlist, verify it resumes from saved state
- [ ] Run with `--once`, verify exits after one complete cycle
- [ ] Run with `--index 2`, verify starts at index 2
- [ ] Run with `--id abc1`, verify starts at that item
- [ ] Try running second instance, verify error message
- [ ] Kill process with `kill -9`, restart, verify stale lock handled
- [ ] Edit playlist.json while running, verify hot reload
- [ ] Corrupt playlist.json while running, verify continues with old data
- [ ] Run with empty playlist, verify helpful error message

---

## Implementation Progress Tracker

This section tracks implementation progress. Update this section as work is completed to enable seamless handoff if implementation is interrupted.

### Current Status

| Phase | Status | Last Updated | Notes |
|-------|--------|--------------|-------|
| Phase 1: Foundation | Not Started | - | - |
| Phase 2: Playlist CLI | Not Started | - | - |
| Phase 3: Playlist Execution | Not Started | - | - |
| Phase 4: Schedule Refactoring | Not Started | - | - |
| Phase 5: Cleanup | Not Started | - | - |

### Detailed Progress

#### Phase 1 Progress
- [ ] 1.1 Error variants added to `src/errors.rs`
- [ ] 1.2 PlaylistItem struct created in `src/playlist.rs`
- [ ] 1.3 Playlist struct with CRUD operations
- [ ] 1.4 RuntimeState created in `src/runtime_state.rs`
- [ ] 1.5 InstanceLock created in `src/runner/lock.rs`
- [ ] 1.6 FileMonitor generalized in `src/file_monitor.rs`
- [ ] 1.7 Runner trait defined in `src/runner/mod.rs`
- [ ] 1.8 MockInput for testing in `src/runner/keyboard.rs`
- [ ] 1.9 Config updated with new fields

#### Phase 2 Progress
- [ ] 2.1 CLI structure added to `src/cli_setup.rs`
- [ ] 2.2 Commands wired up in `src/main.rs`

#### Phase 3 Progress
- [ ] 3.1 PlaylistRunner struct created
- [ ] 3.2 State machine implemented
- [ ] 3.3 Keyboard handling implemented
- [ ] 3.4 --once, --index, --id flags added
- [ ] 3.5 State persistence integrated
- [ ] 3.6 CLI wired up and tested

#### Phase 4 Progress
- [ ] 4.1 ScheduleRunner created
- [ ] 4.2 CLI command added
- [ ] 4.3 Deprecation warnings added

#### Phase 5 Progress
- [ ] 5.1 Deprecated commands removed
- [ ] 5.2 Old files deleted
- [ ] 5.3 Documentation updated

### Last Checkpoint

**Date**: Not started
**Completed**: None
**In Progress**: None
**Blockers**: None
**Next Steps**: Begin Phase 1.1 - Add error variants

### How to Resume Implementation

1. Check the "Detailed Progress" section above to see what's been completed
2. Find the next unchecked item - that's where to start
3. Follow the TDD approach for that item (find the test in the document, write it, make it pass)
4. Mark the checkbox when done
5. Update the "Last Checkpoint" section with your progress
6. Commit with message referencing the phase and step (e.g., "Phase 1.2: Add PlaylistItem struct")

---

## Document History

| Date | Author | Changes |
|------|--------|---------|
| 2025-12-24 | Claude (architect review) | Initial draft |
| 2025-12-25 | Claude + Nicholas | Major revision: mutual exclusivity, command structure (Option D), keyboard controls, removed IPC/priority system, simplified design |
| 2025-12-28 | Claude + Nicholas | Added: `--once` flag, lock file for instance prevention, cross-terminal control clarification (data vs process commands), empty playlist handling, preview behavior, foreground-only architecture (v1), state persistence timing |
| 2025-12-29 | Claude (architect + senior review) | Added: Complete module structure, Rust type definitions, Runner trait pattern, InstanceLock implementation, KeyboardListener implementation, FileMonitor generalization, error variants, dependency list, implementation code patterns |
| 2026-01-05 | Claude (TDD review) | Major revision: Restructured for TDD approach. Replaced Migration Path with comprehensive TDD Implementation Guide. Added 100+ test cases with actual test code. Added granular checklists with checkboxes for each phase. Added Definition of Done criteria for each phase. Reorganized to show tests before implementation. Added Test Summary section with test counts per phase. |
| 2026-01-05 | Claude (deep review) | Fixed: Atomic lock implementation using OS-level flock/LockFile. Fixed: Main loop design to prevent missed keyboard events. Added: ID generation reference to scheduler.rs. Fixed: RuntimeState.load() to return Self directly (infallible). Added: Config struct updates with new fields. Fixed: Error variants to match existing pattern (no thiserror). Added: Help text constants. Added: Test for empty playlist remove. Fixed: State persistence timing to save before display. Added: Edge case tests (concurrent access, permissions, clock handling). Completed: Phase 3.6 with integration tests. Added: Implementation Progress Tracker section for resumable implementation. |
| 2026-01-05 | Claude (codebase alignment review) | **Major architecture alignment with existing codebase**: Fixed ProcessController integration to use should_shutdown() polling pattern (matching daemon.rs/cycle.rs). Added complete PlaylistRunner implementation with execute_widget() and handle_message() integration. Consolidated duplicate KeyboardListener definitions. Added load_silent()/save_silent() patterns for Playlist (matching scheduler.rs). Added CLI display helpers (print_progress, print_error, print_success). Fixed MockInput to not be cfg(test) for integration test availability. Added Test Utilities section with test_item() and create_test_playlist() helpers. Ensured all code follows established patterns in errors.rs, config.rs, process_control.rs, api_broker.rs, and widgets/resolver.rs. |
| 2026-01-05 | Claude (final quality review) | **DRY and quality improvements**: Fixed FileMonitor to use existing VestaboardError::io_error() pattern. Added error_to_display_message() handling for new error types (LockError, InputError). Updated Dependencies section with complete winapi features for Windows file locking. Verified all patterns align with existing codebase (errors.rs, widget_utils.rs, cli_display.rs, process_control.rs). |
| 2026-01-05 | Claude (simplification review) | **Removed over-engineering**: Removed RateLimiter (skip now works immediately). Removed widget validation at add-time (validated at execution via execute_widget(), matching schedule pattern). Removed DisplayConfig. Simplified PlaylistItem::new() to not return Result. Removed KNOWN_WIDGETS constant. Updated tests to reflect simplified API. |
| 2026-01-06 | Claude (architect + senior review) | **Bug fixes**: Fixed skip_to_next() to reset last_display_time so item displays immediately. Fixed self.state.clone() to self.state (Copy type). Removed dead code is_pid_running() (OS flock handles stale locks). Removed orphaned show_errors_on_vestaboard config. Fixed FileMonitor.reload() to handle parse errors gracefully without propagating. |
