#[path = "../cli_setup.rs"]
mod cli_setup;

use cli_setup::{Command, FileArgs, ScheduleArgs, ShowArgs, TextArgs, WidgetCommand};

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
