//! Keyboard input handling for interactive runners.
//!
//! This module provides abstractions for keyboard input, including a mock
//! implementation for testing and a real implementation using crossterm.

use std::collections::VecDeque;
use std::io::IsTerminal;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub use crossterm::event::KeyCode;
use crossterm::event::{self, Event, KeyEvent};

use crate::errors::VestaboardError;

/// Trait for abstracting keyboard input sources.
///
/// This allows for easy mocking in tests while supporting real keyboard
/// input in production.
pub trait InputSource: Send {
  /// Get the next key if available (non-blocking).
  ///
  /// Returns `Some(KeyCode)` if a key is available, `None` otherwise.
  fn try_recv(&mut self) -> Option<KeyCode>;
}

/// Real keyboard listener using crossterm.
///
/// Spawns a background thread that polls for keyboard events and sends them
/// through a channel. This allows non-blocking keyboard input in async contexts.
pub struct KeyboardListener {
  receiver: Receiver<KeyCode>,
  _handle: JoinHandle<()>,
}

impl KeyboardListener {
  /// Create a new keyboard listener.
  ///
  /// This spawns a background thread that polls for keyboard input.
  /// Requires a TTY (terminal) for interactive input.
  pub fn new() -> Result<Self, VestaboardError> {
    // Check if stdin is a TTY (required for interactive mode)
    if !std::io::stdin().is_terminal() {
      return Err(VestaboardError::input_error("Interactive mode requires a terminal. Stdin is not a TTY."));
    }

    let (sender, receiver) = mpsc::channel();

    let handle = thread::spawn(move || {
      loop {
        // Poll for keyboard events with short timeout
        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
          if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
            if sender.send(code).is_err() {
              break; // Receiver dropped, exit thread
            }
          }
        }
      }
    });

    log::debug!("KeyboardListener started");
    Ok(Self {
      receiver,
      _handle: handle,
    })
  }
}

impl InputSource for KeyboardListener {
  fn try_recv(&mut self) -> Option<KeyCode> {
    match self.receiver.try_recv() {
      Ok(key) => Some(key),
      Err(TryRecvError::Empty) => None,
      Err(TryRecvError::Disconnected) => {
        log::warn!("Keyboard input thread disconnected");
        None
      },
    }
  }
}

/// Mock input source for testing.
///
/// Provides a predetermined sequence of keys for testing keyboard handling.
/// Keys are returned in order, and `None` is returned once all keys are exhausted.
pub struct MockInput {
  keys: VecDeque<KeyCode>,
}

impl MockInput {
  /// Create a new MockInput with the given sequence of keys.
  pub fn new(keys: Vec<KeyCode>) -> Self {
    Self { keys: keys.into() }
  }

  /// Create a MockInput from a slice of keys.
  pub fn with_keys(keys: &[KeyCode]) -> Self {
    Self {
      keys: keys.iter().copied().collect(),
    }
  }
}

impl InputSource for MockInput {
  fn try_recv(&mut self) -> Option<KeyCode> {
    self.keys.pop_front()
  }
}
