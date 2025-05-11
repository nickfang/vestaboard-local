use std::path::PathBuf;

use chrono::{ DateTime, Utc, Local };
use nanoid::nanoid;
use serde::{ Deserialize, Serialize };
use serde_json::Value;

use crate::{ daemon::{ load_schedule, SCHEDULE_FILE_PATH }, errors::VestaboardError };

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

pub fn print_scheduled_tasks() -> Result<(), VestaboardError> {
    let schedule_path = PathBuf::from(SCHEDULE_FILE_PATH);
    let schedule = load_schedule(&schedule_path)?;

    if schedule.tasks.is_empty() {
        println!("No tasks scheduled.");
        return Ok(());
    }

    println!("\nScheduled Tasks:");
    println!("{:<6} | {:<22} | {:<15} | {}", "ID", "Time (Local)", "Widget", "Input");
    println!("{:-<80}", ""); // Separator line
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
