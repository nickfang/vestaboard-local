use std::path::PathBuf;
use clap::{ Args, Parser, Subcommand };

#[derive(Args, Debug)]
pub struct TextArgs {
    #[arg(required = true, help = "The message to display (use quotes if there are spaces)")]
    pub message: String,
}

#[derive(Args, Debug)]
pub struct FileArgs {
    pub name: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum WidgetCommand {
    #[command(name = "text", about = "Display a text message")] Text(TextArgs),
    #[command(name = "file", about = "Display a message from a file")] File(FileArgs),
    #[command(name = "weather", about = "Display the weather")] Weather,
    #[command(name = "jokes", about = "Display a random joke")] Jokes,
    #[command(name = "clear", about = "Clear the Vestaboard")] Clear,
    #[command(name = "sat-word", about = "Display a random SAT word")] SATWord,
}

#[derive(Args, Debug)]
pub struct SendArgs {
    #[command(subcommand)]
    pub widget_command: WidgetCommand,
    #[arg(short = 'd', long = "dry-run", help = "Show message content without updating Vestaboard")]
    pub dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub enum ScheduleArgs {
    #[command(name = "list", about = "List all scheduled messages")]
    List,
    #[command(name = "add", about = "Add a new scheduled message")] Add {
        #[clap(help = "The time to schedule the message", required = true)]
        time: String,
        #[clap(help = "The widget to use", required = true)]
        widget: String,
        #[clap(help = "The message to display", required = true)]
        input: Vec<String>,
    },
    #[command(name = "remove", about = "Remove a scheduled message")] Remove {
        #[clap(help = "The ID of the scheduled task", required = true)]
        id: String,
    },
    #[command(name = "clear", about = "Clear all scheduled messages")]
    Clear,
    #[command(name = "dry-run", about = "Test the schedule without updating the Vestaboard")]
    Dryrun,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Send(SendArgs),
    Schedule {
        #[command(subcommand)]
        action: ScheduleArgs,
    },
    Daemon,
}

#[derive(Parser, Debug)]
#[command(
    name = "Vestaboard CLI",
    author = "Nicholas Fang",
    version = "1.0",
    about = "CLI for updating a local Vestaboard",
    long_about = None
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}
