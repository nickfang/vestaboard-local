use crate::scheduler::{ Schedule, ScheduledTask };
use crate::errors::VestaboardError::{ self, IOError, ScheduleError, JsonError, WidgetError };
use chrono::{ DateTime, Utc };
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{ AtomicBool, Ordering };
use std::thread;
use std::time::{ Duration, SystemTime };

static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);

const SCHEDULE_FILE_PATH: &str = "schedule.json";
const CHECK_INTERVAL_SECONDS: u64 = 3;

pub fn load_schedule(path: &PathBuf) -> Result<Schedule, VestaboardError> {
    // Check if the schedule file exists
    // If it doesn't, create an empty schedule
    // If it does, load the schedule from the file
    // handle errors appropriately
    println!("Loading schedule from {}", SCHEDULE_FILE_PATH);
    match fs::read_to_string(&path) {
        Ok(content) => {
            if content.trim().is_empty() {
                println!("Schedule is empty. Creating a new schedule.");
                Ok(Schedule::default())
            } else {
                match serde_json::from_str::<Schedule>(&content) {
                    Ok(schedule) => {
                        println!(
                            "Successfully loaded {} tasks from schedule {}.",
                            schedule.tasks.len(),
                            path.display()
                        );
                        Ok(schedule)
                    }
                    Err(e) => {
                        println!("Failed to parse schedule from {} : {}", path.display(), e);
                        Err(JsonError(e))
                    }
                }
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("Schedule file not found. Creating a new schedule.");
            Ok(Schedule::default())
        }
        Err(e) => {
            eprintln!("Error reading schedule file {} : {}", path.display(), e);
            Err(ScheduleError(format!("Failed to parse schedule: {}", e)))
        }
    }
}

pub fn get_file_mod_time(path: &PathBuf) -> Result<SystemTime, VestaboardError> {
    // Get the last modified time of the file
    // If the file doesn't exist, return an error
    // handle errors appropriately
    println!("Getting file modification time for {}", path.display());
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .map_err(|e| {
            eprintln!("Error getting mod time for {}: {}", path.display(), e);
            IOError(e)
        })
}

pub fn execute_task(task: &ScheduledTask) -> Result<(), VestaboardError> {
    // Find widget based on task
    // Execute widget with task.widget_input
    // Send the message to the Vestaboard
    // handle errors appropriately
    println!("Executing task: {:?}", task);
    return Err(VestaboardError::Other("execute_task() not implemented".to_string()));
}

pub fn run_daemon() -> Result<(), VestaboardError> {
    println!("Starting daemon...");
    println!("Press Ctrl+C to stop the daemon.");

    ctrlc
        ::set_handler(move || {
            println!("\nCtrl+C received, shutting down...");
            SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
    let schedule_path = PathBuf::from(SCHEDULE_FILE_PATH);
    let check_interval = Duration::from_secs(CHECK_INTERVAL_SECONDS);

    let mut current_schedule = load_schedule(&schedule_path).unwrap_or_else(|e| {
        eprintln!("Error loading initial schedule: {:?}.  Starting with empty schedule.", e);
        Schedule::default()
    });
    let mut last_mod_time = get_file_mod_time(&schedule_path).unwrap_or(SystemTime::UNIX_EPOCH);

    let mut executed_task_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    println!("Daemon started. Monitoring schedule...");

    loop {
        if SHUTDOWN_FLAG.load(Ordering::SeqCst) {
            println!("Daemon shutting down...");
            break;
        }

        match get_file_mod_time(&schedule_path) {
            Ok(current_mod_time) => {
                if current_mod_time > last_mod_time {
                    println!("Schedule file modified. Reloading schedule...");
                    match load_schedule(&schedule_path) {
                        Ok(new_schedule) => {
                            current_schedule = new_schedule;
                            last_mod_time = current_mod_time;
                            // executed_task_ids.clear();
                            println!("Successfully reloaded schedule.");
                        }
                        Err(e) => {
                            eprintln!("Error reloading schedule: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error getting file modification time: {:?}", e);
            }
        }
        let now = Utc::now();
        let mut tasks_to_execute = Vec::new();

        for task in &current_schedule.tasks {
            if task.time <= now && !executed_task_ids.contains(&task.id) {
                tasks_to_execute.push(task.clone());
            }
        }

        for task in tasks_to_execute {
            println!("Executing task: {:?}", task);
            match execute_task(&task) {
                Ok(_) => {
                    executed_task_ids.insert(task.id.clone());
                }
                Err(e) => {
                    eprintln!("Error executing task: {:?}", e);
                }
            }
        }
        thread::sleep(check_interval);
    }
    println!("Shutdown complete.");
    Ok(())
}
