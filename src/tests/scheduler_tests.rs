#[path = "../scheduler.rs"]
mod scheduler;

use crate::scheduler::{
    Schedule,
    ScheduledTask,
    CUSTOM_ALPHABET,
    ID_LENGTH,
    load_schedule,
    save_schedule,
};
use crate::errors::VestaboardError;
use chrono::{ DateTime, TimeZone, Utc };
use tempfile::NamedTempFile;
use serde_json::json;
use std::io::{ Write, Seek };

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
    use tempfile::NamedTempFile;
    use std::io::Read;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path().to_path_buf();

    // Test saving an empty schedule
    let empty_schedule = Schedule::default();
    let result = save_schedule(&empty_schedule, &path);
    assert!(result.is_ok());

    // Verify the saved content is an empty schedule
    let mut file_content = String::new();
    temp_file.read_to_string(&mut file_content).expect("Failed to read from temp file");
    assert_eq!(file_content, "{\n  \"tasks\": []\n}");

    // Test saving a schedule with tasks
    let task1_time = Utc.with_ymd_and_hms(2025, 5, 1, 9, 0, 0).unwrap();
    let task1 = ScheduledTask::new(task1_time, "Weather".to_string(), json!({}));
    let task2_time = Utc.with_ymd_and_hms(2025, 5, 1, 17, 30, 0).unwrap();
    let task2 = ScheduledTask::new(
        task2_time,
        "text".to_string(),
        json!({"message": "Hello, world!"})
    );

    let mut schedule = Schedule::default();
    schedule.add_task(task1);
    schedule.add_task(task2);

    let result = save_schedule(&schedule, &path);
    assert!(result.is_ok());

    // Verify the saved content matches the schedule with tasks
    temp_file.seek(std::io::SeekFrom::Start(0)).expect("Failed to seek to start of file");
    file_content.clear();
    temp_file.read_to_string(&mut file_content).expect("Failed to read from temp file");

    let expected_json = serde_json::to_string_pretty(&schedule).unwrap();
    assert_eq!(file_content, expected_json);
}

#[test]
fn test_load_schedule_success() {
    let (json_content, expected_time, expected_id) = create_valid_json_content();
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    write!(temp_file, "{}", json_content).expect("Failed to write to temp file");
    temp_file.as_file_mut().flush().expect("Failed to flush temp file");
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
    write!(temp_file, "{{invalid json data, missing quotes}}").expect(
        "Failed to write invalid JSON"
    );
    temp_file.flush().expect("Failed to flush");

    let path = temp_file.path().to_path_buf();
    let result = load_schedule(&path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    if let VestaboardError::JsonError(_) = err {
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
        json!({"message": "Hello, world!"})
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
    assert_eq!(deserialized_schedule.tasks[1].input, json!({"message": "Hello, world!"}));

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
        json!({"message": "Hello, world!"})
    );
    let task3_time = Utc.with_ymd_and_hms(2025, 4, 1, 17, 30, 0).unwrap();
    let task3 = ScheduledTask::new(
        task3_time,
        "text".to_string(),
        json!({"message": "Hello, world!"})
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
        json!({"message": "Hello, world!"})
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
        json!({"message": "Hello, world!"})
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
