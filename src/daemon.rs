use crate::datetime::is_or_before;
use crate::scheduler::{ load_schedule, Schedule, ScheduledTask, SCHEDULE_FILE_PATH };
use crate::errors::VestaboardError;
use crate::widgets::text::{ get_text, get_text_from_file };
use crate::widgets::weather::get_weather;
use crate::widgets::sat_words::get_sat_word;
use crate::api_broker::{ display_message, validate_message_content };
use crate::widgets::widget_utils::error_to_display_message;

use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{ AtomicBool, Ordering };
use std::thread;
use std::time::{ Duration, SystemTime };

#[allow(dead_code)] // Not dead code, but the compiler doesn't know that.
static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);
#[allow(dead_code)] // Not dead code, but the compiler doesn't know that.
const CHECK_INTERVAL_SECONDS: u64 = 3;

pub fn get_file_mod_time(path: &PathBuf) -> Result<SystemTime, VestaboardError> {
    // Get the last modified time of the file
    // If the file doesn't exist, return an error
    // handle errors appropriately
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .map_err(|e| {
            eprintln!("Error getting mod time for {}: {}", path.display(), e);
            VestaboardError::io_error(e, &format!("getting mod time for {}", path.display()))
        })
}

pub async fn execute_task(task: &ScheduledTask) -> Result<(), VestaboardError> {
    // Find widget based on task
    // Execute widget with task.widget_input
    // Send the message to the Vestaboard
    // handle errors appropriately
    println!("Executing task: {:?}", task);

    let message_result = match task.widget.as_str() {
        "text" => {
            // Execute text widget
            println!("Executing Text widget with input: {:?}", task.input);
            get_text(task.input.as_str().unwrap_or(""))
        }
        "file" => {
            // Execute file widget
            println!("Executing File widget with input: {:?}", task.input);
            get_text_from_file(PathBuf::from(task.input.as_str().unwrap_or("")))
        }
        "weather" => {
            // Execute weather widget
            println!("Executing Weather widget");
            get_weather().await
        }
        "sat-word" => {
            // Execute SAT word widget
            println!("Executing SAT Word widget");
            get_sat_word()
        }
        _ => {
            return Err(
                VestaboardError::widget_error(
                    &task.widget,
                    &format!("Unknown widget type: {}", task.widget)
                )
            );
        }
    };

    let message = match message_result {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("Widget error: {}", e);
            error_to_display_message(&e)
        }
    };

    // Validate message content before sending
    if let Err(validation_error) = validate_message_content(&message) {
        eprintln!("Validation error: {}", validation_error);
        display_message(error_to_display_message(&VestaboardError::other(&validation_error))).await;
        return Ok(());
    }

    display_message(message).await;
    Ok(())
}
// Err(VestaboardError::Other("execute_task() not implemented".to_string()));

pub async fn run_daemon() -> Result<(), VestaboardError> {
    println!("Starting daemon...");
    println!("Press Ctrl+C to stop the daemon.");

    // handle ctrl+c
    ctrlc
        ::set_handler(move || {
            println!("\nCtrl+C received, shutting down...");
            SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

    let schedule_path = PathBuf::from(SCHEDULE_FILE_PATH);
    let check_interval = Duration::from_secs(CHECK_INTERVAL_SECONDS);

    let mut current_schedule = load_schedule(&schedule_path).unwrap_or_else(|e| {
        // schedule not found is handled in load_schedule
        eprintln!("Error loading initial schedule: {:?}.", e);
        Schedule::default()
    });
    println!("Initial schedule loaded with {} tasks.", current_schedule.tasks.len());

    let mut last_mod_time = get_file_mod_time(&schedule_path).unwrap_or(SystemTime::UNIX_EPOCH);
    let mut executed_task_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    println!("Daemon started. Monitoring schedule...");

    loop {
        if SHUTDOWN_FLAG.load(Ordering::SeqCst) {
            println!("Daemon shutting down...");
            break;
        }

        // Reload schedule if the file has been modified
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
            if is_or_before(task.time, now) && !executed_task_ids.contains(&task.id) {
                tasks_to_execute.push(task.clone());
            }
        }

        if let Some(task) = tasks_to_execute.last() {
            match execute_task(task).await {
                Ok(_) => {
                    for task in &tasks_to_execute {
                        executed_task_ids.insert(task.id.clone());
                    }
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
