mod api;
mod api_broker;
mod cli_display;
mod cli_setup;
mod config;
mod daemon;
mod datetime;
mod errors;
mod logging;
mod process_control;
mod scheduler;
mod widgets;

use api_broker::{handle_message, MessageDestination};
use cli_setup::{Cli, Command, CycleCommand, ScheduleArgs, WidgetCommand};
use daemon::run_daemon;
use datetime::datetime_to_utc;
use errors::VestaboardError;
use scheduler::{
  add_task_to_schedule, clear_schedule, list_schedule, preview_schedule, remove_task_from_schedule,
};
use widgets::resolver::execute_widget;
use widgets::widget_utils::error_to_display_message;

use clap::Parser;
use serde_json::json;


async fn process_widget_command(
  widget_command: &WidgetCommand,
  dry_run: bool,
) -> Result<(), VestaboardError> {
  let (widget_name, input_value) = match widget_command {
    WidgetCommand::Text(args) => ("text", json!(&args.message)),
    WidgetCommand::File(args) => ("file", json!(args.name.to_string_lossy())),
    WidgetCommand::Weather => ("weather", json!(null)),
    WidgetCommand::Jokes => ("jokes", json!(null)),
    WidgetCommand::SATWord => ("sat-word", json!(null)),
    WidgetCommand::Clear => ("clear", json!(null)),
  };

  // In dry-run mode, handle errors by converting them to display messages
  let message = match execute_widget(widget_name, &input_value).await {
    Ok(message) => message,
    Err(e) => error_to_display_message(&e),
  };

  let destination = if dry_run {
    MessageDestination::Console
  } else {
    MessageDestination::Vestaboard
  };

  match handle_message(message.clone(), destination).await {
    Ok(_) => Ok(()),
    Err(e) => {
      log::error!("Failed to handle message: {}", e);
      eprintln!("Error handling message: {}", e);
      Err(e)
    }
  }
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

  match cli.command {
    Command::Send(send_args) => {
      log::info!(
        "Processing send command with dry_run: {}",
        send_args.dry_run
      );

      match process_widget_command(&send_args.widget_command, send_args.dry_run).await {
        Ok(_) => {},
        Err(e) => {
          log::error!("Failed to process widget command: {}", e);
          eprintln!("Error processing widget command: {}", e);
        }
      }
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
          if let Err(e) = process_widget_command(&widget_command, false).await {
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
              println!("Task scheduled successfully");
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
            Ok(_) => {
              log::info!("Successfully cleared schedule");
              println!("Schedule cleared successfully");
            },
            Err(e) => {
              log::error!("Failed to clear schedule: {}", e);
              eprintln!("Error clearing schedule: {}", e);
            },
          }
        },
        ScheduleArgs::Preview => {
          log::info!("Running schedule preview");
          println!("Preview...");
          preview_schedule().await
        },
      }
    },
    Command::Cycle { command, args } => {
      // Use repeat args if available, otherwise use main cycle args
      let (is_repeat, cycle_args) = match command {
        Some(CycleCommand::Repeat { args: repeat_args }) => (true, repeat_args),
        None => (false, args),
      };
      
      log::info!(
        "Starting {} cycle mode - interval: {}s, delay: {}s, dry_run: {}",
        if is_repeat { "continuous" } else { "single" },
        cycle_args.interval,
        cycle_args.delay,
        cycle_args.dry_run
      );

      if cycle_args.dry_run {
        println!(
          "Running {} cycle in preview mode...",
          if is_repeat { "continuous" } else { "single" }
        );
      } else {
        println!(
          "Starting {} cycle with {} second intervals...",
          if is_repeat { "continuous" } else { "single" },
          cycle_args.interval
        );
      }

      if is_repeat {
        println!("Cycle will repeat continuously until stopped (Ctrl-C).");
      } else {
        println!("Cycle will run through all scheduled tasks once.");
      }

      if cycle_args.delay > 0 {
        println!("Waiting {} seconds before starting...", cycle_args.delay);
        tokio::time::sleep(tokio::time::Duration::from_secs(cycle_args.delay)).await;
      }

      // TODO: Implement cycle functionality
      log::warn!(
        "{} cycle functionality not yet implemented",
        if is_repeat { "Continuous" } else { "Single" }
      );
      println!(
        "{} cycle functionality is not yet implemented.",
        if is_repeat { "Continuous" } else { "Single" }
      );
      println!("This command will read from schedule.json and execute tasks in order:");
      println!("  - Ignoring scheduled datetime constraints");
      println!("  - Using {} second intervals between tasks", cycle_args.interval);
      if is_repeat {
        println!("  - Continuously repeating the cycle until Ctrl-C");
      } else {
        println!("  - Running through the schedule once");
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
