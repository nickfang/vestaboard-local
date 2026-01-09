# Playlist Feature

This document provides a comprehensive overview of the Playlist feature for the Vestaboard Local CLI. It explains how the feature works, the architectural decisions made, and key implementation details.

---

## Overview

The Playlist feature enables **interval-based content rotation** on the Vestaboard. Unlike schedules (which execute at specific times), playlists continuously rotate through a set of items at a fixed interval.

**Key characteristics:**
- Rotates through items sequentially at a configurable interval
- Supports pause/resume with position memory
- Persists state across restarts
- Provides interactive keyboard controls
- Hot-reloads when the playlist file changes

---

## Command Reference

```bash
# Playlist management
vbl playlist add weather              # Add a widget to the playlist
vbl playlist add text "welcome"       # Add text with content
vbl playlist list                     # Show all playlist items
vbl playlist remove <id>              # Remove item by ID
vbl playlist clear                    # Remove all items
vbl playlist interval [seconds]       # Get/set rotation interval (min: 60s, default: 300s)
vbl playlist preview                  # Dry-run all items without delays

# Playlist execution
vbl playlist run                      # Start from index 0 (loops forever)
vbl playlist run --resume             # Continue from last saved position
vbl playlist run --once               # Run through once, then exit
vbl playlist run --index 3            # Start from index 3
vbl playlist run --id abc1            # Start from item with ID "abc1"
vbl playlist run --dry-run            # Preview mode (console only)
```

---

## Architecture

### Data Model

**File**: `data/playlist.json`

```json
{
  "interval_seconds": 300,
  "items": [
    {
      "id": "abc1",
      "widget": "weather",
      "input": null
    },
    {
      "id": "def2",
      "widget": "text",
      "input": "welcome to our office"
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `interval_seconds` | u64 | Time between rotations (min: 60, default: 300) |
| `items` | array | Playlist entries in display order |
| `items[].id` | string | Auto-generated unique identifier |
| `items[].widget` | string | Widget type (weather, text, sat-word, jokes, clear, file) |
| `items[].input` | Value | Widget-specific input (null for widgets that don't need input) |

### Runtime State

**File**: `data/runtime_state.json`

```json
{
  "playlist_state": "Paused",
  "playlist_index": 3,
  "last_shown_time": "2025-01-15T14:32:00Z"
}
```

State is saved **before** each item is displayed. This ensures crash recovery retries the current item rather than skipping it.

### Module Structure

```
src/
├── playlist.rs              # Playlist data model + CRUD operations
├── runtime_state.rs         # State persistence (best-effort)
├── runner/
│   ├── mod.rs               # Runner trait, ControlFlow enum
│   ├── playlist_runner.rs   # Core execution logic
│   ├── lock.rs              # Instance lock (prevents multiple runs)
│   └── keyboard.rs          # Keyboard input handling
```

### Key Components

1. **PlaylistRunner** (`src/runner/playlist_runner.rs`)
   - Implements the `Runner` trait
   - Manages state machine (Stopped → Running ↔ Paused)
   - Handles keyboard input
   - Integrates with widget execution

2. **InstanceLock** (`src/runner/lock.rs`)
   - Prevents multiple instances from running simultaneously
   - Uses lock file with PID for stale lock detection
   - Auto-releases on process exit (RAII)

3. **KeyboardListener** (`src/runner/keyboard.rs`)
   - Non-blocking keyboard input via background thread
   - Uses crossterm for cross-platform support

---

## How It Works

### Execution Flow

```
┌─────────────────────────────────────────────────────────────┐
│  vbl playlist run                                            │
│                                                              │
│  1. Load config and playlist file                           │
│  2. Acquire instance lock (fail if already running)         │
│  3. Setup keyboard listener and signal handler              │
│  4. Enter main loop:                                        │
│     ┌────────────────────────────────────────────────────┐  │
│     │ a. Check for shutdown signal (Ctrl+C)              │  │
│     │ b. Check for keyboard input (non-blocking)         │  │
│     │ c. Check for playlist file changes (hot-reload)    │  │
│     │ d. If interval elapsed and running:                │  │
│     │    - Save state                                    │  │
│     │    - Execute widget → generate message             │  │
│     │    - Send to Vestaboard                           │  │
│     │    - Advance to next item                         │  │
│     │ e. Sleep 100ms (prevents busy-loop)               │  │
│     └────────────────────────────────────────────────────┘  │
│  5. Cleanup and release lock                                │
└─────────────────────────────────────────────────────────────┘
```

### State Machine

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

### Keyboard Controls

| Key | Action | Description |
|-----|--------|-------------|
| `p` | Pause | Stop rotation, remember position and remaining time |
| `r` | Resume | Continue from paused position with preserved timing |
| `n` | Next | Show next item immediately (or queue if paused) |
| `q` | Quit | Exit cleanly |
| `?` | Help | Show available commands |

### Timing Preservation

When pausing and resuming, the **remaining time** before the next item is preserved:

```
Example: 60-second interval

T=0:   Display A, start 60s timer
T=30:  User pauses (30 seconds remaining)
T=90:  User resumes
       → Timer adjusted: still need 30 more seconds
T=120: 30 seconds elapsed since resume, B displays
```

**Exception**: If 'n' was pressed while paused, the next item displays immediately on resume.

---

## Design Decisions

### Why Separate Playlist from Schedule?

**Problem**: The original `vbl cycle` command reused `schedule.json` but ignored the time fields. This created confusion:
- Users scheduled tasks with specific times, but cycle mode discarded this information
- Two different execution semantics shared one data model
- The relationship between daemon and cycle modes was unclear

**Solution**: Separate data models with distinct semantics:
- **Schedule**: Time-triggered ("show weather at 8:00 AM")
- **Playlist**: Interval-based ("rotate through items every 5 minutes")

### Why Mutual Exclusivity?

Schedule and Playlist cannot run simultaneously. Only one controls the Vestaboard at a time.

**Rationale**:
- **Simplicity**: No collision handling, priority system, or arbitration needed
- **Clear mental model**: One thing controls the board at a time
- **Easier debugging**: You always know what's controlling the display

### Why Foreground Processes?

Both `vbl playlist run` and `vbl schedule run` are foreground processes requiring an attached terminal.

**Rationale**:
- Keyboard controls work via stdin
- Ctrl+C triggers graceful shutdown
- For persistence, users can use `screen`, `tmux`, or `nohup`
- Avoids complexity of daemon management and IPC

### Why Save State Before Display?

State is saved **before** displaying each item, not after.

**Rationale**:
- If the process crashes during display or API call, the state reflects the item that was about to be shown
- On restart, we retry that item rather than skip it
- Worst case: an item might show twice (acceptable) rather than be skipped (frustrating)

### Why Best-Effort State Persistence?

`RuntimeState.load()` and `RuntimeState.save()` are infallible (never return errors).

**Rationale**:
- State persistence is non-critical - it's a convenience feature
- Crashing because of state file corruption would be worse than losing position
- On any error (file not found, corrupted JSON), return defaults and continue

### Why 60-Second Minimum Interval?

The minimum interval is 60 seconds to prevent API rate limiting and excessive Vestaboard wear.

### Why Lock Files Instead of OS-Level Locking?

The implementation uses a JSON lock file with PID checking rather than `flock`/`LockFileEx`.

**Rationale**:
- Sufficient for single-user CLI use case
- Cross-platform without platform-specific code
- Human-readable for debugging
- **Note**: There's a small race window between checking and acquiring, but this is acceptable for the use case

---

## Integration Points

### Widget Execution

Uses the same widget system as all other commands:

```rust
// From src/widgets/resolver.rs
let message = execute_widget(&item.widget, &item.input).await?;
```

### API Communication

Uses the existing message broker:

```rust
// From src/api_broker.rs
handle_message(message, MessageDestination::Vestaboard).await
```

### Error Handling

Widget failures are displayed on the Vestaboard (using `error_to_display_message()`) and execution continues. This matches the behavior of other commands and ensures the user knows something went wrong.

---

## File Locations

| File | Purpose | Default Location |
|------|---------|------------------|
| Playlist | Item definitions | `data/playlist.json` |
| Runtime State | Position persistence | `data/runtime_state.json` |
| Lock File | Instance prevention | `data/vestaboard.lock` |

Paths are configured in `~/.vestaboard/config.toml` or use defaults.

---

## Testing

The playlist feature has comprehensive test coverage:

- **Unit tests**: `cargo test playlist` (data model, CRUD operations)
- **Runner tests**: `cargo test playlist_runner` (state machine, keyboard handling)
- **Lock tests**: `cargo test lock` (instance prevention)
- **CLI tests**: `cargo test cli_setup` (command parsing)

Total: ~60 tests for playlist-related functionality.
