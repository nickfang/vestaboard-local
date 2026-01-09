//! Tests for the ScheduleRunner.

use chrono::{Duration, Utc};
use crossterm::event::KeyCode;
use serde_json::json;

use crate::runner::schedule_runner::ScheduleRunner;
use crate::runner::{ControlFlow, Runner};
use crate::scheduler::{Schedule, ScheduledTask};

fn create_task(id: &str, hours_offset: i64, widget: &str) -> ScheduledTask {
    let now = Utc::now();
    let time = if hours_offset >= 0 {
        now + Duration::hours(hours_offset)
    } else {
        now - Duration::hours(-hours_offset)
    };
    ScheduledTask {
        id: id.to_string(),
        time,
        widget: widget.to_string(),
        input: json!(null),
    }
}

fn create_test_schedule() -> Schedule {
    Schedule {
        tasks: vec![
            create_task("past", -2, "weather"),
            create_task("future", 1, "text"),
        ],
    }
}

#[test]
fn test_schedule_runner_identifies_next_task() {
    let schedule = create_test_schedule();
    let runner = ScheduleRunner::new(schedule, false);

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

    let runner = ScheduleRunner::new(schedule, false);
    let next = runner.next_pending_task();

    assert!(next.is_none()); // All tasks in past
}

#[test]
fn test_schedule_runner_q_key_exits() {
    let schedule = create_test_schedule();
    let mut runner = ScheduleRunner::new(schedule, false);

    let result = runner.handle_key(KeyCode::Char('q'));
    assert_eq!(result, ControlFlow::Exit);
}

#[test]
fn test_schedule_runner_capital_q_key_exits() {
    let schedule = create_test_schedule();
    let mut runner = ScheduleRunner::new(schedule, false);

    let result = runner.handle_key(KeyCode::Char('Q'));
    assert_eq!(result, ControlFlow::Exit);
}

#[test]
fn test_schedule_runner_help_text() {
    let schedule = create_test_schedule();
    let runner = ScheduleRunner::new(schedule, false);

    let help = runner.help_text();
    assert!(help.contains("q"));
    assert!(help.contains("quit") || help.contains("Quit"));
}

#[test]
fn test_schedule_runner_unknown_key_continues() {
    let schedule = create_test_schedule();
    let mut runner = ScheduleRunner::new(schedule, false);

    let result = runner.handle_key(KeyCode::Char('x'));
    assert_eq!(result, ControlFlow::Continue);
}

#[test]
fn test_schedule_runner_help_key_continues() {
    let schedule = create_test_schedule();
    let mut runner = ScheduleRunner::new(schedule, false);

    let result = runner.handle_key(KeyCode::Char('?'));
    assert_eq!(result, ControlFlow::Continue);
}

#[test]
fn test_schedule_runner_returns_tasks_in_chronological_order() {
    let now = Utc::now();
    let schedule = Schedule {
        tasks: vec![
            ScheduledTask {
                id: "later".to_string(),
                time: now + Duration::hours(2),
                widget: "text".to_string(),
                input: json!("later"),
            },
            ScheduledTask {
                id: "sooner".to_string(),
                time: now + Duration::hours(1),
                widget: "weather".to_string(),
                input: json!(null),
            },
        ],
    };

    let runner = ScheduleRunner::new(schedule, false);
    let next = runner.next_pending_task();

    // Should return the sooner task (1 hour from now), not the later one
    assert!(next.is_some());
    assert_eq!(next.unwrap().id, "sooner");
}

#[test]
fn test_schedule_runner_empty_schedule() {
    let schedule = Schedule { tasks: vec![] };
    let runner = ScheduleRunner::new(schedule, false);

    let next = runner.next_pending_task();
    assert!(next.is_none());
}

#[test]
fn test_schedule_runner_marks_task_executed() {
    let schedule = create_test_schedule();
    let mut runner = ScheduleRunner::new(schedule, false);

    // Initially, the future task should be pending
    let next = runner.next_pending_task();
    assert!(next.is_some());
    let task_id = next.unwrap().id.clone();

    // Mark it as executed
    runner.mark_executed(&task_id);

    // Now it shouldn't be returned as pending
    let next_after = runner.next_pending_task();
    assert!(next_after.is_none() || next_after.unwrap().id != task_id);
}

#[test]
fn test_schedule_runner_time_until_next_with_future_task() {
    let schedule = create_test_schedule();
    let runner = ScheduleRunner::new(schedule, false);

    let duration = runner.time_until_next_task();
    assert!(duration.is_some());
    // Should be roughly 1 hour (3600 seconds) minus a small margin for test execution
    let secs = duration.unwrap().as_secs();
    assert!(secs > 3500 && secs < 3700, "Expected ~3600 seconds, got {}", secs);
}

#[test]
fn test_schedule_runner_time_until_next_with_no_tasks() {
    let schedule = Schedule { tasks: vec![] };
    let runner = ScheduleRunner::new(schedule, false);

    let duration = runner.time_until_next_task();
    assert!(duration.is_none());
}

#[test]
fn test_schedule_runner_time_until_next_all_past() {
    let now = Utc::now();
    let schedule = Schedule {
        tasks: vec![
            ScheduledTask {
                id: "past1".to_string(),
                time: now - Duration::hours(2),
                widget: "weather".to_string(),
                input: json!(null),
            },
        ],
    };

    let runner = ScheduleRunner::new(schedule, false);
    let duration = runner.time_until_next_task();
    assert!(duration.is_none());
}

#[test]
fn test_schedule_runner_dry_run_mode() {
    let schedule = create_test_schedule();
    let runner = ScheduleRunner::new(schedule, true);

    assert!(runner.is_dry_run());
}

#[test]
fn test_schedule_runner_reload_schedule() {
    let schedule = create_test_schedule();
    let mut runner = ScheduleRunner::new(schedule, false);

    // Create a new schedule with different tasks
    let new_schedule = Schedule {
        tasks: vec![
            create_task("new_task", 3, "sat-word"),
        ],
    };

    runner.reload_schedule(new_schedule);

    let next = runner.next_pending_task();
    assert!(next.is_some());
    assert_eq!(next.unwrap().id, "new_task");
}

#[test]
fn test_schedule_runner_reload_clears_executed_set() {
    let schedule = create_test_schedule();
    let mut runner = ScheduleRunner::new(schedule, false);

    // Mark the future task as executed
    runner.mark_executed("future");

    // Reload with the same schedule
    let new_schedule = create_test_schedule();
    runner.reload_schedule(new_schedule);

    // The task should be pending again (executed set cleared)
    let next = runner.next_pending_task();
    assert!(next.is_some());
    assert_eq!(next.unwrap().id, "future");
}
