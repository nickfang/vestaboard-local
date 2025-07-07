mod api;
mod api_broker;
mod cli_display;
mod cli_setup;
mod config;
mod daemon;
mod datetime;
mod errors;
mod logging;
mod scheduler;
mod widgets;

use api_broker::{display_message, validate_message_content};
use clap::Parser;
use cli_display::print_message;
use cli_setup::{Cli, Command, ScheduleArgs, WidgetCommand};
use daemon::run_daemon;
use datetime::datetime_to_utc;
use errors::VestaboardError;
use scheduler::{
  add_task_to_schedule, clear_schedule, list_schedule, print_schedule, remove_task_from_schedule,
};
use serde_json::json;
use widgets::jokes::get_joke;
use widgets::sat_words::get_sat_word;
use widgets::text::{get_text, get_text_from_file};
use widgets::weather::get_weather;

/// Processes a widget command and validates the resulting message
/// This ensures all messages are validated before any output method
async fn process_and_validate_widget(
  widget_command: &WidgetCommand,
) -> Result<Vec<String>, VestaboardError> {
  let start_time = std::time::Instant::now();
  let widget_name = match widget_command {
    WidgetCommand::Text(_) => "text",
    WidgetCommand::File(_) => "file",
    WidgetCommand::Weather => "weather",
    WidgetCommand::Jokes => "jokes",
    WidgetCommand::SATWord => "sat-word",
    WidgetCommand::Clear => "clear",
  };

  log_widget_start!(widget_name, widget_command);

  let message_result = match widget_command {
    WidgetCommand::Text(args) => get_text(&args.message),
    WidgetCommand::File(args) => get_text_from_file(args.name.clone()),
    WidgetCommand::Weather => get_weather().await,
    WidgetCommand::Jokes => get_joke(),
    WidgetCommand::SATWord => get_sat_word(),
    WidgetCommand::Clear => Ok(vec![String::from("")]), // Clear command
  };

  let duration = start_time.elapsed();

  let message = match message_result {
    Ok(msg) => {
      log_widget_success!(widget_name, duration);
      msg
    },
    Err(e) => {
      log_widget_error!(widget_name, e, duration);
      return Err(e);
    },
  };

  // Single validation point for all messages
  if let Err(validation_error) = validate_message_content(&message) {
    log::error!(
      "Message validation failed for widget '{}': {}",
      widget_name,
      validation_error
    );
    return Err(VestaboardError::other(&validation_error));
  }

  log::debug!(
    "Widget '{}' validation successful, message length: {} lines",
    widget_name,
    message.len()
  );
  Ok(message)
}

#[tokio::main]
async fn main() {
  // Initialize logging first
  if let Err(e) = logging::init_logging() {
    eprintln!("Failed to initialize logging: {}", e);
    // Continue without logging rather than failing completely
  }

  log::info!("Vestaboard Local starting up");

  let cli = Cli::parse();
  let mut test_mode = false;

  match cli.command {
    Command::Send(send_args) => {
      log::info!(
        "Processing send command with dry_run: {}",
        send_args.dry_run
      );

      if send_args.dry_run {
        test_mode = true;
        log::debug!("Running in test mode (dry run)");
      }

      // Handle clear command separately since it doesn't need validation
      if matches!(send_args.widget_command, WidgetCommand::Clear) {
        log::info!("Executing clear command");
        if test_mode {
          print_message(vec![String::from("")], "");
        } else {
          if let Err(e) = api::clear_board().await {
            log::error!("Failed to clear board: {}", e);
            eprintln!("Error clearing board: {}", e);
            std::process::exit(1);
          }
          log::info!("Board cleared successfully");
        }
        return;
      }

      // Process and validate the widget message
      let message = match process_and_validate_widget(&send_args.widget_command).await {
        Ok(msg) => {
          log::debug!("Widget processing completed successfully");
          msg
        },
        Err(e) => {
          log::error!("Widget processing failed: {}", e);
          eprintln!("Widget error: {}", e);
          // Convert VestaboardError directly to display message
          use crate::widgets::widget_utils::error_to_display_message;
          error_to_display_message(&e)
        },
      };

      if test_mode {
        log::info!("Displaying message in test mode");
        print_message(message, "");
        return;
      }

      log::info!("Sending message to Vestaboard");
      display_message(message).await;
    },
    Command::Schedule { action } => {
      log::info!("Processing schedule command");
      match action {
        ScheduleArgs::Add {
          time,
          widget,
          input,
        } => {
          log::info!(
            "Adding scheduled task - time: {}, widget: {}, input: {:?}",
            time,
            widget,
            input
          );
          println!("Scheduling task...");
          let datetime_utc = match datetime_to_utc(&time) {
            Ok(dt) => {
              log::debug!("Parsed datetime: {}", dt);
              dt
            },
            Err(e) => {
              log::error!("Invalid datetime format '{}': {}", time, e);
              println!("datetime: {}", time);
              eprintln!("Error invalid datetime format: {}", e);
              return;
            },
          };

          // Convert the schedule widget args to a WidgetCommand for validation
          let widget_command = match widget.to_lowercase().as_str() {
            "weather" => WidgetCommand::Weather,
            "sat-word" => WidgetCommand::SATWord,
            "jokes" => WidgetCommand::Jokes,
            "clear" => WidgetCommand::Clear,
            "text" => {
              if !input.is_empty() {
                WidgetCommand::Text(cli_setup::TextArgs {
                  message: input.join(" "),
                })
              } else {
                eprintln!("Error: Input is required for text widgets.");
                return;
              }
            },
            "file" => {
              if !input.is_empty() {
                WidgetCommand::File(cli_setup::FileArgs {
                  name: std::path::PathBuf::from(input.join(" ")),
                })
              } else {
                eprintln!("Error: Input is required for file widgets.");
                return;
              }
            },
            _ => {
              eprintln!("Error: Unsupported widget type {}.", widget);
              return;
            },
          };

          // Validate the widget can produce a valid message
          if let Err(e) = process_and_validate_widget(&widget_command).await {
            log::error!("Scheduled widget validation failed: {}", e);
            eprintln!("Error validating scheduled widget: {}", e);
            return;
          }

          log::debug!("Scheduled widget validation successful");

          // Convert back to the format expected by the scheduler
          let input_json: serde_json::Value;
          let widget_lower = widget.to_lowercase();
          match widget_lower.as_str() {
            "weather" | "sat-word" | "jokes" | "clear" => {
              input_json = json!(null);
            },
            "text" | "file" => {
              input_json = serde_json::to_value(input.join(" ")).unwrap();
            },
            _ => {
              log::error!("Unsupported widget type: {}", widget_lower);
              eprintln!("Error: Unsupported widget type {}.", widget_lower);
              return;
            },
          }

          match add_task_to_schedule(datetime_utc, widget_lower, input_json) {
            Ok(_) => {
              log::info!("Successfully added task to schedule");
            },
            Err(e) => {
              log::error!("Failed to add task to schedule: {}", e);
              eprintln!("Error adding task to schedule: {}", e);
            },
          }
        },
        ScheduleArgs::Remove { id } => {
          log::info!("Removing scheduled task: {}", id);
          println!("Removing scheduled task {}...", id);
          match remove_task_from_schedule(&id) {
            Ok(removed) => {
              if removed {
                log::info!("Task removed successfully");
                println!("Task removed successfully");
              } else {
                log::info!("No tasks removed");
                println!("Task not found, no tasks removed");
              }
            }
            Err(e) => {
              log::error!("Failed to remove task: {}", e);
              eprintln!("Error removing task: {}", e);
            },
          }
        },
        ScheduleArgs::List => {
          log::info!("Listing scheduled tasks");
          println!("Listing tasks...");
          match list_schedule() {
            Ok(_) => log::debug!("Listed tasks successfully"),
            Err(e) => {
              log::error!("Failed to list tasks: {}", e);
              eprintln!("Error listing tasks: {}", e);
            },
          }
        },
        ScheduleArgs::Clear => {
          log::info!("Clearing all scheduled tasks");
          println!("Clearing schedule...");
          match clear_schedule() {
            Ok(_) => log::info!("Successfully cleared schedule"),
            Err(e) => {
              log::error!("Failed to clear schedule: {}", e);
              eprintln!("Error clearing schedule: {}", e);
            },
          }
        },
        ScheduleArgs::Dryrun => {
          log::info!("Running schedule dry run");
          println!("Dry run...");
          print_schedule().await
        },
      }
    },
    Command::Daemon => {
      log::info!("Starting daemon mode");
      match run_daemon().await {
        Ok(_) => log::info!("Daemon completed successfully"),
        Err(e) => {
          log::error!("Daemon failed: {}", e);
          eprintln!("Daemon error: {}", e);
        },
      }
    },
  }
}

#[cfg(test)]
mod tests;
