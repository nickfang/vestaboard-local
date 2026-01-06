//! Keyboard input handling for interactive runners.
//!
//! This module provides abstractions for keyboard input, including a mock
//! implementation for testing.

use std::collections::VecDeque;

/// Represents a keyboard key (simplified version without crossterm dependency)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    /// A character key
    Char(char),
    /// Enter/Return key
    Enter,
    /// Escape key
    Esc,
}

/// Trait for abstracting keyboard input sources.
///
/// This allows for easy mocking in tests while supporting real keyboard
/// input in production.
pub trait InputSource: Send {
    /// Get the next key if available (non-blocking).
    ///
    /// Returns `Some(KeyCode)` if a key is available, `None` otherwise.
    fn next_key(&mut self) -> Option<KeyCode>;
}

/// Mock input source for testing.
///
/// Provides a predetermined sequence of keys for testing keyboard handling.
/// Keys are returned in order, and `None` is returned once all keys are exhausted.
///
/// # Example
///
/// ```
/// use vestaboard_local::runner::keyboard::{MockInput, InputSource, KeyCode};
///
/// let mut mock = MockInput::new(vec![
///     KeyCode::Char('p'),
///     KeyCode::Char('q'),
/// ]);
///
/// assert_eq!(mock.next_key(), Some(KeyCode::Char('p')));
/// assert_eq!(mock.next_key(), Some(KeyCode::Char('q')));
/// assert_eq!(mock.next_key(), None);
/// ```
pub struct MockInput {
    keys: VecDeque<KeyCode>,
}

impl MockInput {
    /// Create a new MockInput with the given sequence of keys.
    pub fn new(keys: Vec<KeyCode>) -> Self {
        Self {
            keys: keys.into(),
        }
    }

    /// Create a MockInput from a slice of keys.
    pub fn with_keys(keys: &[KeyCode]) -> Self {
        Self {
            keys: keys.iter().copied().collect(),
        }
    }
}

impl InputSource for MockInput {
    fn next_key(&mut self) -> Option<KeyCode> {
        self.keys.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_input_provides_keys_in_order() {
        let mut mock = MockInput::new(vec![
            KeyCode::Char('p'),
            KeyCode::Char('r'),
            KeyCode::Char('q'),
        ]);

        assert_eq!(mock.next_key(), Some(KeyCode::Char('p')));
        assert_eq!(mock.next_key(), Some(KeyCode::Char('r')));
        assert_eq!(mock.next_key(), Some(KeyCode::Char('q')));
    }

    #[test]
    fn test_mock_input_returns_none_when_exhausted() {
        let mut mock = MockInput::new(vec![KeyCode::Char('q')]);

        assert_eq!(mock.next_key(), Some(KeyCode::Char('q')));
        assert_eq!(mock.next_key(), None);
        assert_eq!(mock.next_key(), None);
    }

    #[test]
    fn test_mock_input_empty_returns_none() {
        let mut mock = MockInput::new(vec![]);
        assert_eq!(mock.next_key(), None);
    }

    #[test]
    fn test_mock_input_with_keys() {
        let keys = [KeyCode::Char('a'), KeyCode::Char('b')];
        let mut mock = MockInput::with_keys(&keys);

        assert_eq!(mock.next_key(), Some(KeyCode::Char('a')));
        assert_eq!(mock.next_key(), Some(KeyCode::Char('b')));
        assert_eq!(mock.next_key(), None);
    }
}
