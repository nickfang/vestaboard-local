use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

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
  #[arg(short = 'd', long = "dry-run", help = "Preview message without updating Vestaboard")]
  pub dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub enum PlaylistArgs {
  #[command(
    name = "add",
    about = "Add a widget to the playlist",
    after_help = "Examples:\n  vbl playlist add weather\n  vbl playlist add text \"Hello world\"\n  vbl playlist add sat-word"
  )]
  Add {
    #[clap(help = "The widget to add (weather, text, sat-word, jokes, clear)", required = true)]
    widget: String,
    #[clap(help = "Widget input (required for text widget)")]
    input: Vec<String>,
  },
  #[command(name = "list", about = "List all playlist items")]
  List,
  #[command(name = "remove", about = "Remove a playlist item by ID")]
  Remove {
    #[clap(help = "The ID of the playlist item to remove", required = true)]
    id: String,
  },
  #[command(name = "clear", about = "Remove all playlist items")]
  Clear,
  #[command(name = "interval", about = "Show or set the rotation interval in seconds (minimum 60)")]
  Interval {
    #[clap(help = "Interval in seconds between items (omit to show current)")]
    seconds: Option<u64>,
  },
  #[command(name = "preview", about = "Preview all playlist items without sending to Vestaboard")]
  Preview,
  #[command(
    name = "run",
    about = "Run the playlist, rotating through items at the set interval",
    after_help = "Examples:\n  vbl playlist run\n  vbl playlist run --resume\n  vbl playlist run --once\n  vbl playlist run --index 2\n  vbl playlist run --id abc1\n  vbl playlist run --dry-run"
  )]
  Run {
    #[arg(long, help = "Run through playlist once and exit")]
    once: bool,
    #[arg(long, help = "Resume from last position", conflicts_with_all = ["index", "id"])]
    resume: bool,
    #[arg(long, help = "Start from this index (0-based)", conflicts_with_all = ["id", "resume"])]
    index: Option<usize>,
    #[arg(long, help = "Start from item with this ID", conflicts_with_all = ["index", "resume"])]
    id: Option<String>,
    #[arg(short = 'd', long = "dry-run", help = "Preview mode - show messages without sending to Vestaboard")]
    dry_run: bool,
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
    #[clap(help = "The time to (YYYY-MM-DD HH:MM:SS) in military time.", required = true)]
    time: String,
    #[clap(help = "The widget to use (text, file, weather, sat-word).", required = true)]
    widget: String,
    #[clap(help = "Widget input (optional).  To use quotes use \\' or \\\".")]
    input: Vec<String>,
  },
  #[command(name = "remove", about = "Remove a scheduled message by ID.  Run vbl schdule list to see the ID's")]
  Remove {
    #[clap(help = "The ID of the scheduled task", required = true)]
    id: String,
  },
  #[command(name = "clear", about = "Clear all scheduled messages")]
  Clear,
  #[command(name = "preview", about = "Preview the schedule without updating the Vestaboard")]
  Preview,
  #[command(
    name = "run",
    about = "Run the schedule, executing tasks at their scheduled times",
    after_help = "Examples:\n  vbl schedule run\n  vbl schedule run --dry-run"
  )]
  Run {
    #[arg(short = 'd', long = "dry-run", help = "Preview mode - show messages without sending to Vestaboard")]
    dry_run: bool,
  },
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
    about = "Manage and run the playlist",
    after_help = "Examples:\n  vbl playlist add weather\n  vbl playlist add text \"Hello world\"\n  vbl playlist list\n  vbl playlist remove abc1\n  vbl playlist interval 300"
  )]
  Playlist {
    #[command(subcommand)]
    action: PlaylistArgs,
  },
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

  #[arg(short = 'q', long = "quiet", global = true, help = "Suppress all non-error output")]
  pub quiet: bool,

  #[arg(short = 'v', long = "verbose", global = true, help = "Show detailed progress information")]
  pub verbose: bool,
}
