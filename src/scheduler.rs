use std::{ fs, path::PathBuf };

use chrono::{ DateTime, Utc, Local };
use nanoid::nanoid;
use serde::{ Deserialize, Serialize };
use serde_json::Value;

use crate::datetime::datetime_to_local;
use crate::widgets::{
    text::get_text,
    text::get_text_from_file,
    weather::get_weather,
    sat_words::get_sat_word,
};
use crate::cli_display::print_message;
use crate::{ errors::VestaboardError };

pub const SCHEDULE_FILE_PATH: &str = "./data/schedule.json";

pub const CUSTOM_ALPHABET: &[char] = &[
    'a',
    'b',
    'c',
    'd',
    'e',
    'f',
    'g',
    'h',
    'i',
    'j',
    'k',
    'l',
    'm',
    'n',
    'o',
    'p',
    'q',
    'r',
    's',
    't',
    'u',
    'v',
    'w',
    'x',
    'y',
    'z',
    '0',
    '1',
    '2',
    '3',
    '4',
    '5',
    '6',
    '7',
    '8',
    '9',
];
pub const ID_LENGTH: usize = 4;

fn generate_task_id() -> String {
    nanoid!(ID_LENGTH, CUSTOM_ALPHABET)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    #[serde(default = "generate_task_id")]
    pub id: String,
    pub time: DateTime<Utc>,
    pub widget: String,
    pub input: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Schedule {
    #[serde(default)]
    pub tasks: Vec<ScheduledTask>,
}

impl ScheduledTask {
    pub fn new(time: DateTime<Utc>, widget: String, input: Value) -> Self {
        ScheduledTask {
            id: generate_task_id(),
            time,
            widget,
            input,
        }
    }
}

impl Schedule {
    pub fn add_task(&mut self, task: ScheduledTask) {
        let position = self.tasks
            .iter()
            .position(|t| t.time > task.time)
            .unwrap_or(self.tasks.len());
        self.tasks.insert(position, task);
    }
    pub fn remove_task(&mut self, id: &str) -> bool {
        let initial_len = self.tasks.len();
        self.tasks.retain(|task| task.id != id);
        self.tasks.len() < initial_len
    }
    pub fn get_tasks(&self) -> &[ScheduledTask] {
        &self.tasks
    }
    pub fn get_task(&self, id: &str) -> Option<&ScheduledTask> {
        self.tasks.iter().find(|task| task.id == id)
    }
    pub fn get_task_mut(&mut self, id: &str) -> Option<&mut ScheduledTask> {
        self.tasks.iter_mut().find(|task| task.id == id)
    }
    pub fn clear(&mut self) {
        self.tasks.clear();
    }
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

pub fn save_schedule(schedule: &Schedule, path: &PathBuf) -> Result<(), VestaboardError> {
    // Save the schedule to the file
    // handle errors appropriately
    println!("Saving schedule to {}", path.display());
    match fs::write(path, serde_json::to_string_pretty(schedule).unwrap()) {
        Ok(_) => {
            println!("Schedule saved successfully.");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error saving schedule: {}", e);
            Err(VestaboardError::io_error(e, "saving schedule to file"))
        }
    }
}

pub fn load_schedule(path: &PathBuf) -> Result<Schedule, VestaboardError> {
    println!("Loading schedule from {}", path.display());
    match fs::read_to_string(&path) {
        Ok(content) => {
            if content.trim().is_empty() {
                println!("Schedule is empty. Creating a new schedule.");
                Ok(Schedule::default())
            } else {
                match serde_json::from_str::<Schedule>(&content) {
                    Ok(mut schedule) => {
                        println!(
                            "Successfully loaded {} tasks from schedule {}.",
                            schedule.tasks.len(),
                            path.display()
                        );
                        schedule.tasks.sort_by_key(|task| task.time);
                        Ok(schedule)
                    }
                    Err(e) => {
                        println!("Failed to parse schedule from {} : {}", path.display(), e);
                        Err(VestaboardError::json_error(e, "parsing schedule JSON"))
                    }
                }
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("Schedule file not found. Creating a new schedule.");
            let schedule = Schedule::default();
            match save_schedule(&schedule, path) {
                Ok(_) => {
                    println!("New schedule created and saved.");
                }
                Err(e) => {
                    eprintln!("Error saving new schedule: {:?}", e);
                }
            }
            Ok(schedule)
        }
        Err(e) => {
            eprintln!("Error reading schedule file {} : {}", path.display(), e);
            Err(
                VestaboardError::schedule_error(
                    "load_schedule",
                    &format!("Failed to read schedule file: {}", e)
                )
            )
        }
    }
}

pub fn add_task_to_schedule(
    time: DateTime<Utc>,
    widget: String,
    input: Value
) -> Result<(), VestaboardError> {
    let schedule_path = PathBuf::from(SCHEDULE_FILE_PATH);
    let mut schedule = load_schedule(&schedule_path)?;

    let task = ScheduledTask::new(time, widget, input);
    schedule.add_task(task);
    save_schedule(&schedule, &schedule_path)
}

pub fn remove_task_from_schedule(id: &str) -> Result<(), VestaboardError> {
    let schedule_path = PathBuf::from(SCHEDULE_FILE_PATH);
    let mut schedule = load_schedule(&schedule_path)?;
    if schedule.get_task(id).is_none() {
        println!("Task with ID {} not found.", id);
        return Ok(());
    }
    if schedule.remove_task(id) {
        save_schedule(&schedule, &schedule_path)?;
        println!("Task with ID {} removed successfully.", id);
    }
    Ok(())
}

pub fn clear_schedule() -> Result<(), VestaboardError> {
    let schedule_path = PathBuf::from(SCHEDULE_FILE_PATH);
    let mut schedule = load_schedule(&schedule_path)?;
    println!("Clearing schedule...");
    schedule.clear();
    save_schedule(&schedule, &schedule_path)
}

pub fn list_schedule() -> Result<(), VestaboardError> {
    let schedule_path = PathBuf::from(SCHEDULE_FILE_PATH);
    let schedule = load_schedule(&schedule_path)?;

    println!("\nScheduled Tasks:");
    println!("{:<6} | {:<22} | {:<15} | {}", "ID", "Time (Local)", "Widget", "Input");
    println!("{:-<80}", ""); // Separator line
    if schedule.tasks.is_empty() {
        println!("");
        return Ok(());
    }
    for task in schedule.tasks {
        let local_time = task.time.with_timezone(&Local::now().timezone());
        let formatted_time = local_time.format("%Y.%m.%d %I:%M %p").to_string();
        let input_str = serde_json
            ::to_string(&task.input)
            .unwrap_or_else(|_| "Invalid JSON".to_string());
        println!("{:<6} | {:<22} | {:<15} | {}", task.id, formatted_time, task.widget, input_str);
    }
    println!("{:-<80}", ""); // Footer separator line
    Ok(())
}

pub async fn print_schedule() {
    let schedule_path = PathBuf::from(SCHEDULE_FILE_PATH);
    let schedule = load_schedule(&schedule_path).unwrap_or_else(|_| Schedule::default());
    for task in schedule.tasks.iter() {
        let message_result = match task.widget.as_str() {
            "text" => { get_text(task.input.as_str().unwrap_or("")) }
            "file" => { get_text_from_file(PathBuf::from(task.input.as_str().unwrap_or(""))) }
            "weather" => { get_weather().await }
            "sat-word" => { get_sat_word() }
            _ => {
                println!("Unknown widget type: {}", task.widget);
                Err(VestaboardError::widget_error(&task.widget, "Unknown widget type"))
            }
        };

        let message = match message_result {
            Ok(msg) => msg,
            Err(e) => {
                use crate::widgets::widget_utils::error_to_display_message;
                error_to_display_message(&e)
            }
        };

        print_message(message, &datetime_to_local(task.time));
        println!("");
    }
}
