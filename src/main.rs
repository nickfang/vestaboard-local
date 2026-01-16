mod api;
mod api_broker;
mod cli_display;
mod cli_setup;
mod config;
mod datetime;
mod errors;
mod logging;
mod playlist;
mod process_control;
mod runner;
mod runtime_state;
mod scheduler;
mod widgets;

use api::{Transport, TransportType};
use api_broker::{handle_message, MessageDestination};
use config::Config;
use cli_display::{init_output_control, print_error, print_progress, print_success};
use cli_setup::{Cli, Command, PlaylistArgs, ScheduleArgs, WidgetCommand};
use datetime::datetime_to_utc;
use errors::VestaboardError;
use scheduler::{
  add_task_to_schedule, clear_schedule, list_schedule, preview_schedule, remove_task_from_schedule, run_schedule,
};
use std::process;
use widgets::resolver::execute_widget;
use widgets::widget_utils::error_to_display_message;

use clap::Parser;
use serde_json::json;

async fn process_widget_command(
  widget_command: &WidgetCommand,
  dry_run: bool,
  transport: &Transport,
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

  match handle_message(message.clone(), destination, transport).await {
    Ok(_) => Ok(()),
    Err(e) => {
      log::error!("Failed to handle message: {}", e);
      print_error(&e.to_user_message());
      Err(e)
    },
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

  // Determine transport type: CLI flag takes priority over config
  let config = Config::load_silent().unwrap_or_default();
  let transport_type = if cli.internet {
    TransportType::Internet
  } else {
    config.get_transport()
  };

  // Create transport (exit early if it fails)
  let transport = match Transport::new(transport_type) {
    Ok(t) => t,
    Err(e) => {
      log::error!("Failed to create transport: {}", e);
      print_error(&e.to_user_message());
      process::exit(1);
    },
  };

  let exit_code = match cli.command {
    Command::Show(show_args) => {
      log::info!("Processing show command with dry_run: {}", show_args.dry_run);

      match process_widget_command(&show_args.widget_command, show_args.dry_run, &transport).await {
        Ok(_) => 0,
        Err(e) => {
          log::error!("Failed to process widget command: {}", e);
          print_error(&e.to_user_message());
          1
        },
      }
    },
    Command::Schedule { action } => {
      log::info!("Processing schedule command");
      match action {
        ScheduleArgs::Add { time, widget, input } => {
          log::info!("Adding scheduled task - time: {}, widget: {}, input: {:?}", time, widget, input);
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
          if let Err(e) = process_widget_command(&widget_command, true, &transport).await {
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
          preview_schedule(&transport).await;
          0
        },
        ScheduleArgs::Run { dry_run } => {
          log::info!("Running schedule - dry_run: {}", dry_run);
          match run_schedule(dry_run, &transport).await {
            Ok(_) => 0,
            Err(e) => {
              log::error!("Schedule run failed: {}", e);
              print_error(&e.to_user_message());
              1
            },
          }
        },
      }
    },
    Command::Playlist { action } => {
      log::info!("Processing playlist command");
      match action {
        PlaylistArgs::Add { widget, input } => {
          log::info!("Adding playlist item - widget: {}, input: {:?}", widget, input);

          // Validate widget type and build input
          let widget_lower = widget.to_lowercase();
          let input_json = match widget_lower.as_str() {
            "weather" | "sat-word" | "jokes" | "clear" => json!(null),
            "text" => {
              if input.is_empty() {
                print_error("Input is required for text widgets.");
                process::exit(1);
              }
              json!(input.join(" "))
            },
            "file" => {
              if input.is_empty() {
                print_error("Input is required for file widgets.");
                process::exit(1);
              }
              json!(input.join(" "))
            },
            _ => {
              print_error(&format!(
                "Unsupported widget type: {}. Supported: weather, text, sat-word, jokes, clear, file",
                widget
              ));
              process::exit(1);
            },
          };

          // Validate the widget can produce a valid message (dry-run mode)
          let widget_command = match widget_lower.as_str() {
            "weather" => WidgetCommand::Weather,
            "sat-word" => WidgetCommand::SATWord,
            "jokes" => WidgetCommand::Jokes,
            "clear" => WidgetCommand::Clear,
            "text" => WidgetCommand::Text(cli_setup::TextArgs {
              message: input.join(" "),
            }),
            "file" => WidgetCommand::File(cli_setup::FileArgs {
              name: std::path::PathBuf::from(input.join(" ")),
            }),
            _ => unreachable!(), // Already handled above
          };

          print_progress("Validating widget...");
          if let Err(e) = process_widget_command(&widget_command, true, &transport).await {
            log::error!("Widget validation failed: {}", e);
            print_error(&e.to_user_message());
            process::exit(1);
          }

          match playlist::add_item_to_playlist(&widget_lower, input_json) {
            Ok(item_id) => {
              log::info!("Successfully added item {} to playlist", item_id);
              print_success(&format!("Added {} to playlist (ID: {})", widget_lower, item_id));
              0
            },
            Err(e) => {
              log::error!("Failed to add item to playlist: {}", e);
              print_error(&e.to_user_message());
              1
            },
          }
        },
        PlaylistArgs::List => {
          log::info!("Listing playlist items");
          match playlist::list_playlist() {
            Ok(_) => 0,
            Err(e) => {
              log::error!("Failed to list playlist: {}", e);
              print_error(&e.to_user_message());
              1
            },
          }
        },
        PlaylistArgs::Remove { id } => {
          log::info!("Removing playlist item: {}", id);
          match playlist::remove_item_from_playlist(&id) {
            Ok(_) => {
              print_success(&format!("Removed item {}", id));
              0
            },
            Err(e) => {
              log::error!("Failed to remove item: {}", e);
              print_error(&e.to_user_message());
              1
            },
          }
        },
        PlaylistArgs::Clear => {
          log::info!("Clearing all playlist items");
          match playlist::clear_playlist() {
            Ok(_) => {
              print_success("Playlist cleared.");
              0
            },
            Err(e) => {
              log::error!("Failed to clear playlist: {}", e);
              print_error(&e.to_user_message());
              1
            },
          }
        },
        PlaylistArgs::Interval { seconds } => match seconds {
          Some(secs) => {
            log::info!("Setting playlist interval to {} seconds", secs);
            match playlist::set_playlist_interval(secs) {
              Ok(_) => {
                print_success(&format!("Playlist interval set to {} seconds.", secs));
                0
              },
              Err(e) => {
                log::error!("Failed to set interval: {}", e);
                print_error(&e.to_user_message());
                1
              },
            }
          },
          None => {
            log::info!("Showing current playlist interval");
            match playlist::show_playlist_interval() {
              Ok(_) => 0,
              Err(e) => {
                log::error!("Failed to get interval: {}", e);
                print_error(&e.to_user_message());
                1
              },
            }
          },
        },
        PlaylistArgs::Preview => {
          log::info!("Previewing playlist");
          playlist::preview_playlist(&transport).await;
          0
        },
        PlaylistArgs::Run {
          once,
          resume,
          index,
          id,
          dry_run,
        } => {
          log::info!(
            "Running playlist - once: {}, resume: {}, index: {:?}, id: {:?}, dry_run: {}",
            once,
            resume,
            index,
            id,
            dry_run
          );
          match playlist::run_playlist(once, resume, index, id, dry_run, &transport).await {
            Ok(_) => 0,
            Err(e) => {
              log::error!("Playlist run failed: {}", e);
              print_error(&e.to_user_message());
              1
            },
          }
        },
      }
    },
  };

  process::exit(exit_code);
}

#[cfg(test)]
mod tests;
