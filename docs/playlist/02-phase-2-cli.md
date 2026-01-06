# Phase 2: Playlist CLI

**Goal**: Add playlist management commands.

**Prerequisites**:
- Read [00-overview.md](00-overview.md) for shared context
- Complete [01-phase-1-foundation.md](01-phase-1-foundation.md) first

---

## Command Reference

All commands related to a feature are grouped under that feature's namespace. This provides discoverability - typing `vbl playlist --help` shows everything you can do with playlists.

```bash
# Playlist management
vbl playlist add weather
vbl playlist add text "welcome"
vbl playlist add sat-word
vbl playlist list
vbl playlist remove <id>
vbl playlist clear
vbl playlist interval 300         # Set rotation interval (seconds)
vbl playlist preview              # Dry-run all playlist items

# Playlist execution (Phase 3)
vbl playlist run                  # Long-running: rotate through items (loops forever)
vbl playlist run --once           # Run through playlist once, then exit
vbl playlist run --index 3        # Start from index 3
vbl playlist run --id abc1        # Start from item with id "abc1"
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

## Phase 2 Checklist

### 2.1 Add Playlist CLI structure

- [x] **Write test**: `test_cli_parses_playlist_add`
- [x] **Write test**: `test_cli_parses_playlist_list`
- [x] **Write test**: `test_cli_parses_playlist_remove`
- [x] **Write test**: `test_cli_parses_playlist_clear`
- [x] **Write test**: `test_cli_parses_playlist_interval`
- [x] **Write test**: `test_cli_parses_playlist_preview`
- [x] **Run tests** - fail
- [x] **Implement**: Add `Playlist` command variants to `cli_setup.rs`
- [x] **Run tests** - pass
- [x] **Commit**: "Add playlist CLI command parsing"

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

### 2.2 Wire up playlist commands to main.rs

- [ ] **Write integration test**: `test_playlist_add_creates_item` *(deferred - manual testing done)*
- [ ] **Write integration test**: `test_playlist_list_shows_items` *(deferred - manual testing done)*
- [ ] **Write integration test**: `test_playlist_remove_deletes_item` *(deferred - manual testing done)*
- [x] **Implement**: Handle `Command::Playlist` in `main.rs`
- [x] **Run tests** - pass
- [x] **Manual test**: Run `vbl playlist add weather`, `vbl playlist list`
- [x] **Commit**: "Wire up playlist CLI commands"

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

---

## Phase 2 Definition of Done

- [x] `cargo test cli_setup` - all tests pass (including new playlist tests)
- [x] `vbl playlist add weather` - adds item, shows confirmation
- [x] `vbl playlist add text "hello world"` - adds text item
- [x] `vbl playlist list` - shows all items with IDs
- [x] `vbl playlist remove <id>` - removes item
- [x] `vbl playlist clear` - removes all items
- [x] `vbl playlist interval 120` - sets interval
- [x] `vbl playlist preview` - shows all items (dry-run)
- [x] `vbl playlist --help` - shows help for all subcommands
- [x] Error handling: invalid widget type shows helpful message

**Test count checkpoint**: Phase 2 added 7 new CLI parsing tests (230 total tests).

---

## Next Phase

Continue to [03-phase-3-execution.md](03-phase-3-execution.md) for playlist execution implementation.
