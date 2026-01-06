# Phase 4: Schedule Refactoring

**Goal**: Replace `vbl daemon` with `vbl schedule run`.

**Prerequisites**:
- Read [00-overview.md](00-overview.md) for shared context
- Complete Phase 1-3 first

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

## Phase 4 Checklist

### 4.1 Create ScheduleRunner

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

### 4.2 Add schedule run CLI command

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

### 4.3 Add deprecation warnings

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

---

## Phase 4 Definition of Done

- [ ] `cargo test schedule_runner` - all tests pass
- [ ] `vbl schedule run` - works like daemon but with keyboard controls
- [ ] `vbl daemon` - still works but shows deprecation warning
- [ ] Press `q` - exits cleanly
- [ ] Press `?` - shows help
- [ ] Past tasks are skipped on startup
- [ ] Schedule changes are hot-reloaded

**Test count checkpoint**: Phase 4 should add approximately 15-20 new tests.

---

## Next Phase

Continue to [05-phase-5-cleanup.md](05-phase-5-cleanup.md) for cleanup and final steps.
