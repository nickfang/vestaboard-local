//! Tests for the runner module.

use crate::runner::{ControlFlow, PLAYLIST_HELP, SCHEDULE_HELP};

#[test]
fn test_control_flow_variants() {
  let cont = ControlFlow::Continue;
  let exit = ControlFlow::Exit;
  assert_ne!(cont, exit);
}

#[test]
fn test_playlist_help_contains_expected_keys() {
  assert!(PLAYLIST_HELP.contains("p"));
  assert!(PLAYLIST_HELP.contains("r"));
  assert!(PLAYLIST_HELP.contains("n"));
  assert!(PLAYLIST_HELP.contains("q"));
  assert!(PLAYLIST_HELP.contains("?"));
}

#[test]
fn test_schedule_help_contains_expected_keys() {
  assert!(SCHEDULE_HELP.contains("q"));
  assert!(SCHEDULE_HELP.contains("?"));
}
