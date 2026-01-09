# Phase 5: Cleanup & Reference

**Goal**: Remove deprecated code and finalize the implementation.

**Prerequisites**: Complete Phase 1-4 first.

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

## Phase 5 Checklist

- [x] **Remove**: `vbl daemon` command from CLI
- [x] **Remove**: `vbl cycle` and `vbl cycle repeat` commands
- [x] **Delete**: `src/daemon.rs`
- [x] **Delete**: `src/cycle.rs`
- [x] **Update**: README and documentation (comments updated throughout codebase)
- [x] **Run**: Full test suite (269 tests pass)
- [ ] **Manual test**: All remaining commands
- [ ] **Commit**: "Remove deprecated daemon and cycle commands"

---

## Phase 5 Definition of Done

- [x] `vbl daemon` - command not found
- [x] `vbl cycle` - command not found
- [x] No references to `daemon.rs` or `cycle.rs` in codebase
- [x] README updated with new commands (comments updated throughout)
- [x] `cargo test` - all tests pass (269 tests)
- [x] No orphaned code (run `cargo clippy` and check for dead code warnings)

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
| 2026-01-06 | Claude | **Document split**: Split monolithic architecture document into phase-based documents for easier context management. |

---

## Related Documents

- [00-overview.md](00-overview.md) - Shared context (read first)
- [01-phase-1-foundation.md](01-phase-1-foundation.md) - Phase 1 implementation
- [02-phase-2-cli.md](02-phase-2-cli.md) - Phase 2 implementation
- [03-phase-3-execution.md](03-phase-3-execution.md) - Phase 3 implementation
- [04-phase-4-schedule.md](04-phase-4-schedule.md) - Phase 4 implementation
