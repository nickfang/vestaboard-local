#[path = "../cli_setup.rs"]
mod cli_setup;

use cli_setup::{Cli, Command, FileArgs, PlaylistArgs, ScheduleArgs, ShowArgs, TextArgs, WidgetCommand};
use clap::Parser;

#[cfg(test)]
#[test]
fn test_widget_command_variants() {
  use std::path::PathBuf;

  // Ensure all WidgetCommand variants exist
  fn assert_widget_command(cmd: WidgetCommand) {
    match cmd {
      WidgetCommand::Text(_) => {},
      WidgetCommand::File(_) => {},
      WidgetCommand::Weather => {},
      WidgetCommand::Jokes => {},
      WidgetCommand::Clear => {},
      WidgetCommand::SATWord => {},
    }
  }

  // Call the function to ensure all variants are covered
  assert_widget_command(WidgetCommand::Text(TextArgs {
    message: String::from("example"),
  }));
  assert_widget_command(WidgetCommand::File(FileArgs { name: PathBuf::new() }));
  assert_widget_command(WidgetCommand::Weather);
  assert_widget_command(WidgetCommand::Jokes);
  assert_widget_command(WidgetCommand::Clear);
  assert_widget_command(WidgetCommand::SATWord);
}

#[test]
fn test_command_variants() {
  // Ensure all Command variants exist
  fn assert_command(cmd: Command) {
    match cmd {
      Command::Show(_) => {},
      Command::Schedule { action } => match action {
        ScheduleArgs::Add { .. } => {},
        ScheduleArgs::Remove { .. } => {},
        ScheduleArgs::List => {},
        ScheduleArgs::Clear => {},
        ScheduleArgs::Preview => {},
      },
      Command::Playlist { action } => match action {
        PlaylistArgs::Add { .. } => {},
        PlaylistArgs::List => {},
        PlaylistArgs::Remove { .. } => {},
        PlaylistArgs::Clear => {},
        PlaylistArgs::Interval { .. } => {},
        PlaylistArgs::Preview => {},
      },
      Command::Cycle { .. } => {},
      Command::Daemon => {},
    }
  }

  // Call the function to ensure all variants are covered
  assert_command(Command::Show(ShowArgs {
    widget_command: WidgetCommand::Clear,
    dry_run: false,
  }));
  assert_command(Command::Schedule {
    action: ScheduleArgs::Add {
      time: "2025-05-01T09:00:00Z".to_string(),
      widget: "Weather".to_string(),
      input: vec!["".to_string()],
    },
  });
  assert_command(Command::Playlist {
    action: PlaylistArgs::List,
  });
  assert_command(Command::Daemon);
}

#[test]
fn test_show_args() {
  // Test arguments for ShowArgs
  let show_args = ShowArgs {
    widget_command: WidgetCommand::Text(TextArgs {
      message: String::from("Test message"),
    }),
    dry_run: true,
  };

  // Check if the arguments are handled correctly
  assert_eq!(show_args.dry_run, true);
}

// --- Playlist CLI parsing tests ---

#[test]
fn test_cli_parses_playlist_add_weather() {
  let cli = Cli::parse_from(["vbl", "playlist", "add", "weather"]);
  match cli.command {
    Command::Playlist { action: PlaylistArgs::Add { widget, input } } => {
      assert_eq!(widget, "weather");
      assert!(input.is_empty());
    }
    _ => panic!("Expected Playlist Add command"),
  }
}

#[test]
fn test_cli_parses_playlist_add_text_with_input() {
  let cli = Cli::parse_from(["vbl", "playlist", "add", "text", "hello", "world"]);
  match cli.command {
    Command::Playlist { action: PlaylistArgs::Add { widget, input } } => {
      assert_eq!(widget, "text");
      assert_eq!(input, vec!["hello", "world"]);
    }
    _ => panic!("Expected Playlist Add command"),
  }
}

#[test]
fn test_cli_parses_playlist_list() {
  let cli = Cli::parse_from(["vbl", "playlist", "list"]);
  match cli.command {
    Command::Playlist { action: PlaylistArgs::List } => {}
    _ => panic!("Expected Playlist List command"),
  }
}

#[test]
fn test_cli_parses_playlist_remove() {
  let cli = Cli::parse_from(["vbl", "playlist", "remove", "abc1"]);
  match cli.command {
    Command::Playlist { action: PlaylistArgs::Remove { id } } => {
      assert_eq!(id, "abc1");
    }
    _ => panic!("Expected Playlist Remove command"),
  }
}

#[test]
fn test_cli_parses_playlist_clear() {
  let cli = Cli::parse_from(["vbl", "playlist", "clear"]);
  match cli.command {
    Command::Playlist { action: PlaylistArgs::Clear } => {}
    _ => panic!("Expected Playlist Clear command"),
  }
}

#[test]
fn test_cli_parses_playlist_interval_set() {
  let cli = Cli::parse_from(["vbl", "playlist", "interval", "120"]);
  match cli.command {
    Command::Playlist { action: PlaylistArgs::Interval { seconds } } => {
      assert_eq!(seconds, Some(120));
    }
    _ => panic!("Expected Playlist Interval command"),
  }
}

#[test]
fn test_cli_parses_playlist_interval_show() {
  let cli = Cli::parse_from(["vbl", "playlist", "interval"]);
  match cli.command {
    Command::Playlist { action: PlaylistArgs::Interval { seconds } } => {
      assert_eq!(seconds, None);
    }
    _ => panic!("Expected Playlist Interval command"),
  }
}

#[test]
fn test_cli_parses_playlist_preview() {
  let cli = Cli::parse_from(["vbl", "playlist", "preview"]);
  match cli.command {
    Command::Playlist { action: PlaylistArgs::Preview } => {}
    _ => panic!("Expected Playlist Preview command"),
  }
}
