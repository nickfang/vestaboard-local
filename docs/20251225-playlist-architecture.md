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
│   ├── rate_limiter.rs          # Message rate limiting
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

**When state is saved**: On every rotation (after each item is displayed). This ensures that if the process crashes or is killed, it can resume from approximately where it left off.

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    #[serde(default = "generate_item_id")]
    pub id: String,
    pub widget: String,
    pub input: Value,
}

fn generate_item_id() -> String {
    // Use same ID generation as ScheduledTask (nanoid)
    nanoid!(4, &CUSTOM_ALPHABET)
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

impl Playlist {
    pub fn add_item(&mut self, item: PlaylistItem) {
        self.items.push(item);
    }

    pub fn remove_item(&mut self, id: &str) -> bool {
        let len_before = self.items.len();
        self.items.retain(|item| item.id != id);
        self.items.len() < len_before
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn validate_interval(&self) -> Result<(), VestaboardError> {
        if self.interval_seconds < 60 {
            return Err(VestaboardError::ValidationError {
                message: "Playlist interval must be at least 60 seconds".to_string(),
            });
        }
        Ok(())
    }
}
```

### RuntimeState Struct

```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeState {
    pub playlist_state: PlaylistState,
    pub playlist_index: usize,
    pub last_shown_time: Option<DateTime<Utc>>,
}

impl RuntimeState {
    pub fn load(path: &Path) -> Result<Self, VestaboardError> {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_else(|e| {
                    log::warn!("Invalid runtime state, using defaults: {}", e);
                    Self::default()
                })
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(Self::default())
            }
            Err(e) => {
                log::warn!("Cannot read runtime state: {}", e);
                Ok(Self::default())
            }
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), VestaboardError> {
        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = std::fs::write(path, content) {
                    log::warn!("Cannot save runtime state: {}", e);
                    // Don't fail - state persistence is best-effort
                }
                Ok(())
            }
            Err(e) => {
                log::warn!("Cannot serialize runtime state: {}", e);
                Ok(())
            }
        }
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

## Rate Limiting

### Minimum Message Interval

**Rule**: No two messages may be sent within 30 seconds of each other.

| Situation | Behavior |
|-----------|----------|
| Playlist rotation due, last message < 30s ago | Delay until 30s elapsed |
| User presses `n` (next), last message < 30s ago | Show message: "Please wait N seconds..." then delay |

**Rationale**: 30 seconds provides comfortable reading time and avoids rapid flashing. Can be reduced later if needed.

### Playlist Minimum Interval

**Rule**: Playlist `interval_seconds` minimum is 60 seconds.

**Rationale**: Intervals under 60s create frantic display updates. Enforce at configuration time.

### RateLimiter Implementation

```rust
use std::time::{Duration, Instant};

pub struct RateLimiter {
    last_message: Option<Instant>,
    minimum_gap: Duration,
}

impl RateLimiter {
    pub fn new(minimum_gap_seconds: u64) -> Self {
        Self {
            last_message: None,
            minimum_gap: Duration::from_secs(minimum_gap_seconds),
        }
    }

    /// Returns the duration to wait, or None if no wait needed
    pub fn time_until_ready(&self) -> Option<Duration> {
        self.last_message.and_then(|last| {
            let elapsed = last.elapsed();
            if elapsed < self.minimum_gap {
                Some(self.minimum_gap - elapsed)
            } else {
                None
            }
        })
    }

    /// Wait if needed, then mark as sent
    pub async fn wait_and_mark(&mut self) {
        if let Some(wait_time) = self.time_until_ready() {
            tokio::time::sleep(wait_time).await;
        }
        self.last_message = Some(Instant::now());
    }

    /// Just mark as sent (for when message was already sent)
    pub fn mark_sent(&mut self) {
        self.last_message = Some(Instant::now());
    }
}
```

---

## Runner Framework

### Runner Trait

Both schedule and playlist runners share common patterns. Define a trait:

```rust
use async_trait::async_trait;
use crossterm::event::KeyCode;

pub enum ControlFlow {
    Continue,
    Exit,
}

#[async_trait]
pub trait Runner {
    /// Main execution loop
    async fn run(&mut self) -> Result<(), VestaboardError>;

    /// Handle a keyboard input, return whether to continue
    fn handle_key(&mut self, key: KeyCode) -> ControlFlow;

    /// Get help text for keyboard controls
    fn help_text(&self) -> &'static str;

    /// Called on graceful shutdown
    fn cleanup(&mut self);
}
```

### Common Runner Setup

```rust
use crate::runner::{lock::InstanceLock, keyboard::KeyboardListener};

pub async fn run_with_keyboard<R: Runner>(
    mut runner: R,
    mode: &str,  // "playlist" or "schedule"
) -> Result<(), VestaboardError> {
    // Acquire lock
    let lock = InstanceLock::acquire(mode)?;

    // Setup keyboard listener (spawns thread, returns channel)
    let mut keyboard = KeyboardListener::new()?;

    // Setup Ctrl+C handler
    let process_controller = ProcessController::new();
    process_controller.setup_signal_handler()?;

    // Show initial help hint
    println!("Press ? for help.");

    // Main loop
    loop {
        tokio::select! {
            // Check for keyboard input
            Some(key) = keyboard.next_key() => {
                match runner.handle_key(key) {
                    ControlFlow::Continue => {}
                    ControlFlow::Exit => break,
                }
            }

            // Check for shutdown signal
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                if process_controller.should_shutdown() {
                    break;
                }
            }
        }

        // Run one iteration of the runner
        if let Err(e) = runner.run().await {
            log::error!("Runner error: {}", e);
            // Continue running unless fatal
        }
    }

    // Cleanup
    runner.cleanup();
    drop(lock);  // Release lock (RAII)

    Ok(())
}
```

### Instance Lock Implementation

```rust
use std::fs::{self, File};
use std::io::{Read, Write};
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
}

impl InstanceLock {
    pub fn acquire(mode: &str) -> Result<Self, VestaboardError> {
        let path = PathBuf::from("data/vestaboard.lock");

        // Check for existing lock
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            if let Ok(lock_data) = serde_json::from_str::<LockData>(&content) {
                // Check if PID is still running
                if Self::is_pid_running(lock_data.pid) {
                    return Err(VestaboardError::LockError {
                        message: format!(
                            "{} already running (PID {}, started {})",
                            lock_data.mode,
                            lock_data.pid,
                            lock_data.started_at.format("%H:%M:%S")
                        ),
                    });
                }
                // Stale lock, will overwrite
                log::info!("Removing stale lock file (PID {} not running)", lock_data.pid);
            }
        }

        // Create lock file
        let lock_data = LockData {
            mode: mode.to_string(),
            pid: std::process::id(),
            started_at: Utc::now(),
        };

        let content = serde_json::to_string_pretty(&lock_data)
            .map_err(|e| VestaboardError::LockError {
                message: format!("Cannot serialize lock: {}", e),
            })?;

        fs::write(&path, content).map_err(|e| VestaboardError::LockError {
            message: format!("Cannot create lock file: {}", e),
        })?;

        Ok(Self { path })
    }

    #[cfg(unix)]
    fn is_pid_running(pid: u32) -> bool {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }

    #[cfg(windows)]
    fn is_pid_running(pid: u32) -> bool {
        // Windows implementation using OpenProcess
        use std::ptr::null_mut;
        unsafe {
            let handle = winapi::um::processthreadsapi::OpenProcess(
                winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION,
                0,
                pid,
            );
            if handle.is_null() {
                false
            } else {
                winapi::um::handleapi::CloseHandle(handle);
                true
            }
        }
    }
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path) {
            log::warn!("Cannot remove lock file: {}", e);
        }
    }
}
```

### Keyboard Input Implementation

```rust
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct KeyboardListener {
    receiver: mpsc::Receiver<KeyCode>,
    _handle: thread::JoinHandle<()>,
}

impl KeyboardListener {
    pub fn new() -> Result<Self, VestaboardError> {
        // Check if stdin is a TTY
        if !atty::is(atty::Stream::Stdin) {
            return Err(VestaboardError::InputError {
                message: "Interactive mode requires a terminal. Stdin is not a TTY.".to_string(),
            });
        }

        let (sender, receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            loop {
                // Poll for events with timeout
                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                        if sender.send(code).is_err() {
                            break; // Receiver dropped
                        }
                    }
                }
            }
        });

        Ok(Self {
            receiver,
            _handle: handle,
        })
    }

    pub fn next_key(&mut self) -> Option<KeyCode> {
        self.receiver.try_recv().ok()
    }
}

// For testing - abstract input source
pub trait InputSource: Send {
    fn next_key(&mut self) -> Option<KeyCode>;
}

impl InputSource for KeyboardListener {
    fn next_key(&mut self) -> Option<KeyCode> {
        self.receiver.try_recv().ok()
    }
}

// Mock for tests
#[cfg(test)]
pub struct MockInput {
    keys: std::collections::VecDeque<KeyCode>,
}

#[cfg(test)]
impl MockInput {
    pub fn new(keys: Vec<KeyCode>) -> Self {
        Self {
            keys: keys.into(),
        }
    }
}

#[cfg(test)]
impl InputSource for MockInput {
    fn next_key(&mut self) -> Option<KeyCode> {
        self.keys.pop_front()
    }
}
```

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
                self.current_data = serde_json::from_str(&content)?;
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
            Err(e) => Err(VestaboardError::IoError {
                message: format!("Cannot access file: {}", e),
            }),
        }
    }
}
```

---

## Error Handling

### New Error Variants

Add to `VestaboardError`:

```rust
pub enum VestaboardError {
    // ... existing variants ...

    #[error("Lock error: {message}")]
    LockError { message: String },

    #[error("Input error: {message}")]
    InputError { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },
}
```

### Widget Failures

**Standardized behavior across all execution modes:**

1. `log::error!("Widget '{}' failed: {}", widget, error)` - Always log
2. `eprintln!("Widget {} failed: {}", widget, error)` - Always print to console
3. Continue to next item - Don't stop the playlist/schedule
4. Vestaboard display of error - **Configurable** (default: no)

```toml
# In vblconfig.toml
[display]
show_errors_on_vestaboard = false  # default
```

**Rationale**: Showing "ERROR: WEATHER API DOWN" on a living room Vestaboard may not be desired. But can be enabled for debugging.

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

[display]
minimum_message_gap_seconds = 30
show_errors_on_vestaboard = false
```

---

## Migration Path

### Phase 1: Foundation (Non-Breaking)

**Goal**: Add new data structures and shared components without changing existing behavior.

**Tasks**:
1. Create `src/playlist.rs` - Playlist struct, PlaylistItem, CRUD operations
2. Create `src/runtime_state.rs` - RuntimeState struct, load/save
3. Create `src/file_monitor.rs` - Generic FileMonitor<T> extracted from ScheduleMonitor
4. Create `src/runner/mod.rs` - Runner trait, ControlFlow enum
5. Create `src/runner/lock.rs` - InstanceLock
6. Create `src/runner/keyboard.rs` - KeyboardListener, InputSource trait
7. Create `src/runner/rate_limiter.rs` - RateLimiter
8. Add new error variants to `errors.rs`
9. Add new config fields with defaults
10. All existing commands (`vbl daemon`, `vbl cycle`) continue to work unchanged

**Deliverables**:
- New modules with full test coverage
- No behavior changes to existing commands

### Phase 2: Playlist CLI

**Goal**: Add playlist management commands.

**Tasks**:
1. Add `vbl playlist` subcommand to CLI
2. Implement `add`, `list`, `remove`, `clear`, `interval`, `preview`
3. Wire up to `playlist.rs` functions

**Deliverables**:
- CLI commands for playlist management
- Playlist file CRUD operations
- Tests for CLI parsing and operations

### Phase 3: Playlist Execution

**Goal**: Implement `vbl playlist run` with interactive controls.

**Tasks**:
1. Create `src/runner/playlist_runner.rs`
2. Implement Runner trait for PlaylistRunner
3. Add keyboard controls (p, r, n, q, ?)
4. Implement playlist state machine
5. Add state persistence
6. Add `--once`, `--index`, `--id` flags
7. Wire up to CLI

**Deliverables**:
- Working `vbl playlist run` command
- Interactive keyboard controls
- State persistence across restarts
- Full test coverage

### Phase 4: Schedule Refactoring

**Goal**: Replace `vbl daemon` with `vbl schedule run`.

**Tasks**:
1. Create `src/runner/schedule_runner.rs`
2. Implement Runner trait for ScheduleRunner
3. Add keyboard controls (q, ?)
4. Remove overdue task execution (skip to next upcoming)
5. Add deprecation warnings for `vbl daemon`
6. Refactor existing daemon.rs to use new runner framework

**Deliverables**:
- `vbl schedule run` command
- Deprecation warnings on `vbl daemon`

### Phase 5: Cleanup (Breaking)

**Goal**: Remove deprecated code.

**Tasks**:
1. Remove `vbl daemon` command
2. Remove `vbl cycle` and `vbl cycle repeat` commands
3. Remove `src/cycle.rs`
4. Remove `src/daemon.rs`
5. Remove old `ScheduleMonitor` (replaced by generic `FileMonitor`)
6. Update documentation and README

**Deliverables**:
- Cleaner codebase
- Updated documentation and help text
- Consider major version bump if following semver

---

## Current Code Issues to Address

During implementation, the following issues from the current codebase should be resolved:

| Issue | Location | Resolution |
|-------|----------|------------|
| Compile error | `daemon.rs:79` | Fix undefined `CHECK_INTERVAL_SECONDS` reference |
| Code duplication | `cycle.rs` (3 places) | Use new `RateLimiter.wait_and_mark()` pattern |
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
libc = "0.2"            # PID checking on Unix

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["processthreadsapi", "handleapi", "winnt"] }
```

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
| `runner/rate_limiter.rs` | Wait timing, edge cases |
| `runner/keyboard.rs` | Key parsing (use MockInput) |
| `file_monitor.rs` | Change detection, reload behavior, error handling |

### Integration Tests

| Scenario | Validation |
|----------|------------|
| Playlist rotation | Items cycle correctly at specified interval |
| Pause/resume | State preserved correctly |
| Skip (next) | Advances to next item, respects rate limit |
| Quit | Clean exit, state saved, lock released |
| Start from index/id | Begins at correct position |
| Rate limiting | Messages respect 30s minimum gap |
| File changes | Hot-reload of playlist works |
| Lock contention | Second instance fails with clear error |
| Stale lock | Old lock from dead process is overwritten |

### Manual Testing Checklist

- [ ] Add items to playlist, verify they appear in list
- [ ] Run playlist, observe rotation at correct interval
- [ ] Press `p` to pause, verify rotation stops
- [ ] Press `r` to resume, verify rotation continues from same position
- [ ] Press `n` to skip, verify next item shows (after rate limit)
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

## Document History

| Date | Author | Changes |
|------|--------|---------|
| 2025-12-24 | Claude (architect review) | Initial draft |
| 2025-12-25 | Claude + Nicholas | Major revision: mutual exclusivity, command structure (Option D), keyboard controls, removed IPC/priority system, simplified design |
| 2025-12-28 | Claude + Nicholas | Added: `--once` flag, lock file for instance prevention, cross-terminal control clarification (data vs process commands), empty playlist handling, preview behavior, foreground-only architecture (v1), state persistence timing |
| 2025-12-29 | Claude (architect + senior review) | Added: Complete module structure, Rust type definitions, Runner trait pattern, InstanceLock implementation, KeyboardListener implementation, RateLimiter implementation, FileMonitor generalization, error variants, dependency list, implementation code patterns |
