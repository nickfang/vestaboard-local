# Schedule Feature

This document provides a comprehensive overview of the Schedule feature for the Vestaboard Local CLI. It explains how the feature works, the architectural decisions made, and key implementation details.

---

## Overview

The Schedule feature enables **time-triggered task execution** on the Vestaboard. Unlike playlists (which rotate at intervals), schedules execute specific tasks at specific times.

**Key characteristics:**
- Executes tasks at their scheduled UTC times
- Skips past-due tasks on startup (no "catch up")
- Provides interactive keyboard controls
- Hot-reloads when the schedule file changes
- Only executes tasks once per schedule entry

---

## Command Reference

```bash
# Schedule management
vbl schedule add "2025-01-15 08:00" text "good morning"   # Add a scheduled task
vbl schedule add "2025-01-15 18:00" weather              # Add weather at 6 PM
vbl schedule list                                         # Show all scheduled tasks
vbl schedule remove <id>                                  # Remove task by ID
vbl schedule clear                                        # Remove all tasks
vbl schedule preview                                      # Dry-run all tasks

# Schedule execution
vbl schedule run                      # Run schedule (waits for and executes tasks)
vbl schedule run --dry-run            # Preview mode (console only)
```

---

## Architecture

### Data Model

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

| Field | Type | Description |
|-------|------|-------------|
| `tasks` | array | Scheduled tasks in chronological order |
| `tasks[].id` | string | Auto-generated unique identifier |
| `tasks[].time` | DateTime<Utc> | When to execute (UTC) |
| `tasks[].widget` | string | Widget type (weather, text, sat-word, jokes, clear, file) |
| `tasks[].input` | Value | Widget-specific input |

### Module Structure

```
src/
├── scheduler.rs             # Schedule data model, CRUD, ScheduleMonitor
├── runner/
│   ├── mod.rs               # Runner trait, ControlFlow enum
│   ├── common.rs            # Shared execute_and_send function
│   ├── schedule_runner.rs   # Core execution logic
│   ├── lock.rs              # Instance lock (prevents multiple runs)
│   └── keyboard.rs          # Keyboard input handling
```

### Key Components

1. **ScheduleRunner** (`src/runner/schedule_runner.rs`)
   - Implements the `Runner` trait
   - Skips past tasks, waits for future tasks
   - Tracks executed task IDs to prevent re-execution
   - Handles keyboard input (q to quit, ? for help)

2. **ScheduleMonitor** (`src/scheduler.rs`)
   - Watches schedule file for changes
   - Reloads schedule on modification
   - Used for hot-reload functionality

3. **InstanceLock** (`src/runner/lock.rs`)
   - Prevents multiple instances from running simultaneously
   - Shared with playlist feature

---

## How It Works

### Execution Flow

```
┌─────────────────────────────────────────────────────────────┐
│  vbl schedule run                                           │
│                                                             │
│  1. Load config and schedule file                          │
│  2. Acquire instance lock (fail if already running)        │
│  3. Setup keyboard listener and signal handler             │
│  4. Enter main loop:                                       │
│     ┌───────────────────────────────────────────────────┐  │
│     │ a. Check for shutdown signal (Ctrl+C)             │  │
│     │ b. Check for keyboard input (non-blocking)        │  │
│     │ c. Check for schedule file changes (hot-reload)   │  │
│     │ d. Find next pending task (future, not executed)  │  │
│     │ e. If task is due:                                │  │
│     │    - Execute widget → generate message            │  │
│     │    - Send to Vestaboard                          │  │
│     │    - Mark task as executed                       │  │
│     │ f. Sleep 100ms (prevents busy-loop)              │  │
│     └───────────────────────────────────────────────────┘  │
│  5. Cleanup and release lock                               │
└─────────────────────────────────────────────────────────────┘
```

### Task Filtering

The schedule runner maintains a set of executed task IDs and filters tasks:

```rust
fn next_pending_task(&self) -> Option<&ScheduledTask> {
    let now = Utc::now();
    self.schedule.tasks.iter()
        .filter(|task| task.time > now)           // Future tasks only
        .filter(|task| !self.executed_task_ids.contains(&task.id))  // Not executed
        .min_by_key(|task| task.time)             // Earliest first
}
```

### Keyboard Controls

| Key | Action | Description |
|-----|--------|-------------|
| `q` | Quit | Exit cleanly |
| `?` | Help | Show available commands |

**Note**: Schedule does not have pause/resume. Pausing a schedule creates complexity around missed tasks that isn't worth the added functionality. If you need to stop, quit and restart.

---

## Design Decisions

### Why Skip Past-Due Tasks?

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
- Eliminates confusing "catch up" behavior after restart
- Simple and predictable: what you see is what will happen
- If the old daemon behavior was needed, it would have required complex "catch up" logic

### Why No Pause/Resume for Schedule?

Unlike playlists, schedules don't have pause/resume controls.

**Rationale**:
- Schedules are time-based, not interval-based
- Pausing creates complexity: what happens to tasks that were due during the pause?
- If you need to stop temporarily, just quit and restart
- Keeps the schedule runner simple and predictable

### Why Use Wall Clock for Schedules?

Schedules use wall clock time (UTC) rather than monotonic time.

| Context | Clock Type | Rationale |
|---------|------------|-----------|
| Schedule comparison | Wall clock (UTC) | Schedules specify wall clock times |
| Playlist intervals | Monotonic (`Instant`) | Intervals should be consistent elapsed time |

**Backward clock jump**: If the clock jumps backward and a previously-executed task becomes "due" again, it will re-execute. This is acceptable - it indicates something unusual happened with the system clock, and re-showing a message is not harmful.

### Why Clear Executed Set on Reload?

When the schedule file is hot-reloaded, the executed task IDs set is cleared.

**Rationale**:
- The user edited the schedule, so they likely want tasks to execute
- Allows re-adding a task that was previously executed
- Simple mental model: editing the file = fresh start

### Why Mutual Exclusivity with Playlist?

Schedule and Playlist cannot run simultaneously. Only one controls the Vestaboard at a time.

**Rationale**:
- **Simplicity**: No collision handling, priority system, or arbitration needed
- **Clear mental model**: One thing controls the board at a time
- **Easier debugging**: You always know what's controlling the display

Both features use the same lock file (`data/vestaboard.lock`) to enforce this.

### Why Replace vbl daemon?

The original `vbl daemon` command has been removed and replaced with `vbl schedule run`.

**Rationale**:
- Consistent command structure: `vbl <feature> run`
- Interactive keyboard controls (the daemon was fully background)
- Clear naming that matches the data model (schedule.json → schedule run)

---

## Integration Points

### Widget Execution

Uses the same widget system as all other commands:

```rust
// From src/widgets/resolver.rs
let message = execute_widget(&task.widget, &task.input).await?;
```

### API Communication

Uses the existing message broker:

```rust
// From src/api_broker.rs
handle_message(message, MessageDestination::Vestaboard).await
```

### Error Handling

Widget failures are logged and displayed on the Vestaboard (using `error_to_display_message()`), then execution continues to the next task. This matches the behavior of playlist and other commands.

---

## Comparison with Playlist

| Aspect | Schedule | Playlist |
|--------|----------|----------|
| **Trigger** | Specific times | Fixed intervals |
| **Data file** | `schedule.json` | `playlist.json` |
| **Execution** | Once per task | Loops continuously |
| **Past items** | Skipped | N/A (no time concept) |
| **Pause/Resume** | No | Yes |
| **Next key** | N/A | Skip to next item |
| **Use case** | "Show weather at 8 AM" | "Rotate content every 5 min" |

---

## File Locations

| File | Purpose | Default Location |
|------|---------|------------------|
| Schedule | Task definitions | `data/schedule.json` |
| Lock File | Instance prevention | `data/vestaboard.lock` |

Paths are configured in `~/.vestaboard/config.toml` or use defaults.

---

## Testing

The schedule feature has comprehensive test coverage:

- **Unit tests**: `cargo test scheduler` (data model, CRUD, monitor)
- **Runner tests**: `cargo test schedule_runner` (task filtering, keyboard handling)
- **Lock tests**: `cargo test lock` (instance prevention)
- **CLI tests**: `cargo test cli_setup` (command parsing)

Total: ~35 tests for schedule-related functionality.

---

## DateTime Handling

### Input Format

When adding tasks via CLI, times can be specified in various formats:

```bash
# ISO 8601
vbl schedule add "2025-01-15T08:00:00Z" weather

# Date and time (local timezone, converted to UTC)
vbl schedule add "2025-01-15 08:00" text "hello"

# Relative (if supported)
vbl schedule add "tomorrow 9am" weather
```

### Storage Format

All times are stored as UTC in ISO 8601 format:

```json
{
  "time": "2025-01-15T08:00:00Z"
}
```

### Comparison

The `is_or_before()` function from `src/datetime.rs` handles time comparison with appropriate tolerance for clock skew.
