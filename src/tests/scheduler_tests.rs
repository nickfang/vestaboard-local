#[path = "../scheduler.rs"]
mod scheduler;

use crate::config::DEFAULT_SCHEDULE_FILE_PATH;
use crate::errors::VestaboardError;
use crate::scheduler::{
  add_task_to_schedule, clear_schedule, list_schedule, load_schedule, remove_task_from_schedule,
  save_schedule, Schedule, ScheduledTask, ScheduleMonitor, CUSTOM_ALPHABET, ID_LENGTH,
};
use crate::widgets::text::get_text;
use chrono::{DateTime, TimeZone, Utc};
use serde_json::json;
use serial_test::serial;
use std::io::{Seek, Write};
use std::path::PathBuf;
use tempfile::NamedTempFile;

fn create_valid_json_content() -> (String, DateTime<Utc>, String) {
  let task_time = Utc.with_ymd_and_hms(2025, 5, 4, 18, 30, 0).unwrap();
  let task_id = "tst1".to_string();
  let schedule = Schedule {
    tasks: vec![ScheduledTask {
      id: task_id.clone(),
      time: task_time,
      widget: "test_widget".to_string(),
      input: json!({"value": "test_input"}),
    }],
  };
  let json_string = serde_json::to_string_pretty(&schedule).unwrap();
  (json_string, task_time, task_id)
}

#[cfg(test)]
#[test]
fn save_schedule_test() {
  use save_schedule;
  use std::io::Read;
  use tempfile::NamedTempFile;

  let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
  let path = temp_file.path().to_path_buf();

  // Test saving an empty schedule
  let empty_schedule = Schedule::default();
  let result = save_schedule(&empty_schedule, &path);
  assert!(result.is_ok());

  // Verify the saved content is an empty schedule
  let mut file_content = String::new();
  temp_file
    .read_to_string(&mut file_content)
    .expect("Failed to read from temp file");
  assert_eq!(file_content, "{\n  \"tasks\": []\n}");

  // Test saving a schedule with tasks
  let task1_time = Utc.with_ymd_and_hms(2025, 5, 1, 9, 0, 0).unwrap();
  let task1 = ScheduledTask::new(task1_time, "Weather".to_string(), json!({}));
  let task2_time = Utc.with_ymd_and_hms(2025, 5, 1, 17, 30, 0).unwrap();
  let task2 = ScheduledTask::new(
    task2_time,
    "text".to_string(),
    json!({"message": "Hello, world!"}),
  );

  let mut schedule = Schedule::default();
  schedule.add_task(task1);
  schedule.add_task(task2);

  let result = save_schedule(&schedule, &path);
  assert!(result.is_ok());

  // Verify the saved content matches the schedule with tasks
  temp_file
    .seek(std::io::SeekFrom::Start(0))
    .expect("Failed to seek to start of file");
  file_content.clear();
  temp_file
    .read_to_string(&mut file_content)
    .expect("Failed to read from temp file");

  let expected_json = serde_json::to_string_pretty(&schedule).unwrap();
  assert_eq!(file_content, expected_json);
}

#[test]
fn test_load_schedule_success() {
  let (json_content, expected_time, expected_id) = create_valid_json_content();
  let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
  write!(temp_file, "{}", json_content).expect("Failed to write to temp file");
  temp_file
    .as_file_mut()
    .flush()
    .expect("Failed to flush temp file");
  println!("Temporary file path: {:?}", temp_file.path());

  let path = temp_file.path().to_path_buf();
  let result = load_schedule(&path);

  assert!(result.is_ok());
  let schedule = result.unwrap();
  assert_eq!(schedule.tasks.len(), 1);
  assert_eq!(schedule.tasks[0].id, expected_id);
  assert_eq!(schedule.tasks[0].widget, "test_widget");
  assert_eq!(schedule.tasks[0].time, expected_time);
  assert_eq!(schedule.tasks[0].input["value"], "test_input");
}

#[test]
fn test_load_schedule_file_not_found() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
  let path = temp_dir.path().join("non_existent_schedule.json");

  let result = load_schedule(&path);

  assert!(result.is_ok());
  let schedule = result.unwrap();
  assert!(schedule.tasks.is_empty());
}

#[test]
fn test_load_schedule_empty_file() {
  let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
  // Write nothing (or just whitespace)
  write!(temp_file, "").expect("Failed to write empty string");
  temp_file.flush().expect("Failed to flush");

  let path = temp_file.path().to_path_buf();
  let result = load_schedule(&path);

  assert!(result.is_ok());
  let schedule = result.unwrap();
  assert!(schedule.tasks.is_empty());
}

#[test]
fn test_load_schedule_whitespace_file() {
  let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
  write!(temp_file, "  \n  \t  ").expect("Failed to write whitespace");
  temp_file.flush().expect("Failed to flush");

  let path = temp_file.path().to_path_buf();
  let result = load_schedule(&path);

  assert!(result.is_ok());
  let schedule = result.unwrap();
  assert!(schedule.tasks.is_empty()); // Should also return default empty schedule
}

#[test]
fn test_load_schedule_invalid_json() {
  let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
  write!(temp_file, "{{invalid json data, missing quotes}}").expect("Failed to write invalid JSON");
  temp_file.flush().expect("Failed to flush");

  let path = temp_file.path().to_path_buf();
  let result = load_schedule(&path);

  assert!(result.is_err());
  let err = result.unwrap_err();
  if let VestaboardError::JsonError { .. } = err {
    assert!(true);
  } else {
    panic!("Expected VestaboardError::JsonError");
  }
}

#[test]
fn test_schedule_serialization_deserialization() {
  use chrono::TimeZone;

  let mut schedule = Schedule::default();

  let task1_time = Utc.with_ymd_and_hms(2025, 5, 1, 9, 0, 0).unwrap();
  let task1 = ScheduledTask::new(task1_time, "Weather".to_string(), json!({}));
  let task1_id = task1.id.clone();
  assert_eq!(task1_id.len(), ID_LENGTH);
  assert!(task1_id.chars().all(|c| CUSTOM_ALPHABET.contains(&c)));

  let task2_time = Utc.with_ymd_and_hms(2025, 5, 1, 17, 30, 0).unwrap();
  let task2 = ScheduledTask::new(
    task2_time,
    "text".to_string(),
    json!({"message": "Hello, world!"}),
  );
  let task2_id = task2.id.clone();
  assert_eq!(task2.id.len(), ID_LENGTH);
  assert!(task2_id.chars().all(|c| CUSTOM_ALPHABET.contains(&c)));
  assert_ne!(task1_id, task2_id);

  schedule.add_task(task1);
  schedule.add_task(task2);

  assert_eq!(schedule.get_task(&task1_id).unwrap().id, task1_id);
  assert_eq!(schedule.get_task(&task2_id).unwrap().id, task2_id);

  let json_output = serde_json::to_string_pretty(&schedule).unwrap();

  let deserialized_schedule: Schedule = serde_json::from_str(&json_output).unwrap();

  assert_eq!(deserialized_schedule.tasks.len(), 2);
  assert_eq!(deserialized_schedule.tasks[0].id, task1_id);
  assert_eq!(deserialized_schedule.tasks[1].id, task2_id);
  assert_eq!(deserialized_schedule.tasks[0].time, task1_time);
  assert_eq!(deserialized_schedule.tasks[1].time, task2_time);
  assert_eq!(deserialized_schedule.tasks[0].widget, "Weather");
  assert_eq!(deserialized_schedule.tasks[1].widget, "text");
  assert_eq!(
    deserialized_schedule.tasks[1].input,
    json!({"message": "Hello, world!"})
  );

  let mut schedule_for_removal = deserialized_schedule;
  let removed = schedule_for_removal.remove_task(&task1_id);
  assert!(removed);
  assert_eq!(schedule_for_removal.tasks.len(), 1);
  assert_eq!(schedule_for_removal.tasks[0].id, task2_id); // Verify the correct one remains

  let not_removed = schedule_for_removal.remove_task(&task1_id); // Try removing again
  assert!(!not_removed);
}

#[test]
fn test_schedule_add_task() {
  let mut schedule = Schedule::default();

  let task1_time = Utc.with_ymd_and_hms(2025, 5, 1, 9, 0, 0).unwrap();
  let task1 = ScheduledTask::new(task1_time, "Weather".to_string(), json!({}));
  let task2_time = Utc.with_ymd_and_hms(2025, 5, 1, 17, 30, 0).unwrap();
  let task2 = ScheduledTask::new(
    task2_time,
    "text".to_string(),
    json!({"message": "Hello, world!"}),
  );
  let task3_time = Utc.with_ymd_and_hms(2025, 4, 1, 17, 30, 0).unwrap();
  let task3 = ScheduledTask::new(
    task3_time,
    "text".to_string(),
    json!({"message": "Hello, world!"}),
  );
  let task4_time = Utc.with_ymd_and_hms(2025, 4, 25, 10, 0, 0).unwrap();
  let task4 = ScheduledTask::new(task4_time, "weather".to_string(), json!({}));

  schedule.add_task(task1.clone());
  schedule.add_task(task2.clone());
  schedule.add_task(task3.clone());
  schedule.add_task(task4.clone());

  assert_eq!(schedule.tasks.len(), 4);
  assert_eq!(schedule.tasks[0].id, task3.id);
  assert_eq!(schedule.tasks[1].id, task4.id);
  assert_eq!(schedule.tasks[2].id, task1.id);
  assert_eq!(schedule.tasks[3].id, task2.id);
}

#[test]
fn test_schedule_get_tasks() {
  let mut schedule = Schedule::default();

  let task1_time = Utc.with_ymd_and_hms(2025, 5, 1, 9, 0, 0).unwrap();
  let task1 = ScheduledTask::new(task1_time, "Weather".to_string(), json!({}));
  let task2_time = Utc.with_ymd_and_hms(2025, 5, 1, 17, 30, 0).unwrap();
  let task2 = ScheduledTask::new(
    task2_time,
    "text".to_string(),
    json!({"message": "Hello, world!"}),
  );

  schedule.add_task(task1.clone());
  schedule.add_task(task2.clone());

  let tasks = schedule.get_tasks();
  assert_eq!(tasks.len(), 2);
  assert_eq!(tasks[0].id, task1.id);
  assert_eq!(tasks[1].id, task2.id);
}

#[test]
fn test_schedule_get_task_mut() {
  let mut schedule = Schedule::default();

  let task1_time = Utc.with_ymd_and_hms(2025, 5, 1, 9, 0, 0).unwrap();
  let task1 = ScheduledTask::new(task1_time, "Weather".to_string(), json!({}));
  let task2_time = Utc.with_ymd_and_hms(2025, 5, 1, 17, 30, 0).unwrap();
  let task2 = ScheduledTask::new(
    task2_time,
    "text".to_string(),
    json!({"message": "Hello, world!"}),
  );

  schedule.add_task(task1.clone());
  schedule.add_task(task2.clone());

  let task = schedule.get_task_mut(&task1.id).unwrap();
  assert_eq!(task.id, task1.id);
  task.widget = "text".to_string();
  assert_eq!(task.widget, "text");
}

#[test]
fn test_schedule_is_empty() {
  let schedule = Schedule::default();
  assert!(schedule.is_empty());

  let task_time = Utc.with_ymd_and_hms(2025, 5, 1, 9, 0, 0).unwrap();
  let task = ScheduledTask::new(task_time, "Weather".to_string(), json!({}));
  let mut schedule_with_task = Schedule::default();
  schedule_with_task.add_task(task);
  assert!(!schedule_with_task.is_empty());
  schedule_with_task.clear();
  assert!(schedule_with_task.is_empty());
  assert_eq!(schedule_with_task.tasks.len(), 0);
}

#[test]
fn test_schedule_remove_task() {
  let mut schedule = Schedule::default();

  let task1_time = Utc.with_ymd_and_hms(2025, 5, 1, 9, 0, 0).unwrap();
  let task1 = ScheduledTask::new(task1_time, "Weather".to_string(), json!({}));
  let task2_time = Utc.with_ymd_and_hms(2025, 5, 1, 17, 30, 0).unwrap();
  let task2 = ScheduledTask::new(
    task2_time,
    "text".to_string(),
    json!({"message": "Hello, world!"}),
  );

  schedule.add_task(task1.clone());
  schedule.add_task(task2.clone());

  assert_eq!(schedule.tasks.len(), 2);
  assert_eq!(schedule.tasks[0].id, task1.id);
  assert_eq!(schedule.tasks[1].id, task2.id);

  let removed = schedule.remove_task(&task1.id);
  assert!(removed);
  assert_eq!(schedule.tasks.len(), 1);
  assert_eq!(schedule.tasks[0].id, task2.id);

  let not_removed = schedule.remove_task(&task1.id); // Try removing again
  assert!(!not_removed);
}

#[test]
fn test_schedule_clear() {
  let mut schedule = Schedule::default();

  let task1_time = Utc.with_ymd_and_hms(2025, 5, 1, 9, 0, 0).unwrap();
  let task1 = ScheduledTask::new(task1_time, "Weather".to_string(), json!({}));
  let task2_time = Utc.with_ymd_and_hms(2025, 5, 1, 17, 30, 0).unwrap();
  let task2 = ScheduledTask::new(
    task2_time,
    "text".to_string(),
    json!({"message": "Hello, world!"}),
  );

  schedule.add_task(task1.clone());
  schedule.add_task(task2.clone());

  assert_eq!(schedule.tasks.len(), 2);
  assert_eq!(schedule.tasks[0].id, task1.id);
  assert_eq!(schedule.tasks[1].id, task2.id);

  schedule.clear();
  assert_eq!(schedule.tasks.len(), 0);
  assert!(schedule.is_empty());
}

#[test]
fn test_schedule_error_context() {
  let path = PathBuf::from("/root/cannot_write_here.json");
  let schedule = Schedule::default();
  let result = save_schedule(&schedule, &path);

  assert!(result.is_err());
  let error = result.unwrap_err();

  // Test display formatting first
  let error_msg = format!("{}", error);

  match error {
    VestaboardError::IOError { context, .. } => {
      assert!(context.contains("saving schedule to"));
    },
    _ => panic!("Expected IOError with context"),
  }

  assert!(error_msg.contains("IO Error in saving schedule to"));
}

#[test]
fn test_scheduled_task_new() {
  let time = Utc.with_ymd_and_hms(2025, 5, 4, 18, 30, 0).unwrap();
  let widget = "text".to_string();
  let input = json!({"message": "test message"});

  let task = ScheduledTask::new(time, widget.clone(), input.clone());

  assert_eq!(task.time, time);
  assert_eq!(task.widget, widget);
  assert_eq!(task.input, input);
  assert!(!task.id.is_empty()); // Should have generated an ID
  assert_eq!(task.id.len(), ID_LENGTH); // Should be correct length
}

#[test]
#[serial]
fn test_add_task_to_schedule() {
  use std::fs;
  use std::path::Path;

  // Backup existing schedule file if it exists
  let schedule_path = Path::new(DEFAULT_SCHEDULE_FILE_PATH);
  let backup_path = Path::new("./data/schedule_backup_test.json");
  let had_existing_file = schedule_path.exists();

  if had_existing_file {
    fs::copy(schedule_path, backup_path).expect("Failed to backup existing schedule");
  }

  // Ensure the data directory exists
  if let Some(parent) = schedule_path.parent() {
    fs::create_dir_all(parent).expect("Failed to create data directory");
  }

  // Create empty schedule first
  let empty_schedule = Schedule::default();
  save_schedule(&empty_schedule, &schedule_path.to_path_buf())
    .expect("Failed to save initial schedule");

  // Test the actual add_task_to_schedule function
  let time = Utc.with_ymd_and_hms(2025, 5, 4, 18, 30, 0).unwrap();
  let widget = "text".to_string();
  let input = json!({"message": "test message"});

  let result = add_task_to_schedule(time, widget.clone(), input.clone());
  assert!(result.is_ok(), "add_task_to_schedule should succeed");

  // Verify task was added by loading the schedule
  let loaded_schedule =
    load_schedule(&schedule_path.to_path_buf()).expect("Failed to load schedule after adding task");
  assert_eq!(loaded_schedule.tasks.len(), 1);
  assert_eq!(loaded_schedule.tasks[0].widget, widget);
  assert_eq!(loaded_schedule.tasks[0].input, input);
  assert_eq!(loaded_schedule.tasks[0].time, time);

  // Cleanup: restore original file or remove test file
  if had_existing_file {
    fs::copy(backup_path, schedule_path).expect("Failed to restore original schedule");
    fs::remove_file(backup_path).expect("Failed to remove backup file");
  } else {
    fs::remove_file(schedule_path).ok(); // Remove if we created it
  }
}

#[test]
#[serial]
fn test_remove_task_from_schedule() {
  use std::fs;
  use std::path::Path;

  // Backup existing schedule file if it exists
  let schedule_path = Path::new(DEFAULT_SCHEDULE_FILE_PATH);
  let backup_path = Path::new("./data/schedule_backup_remove_test.json");
  let had_existing_file = schedule_path.exists();

  if had_existing_file {
    fs::copy(schedule_path, backup_path).expect("Failed to backup existing schedule");
  }

  // Ensure the data directory exists
  if let Some(parent) = schedule_path.parent() {
    fs::create_dir_all(parent).expect("Failed to create data directory");
  }

  // Create schedule with a task using add_task_to_schedule
  let time = Utc.with_ymd_and_hms(2025, 5, 4, 18, 30, 0).unwrap();
  let widget = "text".to_string();
  let input = json!({"message": "test message"});

  // First create empty schedule
  let empty_schedule = Schedule::default();
  save_schedule(&empty_schedule, &schedule_path.to_path_buf())
    .expect("Failed to save initial schedule");

  // Add a task using the global function
  add_task_to_schedule(time, widget, input).expect("Failed to add task");

  // Get the task ID
  let loaded_schedule =
    load_schedule(&schedule_path.to_path_buf()).expect("Failed to load schedule");
  assert_eq!(loaded_schedule.tasks.len(), 1);
  let task_id = loaded_schedule.tasks[0].id.clone();

  // Test remove_task_from_schedule functionality
  let result = remove_task_from_schedule(&task_id);
  assert!(result.is_ok(), "remove_task_from_schedule should succeed");

  let removed = result.unwrap();
  assert!(removed, "Task should have been removed");

  // Verify task was removed
  let final_schedule =
    load_schedule(&schedule_path.to_path_buf()).expect("Failed to load final schedule");
  assert_eq!(final_schedule.tasks.len(), 0);

  let result2 = remove_task_from_schedule(&task_id);
  assert!(result2.is_ok(), "Removing non-existent task should succeed");
  let removed2 = result2.unwrap();
  assert!(!removed2, "Removing non-existent task should return false");

  // Cleanup: restore original file or remove test file
  if had_existing_file {
    fs::copy(backup_path, schedule_path).expect("Failed to restore original schedule");
    fs::remove_file(backup_path).expect("Failed to remove backup file");
  } else {
    fs::remove_file(schedule_path).ok(); // Remove if we created it
  }
}

#[test]
#[serial]
fn test_clear_schedule() {
  use std::fs;
  use std::path::Path;

  // Backup existing schedule file if it exists
  let schedule_path = Path::new(DEFAULT_SCHEDULE_FILE_PATH);
  let backup_path = Path::new("./data/schedule_backup_clear_test.json");
  let had_existing_file = schedule_path.exists();

  if had_existing_file {
    fs::copy(schedule_path, backup_path).expect("Failed to backup existing schedule");
  }

  // Ensure the data directory exists
  if let Some(parent) = schedule_path.parent() {
    fs::create_dir_all(parent).expect("Failed to create data directory");
  }

  // Create schedule with multiple tasks using add_task_to_schedule
  let time = Utc.with_ymd_and_hms(2025, 5, 4, 18, 30, 0).unwrap();

  // First create empty schedule
  let empty_schedule = Schedule::default();
  save_schedule(&empty_schedule, &schedule_path.to_path_buf())
    .expect("Failed to save initial schedule");

  // Add multiple tasks
  add_task_to_schedule(time, "text".to_string(), json!({"message": "test1"}))
    .expect("Failed to add task 1");
  add_task_to_schedule(time, "weather".to_string(), json!({})).expect("Failed to add task 2");
  add_task_to_schedule(time, "sat-word".to_string(), json!({})).expect("Failed to add task 3");

  // Verify tasks were added
  let loaded_schedule =
    load_schedule(&schedule_path.to_path_buf()).expect("Failed to load schedule");
  assert_eq!(loaded_schedule.tasks.len(), 3);

  // Test clear_schedule functionality
  let result = clear_schedule();
  assert!(result.is_ok(), "clear_schedule should succeed");

  // Verify all tasks were cleared
  let final_schedule =
    load_schedule(&schedule_path.to_path_buf()).expect("Failed to load final schedule");
  assert_eq!(final_schedule.tasks.len(), 0);
  assert!(final_schedule.is_empty());

  // Cleanup: restore original file or remove test file
  if had_existing_file {
    fs::copy(backup_path, schedule_path).expect("Failed to restore original schedule");
    fs::remove_file(backup_path).expect("Failed to remove backup file");
  } else {
    fs::remove_file(schedule_path).ok(); // Remove if we created it
  }
}

#[test]
#[serial]
fn test_list_schedule() {
  use std::fs;
  use std::path::Path;

  // Backup existing schedule file if it exists
  let schedule_path = Path::new(DEFAULT_SCHEDULE_FILE_PATH);
  let backup_path = Path::new("./data/schedule_backup_list_test.json");
  let had_existing_file = schedule_path.exists();

  if had_existing_file {
    fs::copy(schedule_path, backup_path).expect("Failed to backup existing schedule");
  }

  // Ensure the data directory exists
  if let Some(parent) = schedule_path.parent() {
    fs::create_dir_all(parent).expect("Failed to create data directory");
  }

  // Create schedule with tasks using add_task_to_schedule
  let time1 = Utc.with_ymd_and_hms(2025, 5, 4, 18, 30, 0).unwrap();
  let time2 = Utc.with_ymd_and_hms(2025, 5, 5, 19, 45, 0).unwrap();

  // First create empty schedule
  let empty_schedule = Schedule::default();
  save_schedule(&empty_schedule, &schedule_path.to_path_buf())
    .expect("Failed to save initial schedule");

  // Add tasks
  add_task_to_schedule(time1, "text".to_string(), json!({"message": "hello"}))
    .expect("Failed to add task 1");
  add_task_to_schedule(time2, "weather".to_string(), json!({})).expect("Failed to add task 2");

  // Test that list_schedule can run without panicking
  // (We can't easily test the printed output, but we can test that it doesn't crash)
  let result = list_schedule();
  assert!(result.is_ok(), "list_schedule should succeed");

  // Verify the underlying schedule is correct
  let loaded_schedule =
    load_schedule(&schedule_path.to_path_buf()).expect("Failed to load schedule for verification");
  assert_eq!(loaded_schedule.tasks.len(), 2);

  // Verify the tasks are properly formatted for display
  for task in &loaded_schedule.tasks {
    assert!(!task.id.is_empty());
    assert!(!task.widget.is_empty());
    // Time should be valid
    assert!(task.time > Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap());
  }

  // Cleanup: restore original file or remove test file
  if had_existing_file {
    fs::copy(backup_path, schedule_path).expect("Failed to restore original schedule");
    fs::remove_file(backup_path).expect("Failed to remove backup file");
  } else {
    fs::remove_file(schedule_path).ok(); // Remove if we created it
  }
}

#[tokio::test]
async fn test_print_schedule() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
  let temp_path = temp_dir.path().join("test_schedule.json");

  // Create schedule with tasks that won't fail
  let time = Utc.with_ymd_and_hms(2025, 5, 4, 18, 30, 0).unwrap();
  let mut schedule = Schedule::default();

  // Add a text task with valid input
  schedule.add_task(ScheduledTask::new(
    time,
    "text".to_string(),
    json!("hello world"), // Valid text input
  ));

  save_schedule(&schedule, &temp_path).expect("Failed to save schedule");

  // Test that print_schedule can process the schedule without panicking
  // We can't easily test the printed output, but we can ensure it doesn't crash
  let loaded_schedule = load_schedule(&temp_path).expect("Failed to load schedule");
  assert_eq!(loaded_schedule.tasks.len(), 1);

  // Test the widget processing logic manually (similar to what print_schedule does)
  for task in &loaded_schedule.tasks {
    match task.widget.as_str() {
      "text" => {
        let text_input = task.input.as_str().unwrap_or("default text");
        let _result = get_text(text_input); // Should not panic
      },
      "weather" => {
        // Weather widget test would require network, so we'll skip actual execution
        assert_eq!(task.widget, "weather");
      },
      "sat-word" => {
        // SAT word test would require file access, so we'll skip actual execution
        assert_eq!(task.widget, "sat-word");
      },
      _ => {
        // Unknown widget should be handled gracefully
        assert!(false, "Unknown widget type: {}", task.widget);
      },
    }
  }
}

// ScheduleMonitor Tests

#[cfg(test)]
#[test]
fn schedule_monitor_new_test() {
  let temp_file = NamedTempFile::new().expect("Failed to create temp file");
  let path = temp_file.path();

  let monitor = ScheduleMonitor::new(path);
  assert_eq!(monitor.get_schedule_file_path(), path);
  assert_eq!(monitor.get_current_schedule().tasks.len(), 0);
}

#[cfg(test)]
#[test]
fn schedule_monitor_initialize_with_existing_file_test() {
  let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
  let path = temp_file.path().to_path_buf();

  // Create a schedule file with one task
  let task_time = Utc.with_ymd_and_hms(2025, 5, 4, 18, 30, 0).unwrap();
  let schedule = Schedule {
    tasks: vec![ScheduledTask {
      id: "test1".to_string(),
      time: task_time,
      widget: "text".to_string(),
      input: json!("test message"),
    }],
  };

  // Write the schedule to the temp file
  write!(temp_file, "{}", serde_json::to_string_pretty(&schedule).unwrap()).unwrap();
  temp_file.flush().unwrap();

  // Initialize the monitor
  let mut monitor = ScheduleMonitor::new(&path);
  let result = monitor.initialize();

  assert!(result.is_ok());
  assert_eq!(monitor.get_current_schedule().tasks.len(), 1);
  assert_eq!(monitor.get_current_schedule().tasks[0].id, "test1");
}

#[cfg(test)]
#[test]
fn schedule_monitor_initialize_with_nonexistent_file_test() {
  let temp_file = NamedTempFile::new().expect("Failed to create temp file");
  let path = temp_file.path().to_path_buf();

  // Delete the temp file to test non-existent file behavior
  drop(temp_file);

  let mut monitor = ScheduleMonitor::new(&path);
  let result = monitor.initialize();

  // Should succeed and create an empty schedule
  assert!(result.is_ok());
  assert_eq!(monitor.get_current_schedule().tasks.len(), 0);
}

#[cfg(test)]
#[test]
fn schedule_monitor_check_for_updates_test() {
  let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
  let path = temp_file.path().to_path_buf();

  // Create initial schedule
  let schedule = Schedule::default();
  write!(temp_file, "{}", serde_json::to_string_pretty(&schedule).unwrap()).unwrap();
  temp_file.flush().unwrap();

  let mut monitor = ScheduleMonitor::new(&path);
  monitor.initialize().expect("Failed to initialize monitor");

  // First check should return false (no changes since initialization)
  let result = monitor.check_for_updates();
  assert!(result.is_ok());
  assert!(!result.unwrap());

  // Modify the file
  std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamp
  write!(temp_file, "{}", serde_json::to_string_pretty(&schedule).unwrap()).unwrap();
  temp_file.flush().unwrap();

  // Second check should detect changes
  let result = monitor.check_for_updates();
  assert!(result.is_ok());
  assert!(result.unwrap());
}

#[cfg(test)]
#[test]
fn schedule_monitor_reload_if_modified_test() {
  let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
  let path = temp_file.path().to_path_buf();

  // Create initial empty schedule
  let schedule = Schedule::default();
  write!(temp_file, "{}", serde_json::to_string_pretty(&schedule).unwrap()).unwrap();
  temp_file.flush().unwrap();

  let mut monitor = ScheduleMonitor::new(&path);
  monitor.initialize().expect("Failed to initialize monitor");

  assert_eq!(monitor.get_current_schedule().tasks.len(), 0);

  // Update the file with a new task
  std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamp
  let task_time = Utc.with_ymd_and_hms(2025, 5, 4, 18, 30, 0).unwrap();
  let new_schedule = Schedule {
    tasks: vec![ScheduledTask {
      id: "new1".to_string(),
      time: task_time,
      widget: "text".to_string(),
      input: json!("new message"),
    }],
  };

  temp_file.seek(std::io::SeekFrom::Start(0)).unwrap();
  temp_file.as_file_mut().set_len(0).unwrap();
  write!(temp_file, "{}", serde_json::to_string_pretty(&new_schedule).unwrap()).unwrap();
  temp_file.flush().unwrap();

  // Check if modified and reload
  let result = monitor.reload_if_modified();
  assert!(result.is_ok());
  assert_eq!(result.unwrap(), true); // Should indicate file was modified

  // Verify the new schedule was loaded
  assert_eq!(monitor.get_current_schedule().tasks.len(), 1);
  assert_eq!(monitor.get_current_schedule().tasks[0].id, "new1");
}

#[cfg(test)]
#[test]
fn schedule_monitor_handles_file_not_found_test() {
  let temp_file = NamedTempFile::new().expect("Failed to create temp file");
  let path = temp_file.path().to_path_buf();

  // Delete the file
  drop(temp_file);

  let mut monitor = ScheduleMonitor::new(&path);

  // check_for_updates should handle missing file gracefully
  let result = monitor.check_for_updates();
  assert!(result.is_ok());
}
