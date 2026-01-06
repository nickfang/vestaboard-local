# Playlist Feature Architecture - Overview

This document provides shared context for all implementation phases. Read this first before working on any phase.

---

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

## Related Documents

- [01-phase-1-foundation.md](01-phase-1-foundation.md) - Rust types, state machine, Phase 1 implementation
- [02-phase-2-cli.md](02-phase-2-cli.md) - CLI commands, Phase 2 implementation
- [03-phase-3-execution.md](03-phase-3-execution.md) - Runner framework, Phase 3 implementation
- [04-phase-4-schedule.md](04-phase-4-schedule.md) - Schedule refactoring, Phase 4 implementation
- [05-phase-5-cleanup.md](05-phase-5-cleanup.md) - Cleanup, testing, progress tracker
