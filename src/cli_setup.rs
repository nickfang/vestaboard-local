use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct TextArgs {
  #[arg(
    required = true,
    help = "The message to display (use quotes if there are spaces)"
  )]
  pub message: String,
}

#[derive(Args, Debug)]
pub struct FileArgs {
  pub name: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum WidgetCommand {
  #[command(name = "text", about = "Display a text message")]
  Text(TextArgs),
  #[command(name = "file", about = "Display a message from a file")]
  File(FileArgs),
  #[command(name = "weather", about = "Display the weather")]
  Weather,
  #[command(name = "jokes", about = "Display a random joke")]
  Jokes,
  #[command(name = "clear", about = "Clear the Vestaboard")]
  Clear,
  #[command(name = "sat-word", about = "Display a random SAT word")]
  SATWord,
}

#[derive(Args, Debug)]
pub struct ShowArgs {
  #[command(subcommand)]
  pub widget_command: WidgetCommand,
  #[arg(
    short = 'd',
    long = "dry-run",
    help = "Preview message without updating Vestaboard"
  )]
  pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct CycleArgs {
  #[arg(
    short = 'i',
    long = "interval",
    default_value = "60",
    help = "Delay in seconds between messages"
  )]
  pub interval: u64,
  #[arg(
    short = 'w',
    long = "delay",
    default_value = "0",
    help = "Delay in seconds before showing first message"
  )]
  pub delay: u64,
  #[arg(
    short = 'd',
    long = "dry-run",
    help = "Preview mode - show messages without updating Vestaboard"
  )]
  pub dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub enum CycleCommand {
  #[command(
    name = "repeat",
    about = "Continuously repeat the cycle until stopped (Ctrl-C)",
    after_help = "Examples:\n  vbl cycle repeat\n  vbl cycle repeat --interval 300\n  vbl cycle repeat --delay 30 --dry-run"
  )]
  Repeat {
    #[command(flatten)]
    args: CycleArgs,
  },
}

#[derive(Subcommand, Debug)]
pub enum ScheduleArgs {
  #[command(name = "list", about = "List all scheduled messages")]
  List,
  #[command(
    name = "add",
    about = "Add a new scheduled message.  Message must be in lowercase letters.",
    arg_required_else_help = true,
    after_help = "Example:\n  vbl schedule add \"2025-05-01 08:30:30\" text \"Don\\'t panic!\"\n  vbl schedule add \"2025-05-01 20:00:30\" weather"
  )]
  Add {
    #[clap(
      help = "The time to (YYYY-MM-DD HH:MM:SS) in military time.",
      required = true
    )]
    time: String,
    #[clap(
      help = "The widget to use (text, file, weather, sat-word).",
      required = true
    )]
    widget: String,
    #[clap(help = "Widget input (optional).  To use quotes use \\' or \\\".")]
    input: Vec<String>,
  },
  #[command(
    name = "remove",
    about = "Remove a scheduled message by ID.  Run vbl schdule list to see the ID's"
  )]
  Remove {
    #[clap(help = "The ID of the scheduled task", required = true)]
    id: String,
  },
  #[command(name = "clear", about = "Clear all scheduled messages")]
  Clear,
  #[command(
    name = "preview",
    about = "Preview the schedule without updating the Vestaboard"
  )]
  Preview,
}

#[derive(Subcommand, Debug)]
pub enum Command {
  #[command(
    about = "Show a message on the Vestaboard",
    after_help = "Examples:\n  vbl show text \"Hello World\"\n  vbl show --dry-run weather\n  vbl show file message.txt"
  )]
  Show(ShowArgs),
  #[command(
    about = "Manage scheduled messages",
    after_help = "Examples:\n  vbl schedule add \"2025-05-01 08:30:30\" text \"Good morning!\"\n  vbl schedule list\n  vbl schedule preview"
  )]
  Schedule {
    #[command(subcommand)]
    action: ScheduleArgs,
  },
  #[command(
    about = "Cycle through scheduled tasks at set intervals",
    long_about = "Execute all tasks from the schedule.json file in order. The datetime constraints are ignored - tasks are executed based only on the specified interval. Use 'cycle repeat' for continuous cycling, or 'cycle' alone to run once.",
    after_help = "Examples:\n  vbl cycle                                    # Default: 60 second intervals, run once\n  vbl cycle --delay 30                        # Wait 30 seconds before starting\n  vbl cycle --interval 300                     # 5 minute intervals, run once\n  vbl cycle repeat                             # Continuous cycling\n  vbl cycle repeat --dry-run                   # Preview continuous mode\n\nNote: Uses tasks from schedule.json, ignoring their scheduled times."
  )]
  Cycle {
    #[command(subcommand)]
    command: Option<CycleCommand>,
    #[command(flatten)]
    args: CycleArgs,
  },
  #[command(about = "Run as a background daemon")]
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

  #[arg(
    short = 'q',
    long = "quiet",
    global = true,
    help = "Suppress all non-error output"
  )]
  pub quiet: bool,

  #[arg(
    short = 'v',
    long = "verbose",
    global = true,
    help = "Show detailed progress information"
  )]
  pub verbose: bool,
}
