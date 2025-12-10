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
use cli_display::{init_output_control, print_error, print_progress, print_success};
use cli_setup::{Cli, Command, CycleCommand, ScheduleArgs, WidgetCommand};
use std::process;
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
      print_error(&e.to_user_message());
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

  // Initialize output control (quiet, verbose, TTY detection)
  init_output_control(cli.quiet, cli.verbose);

  let exit_code = match cli.command {
    Command::Show(show_args) => {
      log::info!(
        "Processing show command with dry_run: {}",
        show_args.dry_run
      );

      match process_widget_command(&show_args.widget_command, show_args.dry_run).await {
        Ok(_) => 0,
        Err(e) => {
          log::error!("Failed to process widget command: {}", e);
          print_error(&e.to_user_message());
          1
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
          let datetime_utc = match datetime_to_utc(&time) {
            Ok(dt) => {
              log::debug!("Parsed datetime: {}", dt);
              let local_time = dt.with_timezone(&chrono::Local::now().timezone());
              let formatted_time = local_time.format("%Y-%m-%d %I:%M %p").to_string();
              print_progress(&format!("Scheduling task for {}...", formatted_time));
              dt
            },
            Err(e) => {
              log::error!("Invalid datetime format '{}': {}", time, e);
              print_error(&format!("Invalid datetime format: {}", e));
              process::exit(1);
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
                print_error("Input is required for text widgets.");
                process::exit(1);
              }
            },
            "file" => {
              if !input.is_empty() {
                WidgetCommand::File(cli_setup::FileArgs {
                  name: std::path::PathBuf::from(input.join(" ")),
                })
              } else {
                print_error("Input is required for file widgets.");
                process::exit(1);
              }
            },
            _ => {
              print_error(&format!("Unsupported widget type: {}", widget));
              process::exit(1);
            },
          };

          // Validate the widget can produce a valid message (dry-run mode - don't send to Vestaboard)
          print_progress("Validating...");
          if let Err(e) = process_widget_command(&widget_command, true).await {
            log::error!("Scheduled widget validation failed: {}", e);
            print_error(&e.to_user_message());
            process::exit(1);
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
              print_error(&format!("Unsupported widget type: {}", widget_lower));
              process::exit(1);
            },
          }

          match add_task_to_schedule(datetime_utc, widget_lower, input_json) {
            Ok(task_id) => {
              log::info!("Successfully added task {} to schedule", task_id);
              print_success(&format!("Task scheduled (ID: {})", task_id));
              0
            },
            Err(e) => {
              log::error!("Failed to add task to schedule: {}", e);
              print_error(&e.to_user_message());
              1
            },
          }
        },
        ScheduleArgs::Remove { id } => {
          log::info!("Removing scheduled task: {}", id);
          match remove_task_from_schedule(&id) {
            Ok(_) => 0,
            Err(e) => {
              log::error!("Failed to remove task: {}", e);
              print_error(&e.to_user_message());
              1
            },
          }
        },
        ScheduleArgs::List => {
          log::info!("Listing scheduled tasks");
          match list_schedule() {
            Ok(_) => {
              log::debug!("Listed tasks successfully");
              0
            },
            Err(e) => {
              log::error!("Failed to list tasks: {}", e);
              print_error(&e.to_user_message());
              1
            },
          }
        },
        ScheduleArgs::Clear => {
          log::info!("Clearing all scheduled tasks");
          match clear_schedule() {
            Ok(_) => {
              log::info!("Successfully cleared schedule");
              0
            },
            Err(e) => {
              log::error!("Failed to clear schedule: {}", e);
              print_error(&e.to_user_message());
              1
            },
          }
        },
        ScheduleArgs::Preview => {
          log::info!("Running schedule preview");
          preview_schedule().await;
          0
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
      0  // Return success for now (not implemented yet)
    },
    Command::Daemon => {
      log::info!("Starting daemon mode");
      match run_daemon().await {
        Ok(_) => {
          log::info!("Daemon completed successfully");
          0
        },
        Err(e) => {
          log::error!("Daemon failed: {}", e);
          print_error(&e.to_user_message());
          1
        },
      }
    },
  };

  process::exit(exit_code);
}

#[cfg(test)]
mod tests;
