use clap::{ Parser };

mod errors;
mod api;
mod api_broker;
mod cli_display;
mod cli_setup;
mod widgets;
mod scheduler;
mod daemon;

use api_broker::display_message;
use cli_display::print_message;
use cli_setup::{ Cli, Command, ScheduleArgs, WidgetCommand };
use widgets::text::{ get_text, get_text_from_file };
use widgets::weather::get_weather;
use widgets::jokes::get_joke;
use widgets::sat_words::get_sat_word;
use daemon::run_daemon;

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
                        print_message(vec![String::from("")]);
                    } else {
                        api::clear_board().await.unwrap();
                    }
                    return;
                }
            };
            if test_mode {
                print_message(message.clone());
                return;
            }
            display_message(message.clone()).await;
        }
        Command::Schedule { action } => {
            match action {
                ScheduleArgs::Add { time, widget, input } => {
                    println!("Scheduling task...");
                }
                ScheduleArgs::Remove { id } => {
                    println!("Removing task...");
                }
                ScheduleArgs::List => {
                    println!("Listing tasks...");
                }
                ScheduleArgs::Clear => {
                    println!("Clearing schedule...");
                }
                ScheduleArgs::Dryrun => {
                    println!("Dry run...");
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
