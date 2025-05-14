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
use api_broker::display_message;
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut test_mode = false;

    match cli.command {
        Command::Send(send_args) => {
            if send_args.dry_run {
                test_mode = true;
            }
            let message: Vec<String> = match send_args.widget_command {
                WidgetCommand::Text(args) => { get_text(&args.message) }
                WidgetCommand::File(args) => { get_text_from_file(args.name) }
                WidgetCommand::Weather => { get_weather().await }
                WidgetCommand::Jokes => { get_joke() }
                WidgetCommand::SATWord => { get_sat_word() }
                WidgetCommand::Clear => {
                    if test_mode {
                        print_message(vec![String::from("")], "");
                    } else {
                        api::clear_board().await.unwrap();
                    }
                    return;
                }
            };
            if test_mode {
                print_message(message, "");
                return;
            }
            display_message(message.clone()).await;
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
                    let input_json: serde_json::Value;
                    let widget_lower = widget.to_lowercase();
                    match widget_lower.as_str() {
                        "weather" | "sat-word" | "jokes" => {
                            input_json = json!(null);
                        }
                        "text" | "file" => {
                            if !input.is_empty() {
                                input_json = serde_json::to_value(input.join(" ")).unwrap();
                            } else {
                                eprintln!("Error: Input is required for text and file widgets.");
                                return;
                            }
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
