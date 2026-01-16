//! Tests for the keyboard input module.

use crate::runner::keyboard::{InputSource, KeyCode, MockInput};

#[test]
fn test_mock_input_provides_keys_in_order() {
  let mut mock = MockInput::new(vec![KeyCode::Char('p'), KeyCode::Char('r'), KeyCode::Char('q')]);

  assert_eq!(mock.try_recv(), Some(KeyCode::Char('p')));
  assert_eq!(mock.try_recv(), Some(KeyCode::Char('r')));
  assert_eq!(mock.try_recv(), Some(KeyCode::Char('q')));
}

#[test]
fn test_mock_input_returns_none_when_exhausted() {
  let mut mock = MockInput::new(vec![KeyCode::Char('q')]);

  assert_eq!(mock.try_recv(), Some(KeyCode::Char('q')));
  assert_eq!(mock.try_recv(), None);
  assert_eq!(mock.try_recv(), None);
}

#[test]
fn test_mock_input_empty_returns_none() {
  let mut mock = MockInput::new(vec![]);
  assert_eq!(mock.try_recv(), None);
}

#[test]
fn test_mock_input_with_keys() {
  let keys = [KeyCode::Char('a'), KeyCode::Char('b')];
  let mut mock = MockInput::with_keys(&keys);

  assert_eq!(mock.try_recv(), Some(KeyCode::Char('a')));
  assert_eq!(mock.try_recv(), Some(KeyCode::Char('b')));
  assert_eq!(mock.try_recv(), None);
}
