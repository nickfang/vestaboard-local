use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::api::send_codes;
use crate::cli_display::{print_error, print_message, print_progress};
use crate::errors::VestaboardError;

#[derive(Debug)]
pub enum MessageDestination {
  Vestaboard,
  Console,
  ConsoleWithTitle(String),
}

static CHARACTER_CODES: Lazy<HashMap<char, u8>> = Lazy::new(|| {
  let characters = [
    (' ', 0),
    ('a', 1),
    ('b', 2),
    ('c', 3),
    ('d', 4),
    ('e', 5),
    ('f', 6),
    ('g', 7),
    ('h', 8),
    ('i', 9),
    ('j', 10),
    ('k', 11),
    ('l', 12),
    ('m', 13),
    ('n', 14),
    ('o', 15),
    ('p', 16),
    ('q', 17),
    ('r', 18),
    ('s', 19),
    ('t', 20),
    ('u', 21),
    ('v', 22),
    ('w', 23),
    ('x', 24),
    ('y', 25),
    ('z', 26),
    ('1', 27),
    ('2', 28),
    ('3', 29),
    ('4', 30),
    ('5', 31),
    ('6', 32),
    ('7', 33),
    ('8', 34),
    ('9', 35),
    ('0', 36),
    ('!', 37),
    ('@', 38),
    ('#', 39),
    ('$', 40),
    ('(', 41),
    (')', 42),
    ('-', 44),
    ('+', 46),
    ('&', 47),
    ('=', 48),
    (';', 49),
    (':', 50),
    ('\'', 52),
    ('"', 53),
    ('%', 54),
    (',', 55),
    ('.', 56),
    ('/', 59),
    ('?', 60),
    ('D', 62), // Degree symbol
    ('R', 63), // Red
    ('O', 64), // Orange
    ('Y', 65), // Yellow
    ('G', 66), // Green
    ('B', 67), // Blue
    ('V', 68), // Violet
    ('W', 69), // White
    ('K', 70), // Black
  ];
  characters.iter().cloned().collect()
});

pub fn to_codes(message: &str) -> Vec<u8> {
  let mut codes = Vec::new();

  for c in message.chars() {
    if let Some(&code) = CHARACTER_CODES.get(&c) {
      codes.push(code);
    }
    // Note: Invalid characters should have been caught during validation
    // If we reach here with invalid chars, it's a programming error
  }

  codes
}

/// Converts message lines to Vestaboard codes array for testing
/// This function is similar to display_message but returns the codes instead of sending them
pub fn message_to_codes(message: Vec<String>) -> [[u8; 22]; 6] {
  let mut codes: [[u8; 22]; 6] = [[0; 22]; 6];
  let mut current_line = [0; 22];
  let mut line_num = 0;

  for line in message {
    if line_num == 6 {
      break;
    }
    let line_codes = to_codes(&line);
    if line_codes.len() > 22 {
      eprintln!("Too many characters on line {:?}", line_num);
    }
    // make sure and pad with 0's or characters on the previous line will be duplicated
    for i in 0..22 {
      if i < line_codes.len() {
        current_line[i] = line_codes[i];
      } else {
        current_line[i] = 0;
      }
    }
    codes[line_num] = current_line;
    line_num += 1;
  }

  codes
}

pub async fn display_message(message: Vec<String>) -> Result<(), VestaboardError> {
  log::info!("Processing message for display, {} lines", message.len());
  log::debug!("Message content: {:?}", message);

  let codes = message_to_codes(message);
  log::debug!("Converted message to character codes");

  send_codes(codes).await
}

/// Checks if a character is valid for Vestaboard display
/// This is the single source of truth for valid characters
pub fn is_valid_character(c: char) -> bool {
  CHARACTER_CODES.contains_key(&c)
}

/// Gets all valid characters as a formatted string for error messages
pub fn get_valid_characters_description() -> String {
  "a-z, 0-9, space, punctuation (!@#$()-+&=;:'\"%,./?), D (degree), and color codes (ROYGBVWK)".to_string()
}

/// Validates that all characters in the message are valid for Vestaboard
pub fn validate_message_content(message: &[String]) -> Result<(), VestaboardError> {
  let mut invalid_chars = std::collections::HashSet::new();

  for line in message {
    for c in line.chars() {
      if !is_valid_character(c) {
        invalid_chars.insert(c);
      }
    }
  }

  if !invalid_chars.is_empty() {
    let mut chars: Vec<char> = invalid_chars.into_iter().collect();
    chars.sort();
    return Err(VestaboardError::ApiError {
      code: Some(400),
      message: format!(
        "Invalid characters found: {}. Valid characters are: {}.",
        chars.iter().map(|c| format!("'{}'", c)).collect::<Vec<_>>().join(", "),
        get_valid_characters_description()
      ),
    });
  }

  Ok(())
}

pub async fn handle_message(message: Vec<String>, destination: MessageDestination) -> Result<(), VestaboardError> {
  log::debug!("Handling message for destination: {:?}", destination);

  // Validate the message content
  match validate_message_content(&message) {
    Ok(_) => {
      log::debug!("Message validation successful");
    },
    Err(e) => {
      log::error!("Message validation failed: {}", e);
      print_error(&e.to_user_message());
      return Err(e);
    },
  }

  match destination {
    MessageDestination::Vestaboard => {
      display_message(message).await?;
    },
    MessageDestination::Console => {
      print_progress("Displaying message preview:");
      print_message(message, "");
    },
    MessageDestination::ConsoleWithTitle(title) => {
      print_progress("Displaying message preview:");
      print_message(message, &title);
    },
  }

  Ok(())
}
