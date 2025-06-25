mod errors;
mod datetime;
mod api;
mod api_broker;
mod cli_display;
mod cli_setup;
mod widgets;
mod scheduler;
mod daemon;

use clap::Parser;
use serde_json::json;
use api_broker::{ display_message, validate_message_content };
use cli_display::print_message;
use cli_setup::{ Cli, Command, ScheduleArgs, WidgetCommand };
use scheduler::{
    add_task_to_schedule,
    remove_task_from_schedule,
    clear_schedule,
    list_schedule,
    print_schedule,
};
use widgets::text::{ get_text, get_text_from_file };
use widgets::weather::get_weather;
use widgets::jokes::get_joke;
use widgets::sat_words::get_sat_word;
use daemon::run_daemon;
use datetime::datetime_to_utc;

/// Processes a widget command and validates the resulting message
/// This ensures all messages are validated before any output method
async fn process_and_validate_widget(
    widget_command: &WidgetCommand
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let message_result = match widget_command {
        WidgetCommand::Text(args) => get_text(&args.message),
        WidgetCommand::File(args) => get_text_from_file(args.name.clone()),
        WidgetCommand::Weather => get_weather().await,
        WidgetCommand::Jokes => get_joke(),
        WidgetCommand::SATWord => get_sat_word(),
        WidgetCommand::Clear => Ok(vec![String::from("")]), // Clear command
    };

    let message = match message_result {
        Ok(msg) => msg,
        Err(e) => {
            return Err(format!("Widget error: {}", e).into());
        }
    };

    // Single validation point for all messages
    if let Err(validation_error) = validate_message_content(&message) {
        return Err(validation_error.into());
    }

    Ok(message)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut test_mode = false;

    match cli.command {
        Command::Send(send_args) => {
            if send_args.dry_run {
                test_mode = true;
            }

            // Handle clear command separately since it doesn't need validation
            if matches!(send_args.widget_command, WidgetCommand::Clear) {
                if test_mode {
                    print_message(vec![String::from("")], "");
                } else {
                    if let Err(e) = api::clear_board().await {
                        eprintln!("Error clearing board: {}", e);
                        std::process::exit(1);
                    }
                }
                return;
            }

            // Process and validate the widget message
            let message = match process_and_validate_widget(&send_args.widget_command).await {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };

            if test_mode {
                print_message(message, "");
                return;
            }
            display_message(message).await;
        }
        Command::Schedule { action } => {
            match action {
                ScheduleArgs::Add { time, widget, input } => {
                    println!("Scheduling task...");
                    let datetime_utc = match datetime_to_utc(&time) {
                        Ok(dt) => dt,
                        Err(e) => {
                            println!("datetime: {}", time);
                            eprintln!("Error invalid datetime format: {}", e);
                            return;
                        }
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
                        }
                        "file" => {
                            if !input.is_empty() {
                                WidgetCommand::File(cli_setup::FileArgs {
                                    name: std::path::PathBuf::from(input.join(" ")),
                                })
                            } else {
                                eprintln!("Error: Input is required for file widgets.");
                                return;
                            }
                        }
                        _ => {
                            eprintln!("Error: Unsupported widget type {}.", widget);
                            return;
                        }
                    };

                    // Validate the widget can produce a valid message
                    if let Err(e) = process_and_validate_widget(&widget_command).await {
                        eprintln!("Error validating scheduled widget: {}", e);
                        return;
                    }

                    // Convert back to the format expected by the scheduler
                    let input_json: serde_json::Value;
                    let widget_lower = widget.to_lowercase();
                    match widget_lower.as_str() {
                        "weather" | "sat-word" | "jokes" | "clear" => {
                            input_json = json!(null);
                        }
                        "text" | "file" => {
                            input_json = serde_json::to_value(input.join(" ")).unwrap();
                        }
                        _ => {
                            eprintln!("Error: Unsupported widget type {}.", widget_lower);
                            return;
                        }
                    }
                    add_task_to_schedule(datetime_utc, widget_lower, input_json).unwrap();
                }
                ScheduleArgs::Remove { id } => {
                    println!("Removing task...");
                    remove_task_from_schedule(&id).unwrap()
                }
                ScheduleArgs::List => {
                    println!("Listing tasks...");
                    list_schedule().unwrap();
                }
                ScheduleArgs::Clear => {
                    println!("Clearing schedule...");
                    clear_schedule().unwrap();
                }
                ScheduleArgs::Dryrun => {
                    println!("Dry run...");
                    print_schedule().await
                }
            }
        }
        Command::Daemon => {
            run_daemon().await.unwrap();
        }
    }
}

#[cfg(test)]
mod tests;
