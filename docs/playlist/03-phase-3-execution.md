# Phase 3: Playlist Execution

**Goal**: Implement `vbl playlist run` with interactive controls.

**Prerequisites**:
- Read [00-overview.md](00-overview.md) for shared context
- Complete [01-phase-1-foundation.md](01-phase-1-foundation.md) and [02-phase-2-cli.md](02-phase-2-cli.md) first

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

## Phase 3 Checklist

### 3.1 Create PlaylistRunner struct

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

### 3.2 Implement state machine

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

### 3.3 Implement keyboard handling

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

### 3.4 Add --once, --index, --id flags

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

### 3.5 Add state persistence

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

### 3.6 Wire up to CLI and test end-to-end

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
- [ ] **Write integration test**: End-to-end playlist run (uses mock Vestaboard)
- [ ] **Manual test**: Full interactive session
- [ ] **Commit**: "Implement playlist run with interactive controls"

---

## Phase 3 Definition of Done

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

## Next Phase

Continue to [04-phase-4-schedule.md](04-phase-4-schedule.md) for schedule refactoring.
