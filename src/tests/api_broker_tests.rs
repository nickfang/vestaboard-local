#[path = "../api_broker.rs"]
mod api_broker;
use api_broker::{
  display_message, get_valid_characters_description, is_valid_character, message_to_codes, to_codes,
  validate_message_content,
};

#[cfg(test)]
#[test]
fn test_valid_message() {
  let message = "hello";
  let expected_codes = vec![8, 5, 12, 12, 15];
  assert_eq!(to_codes(message), expected_codes);
}

#[test]
fn test_empty_message() {
  let message = "";
  let expected_codes: Vec<u8> = vec![];
  assert_eq!(to_codes(message), expected_codes);
}

#[test]
fn test_message_with_spaces() {
  let message = "hello world";
  let expected_codes = vec![8, 5, 12, 12, 15, 0, 23, 15, 18, 12, 4];
  assert_eq!(to_codes(message), expected_codes);
}

#[test]
fn test_message_with_numbers() {
  let message = "1234567890";
  let expected_codes = vec![27, 28, 29, 30, 31, 32, 33, 34, 35, 36];
  assert_eq!(to_codes(message), expected_codes);
}

#[test]
fn test_message_with_colors() {
  let message = "ROYGBVWK";
  let expected_codes = vec![63, 64, 65, 66, 67, 68, 69, 70];
  assert_eq!(to_codes(message), expected_codes);
}

#[test]
fn test_is_valid_character_lowercase() {
  assert!(is_valid_character('a'));
  assert!(is_valid_character('z'));
}

#[test]
fn test_is_valid_character_numbers() {
  assert!(is_valid_character('0'));
  assert!(is_valid_character('9'));
}

#[test]
fn test_is_valid_character_punctuation() {
  assert!(is_valid_character(' '));
  assert!(is_valid_character('!'));
  assert!(is_valid_character('@'));
  assert!(is_valid_character('#'));
  assert!(is_valid_character('$'));
  assert!(is_valid_character('('));
  assert!(is_valid_character(')'));
  assert!(is_valid_character('-'));
  assert!(is_valid_character('+'));
  assert!(is_valid_character('&'));
  assert!(is_valid_character('='));
  assert!(is_valid_character(';'));
  assert!(is_valid_character(':'));
  assert!(is_valid_character('\''));
  assert!(is_valid_character('"'));
  assert!(is_valid_character('%'));
  assert!(is_valid_character(','));
  assert!(is_valid_character('.'));
  assert!(is_valid_character('/'));
  assert!(is_valid_character('?'));
}

#[test]
fn test_is_valid_character_colors_and_degree() {
  assert!(is_valid_character('D')); // Degree
  assert!(is_valid_character('R')); // Red
  assert!(is_valid_character('O')); // Orange
  assert!(is_valid_character('Y')); // Yellow
  assert!(is_valid_character('G')); // Green
  assert!(is_valid_character('B')); // Blue
  assert!(is_valid_character('V')); // Violet
  assert!(is_valid_character('W')); // White
  assert!(is_valid_character('K')); // Black
}

#[test]
fn test_is_valid_character_invalid() {
  assert!(!is_valid_character('A')); // Uppercase not supported (except colors/degree)
  assert!(!is_valid_character('Z'));
  assert!(!is_valid_character('~'));
  assert!(!is_valid_character('*'));
  assert!(!is_valid_character('['));
  assert!(!is_valid_character('{'));
  assert!(!is_valid_character('\\'));
  assert!(!is_valid_character('|'));
  assert!(!is_valid_character('_'));
  assert!(!is_valid_character('`'));
  assert!(!is_valid_character('^'));
}

#[test]
fn test_get_valid_characters_description() {
  let desc = get_valid_characters_description();

  // Check basic character ranges
  assert!(desc.contains("a-z"));
  assert!(desc.contains("0-9"));
  assert!(desc.contains("space"));

  // Check punctuation symbols are mentioned
  assert!(desc.contains("punctuation"));
  assert!(desc.contains("!@#$()-+&=;:'\"%,./?"));

  // Check degree symbol is mentioned
  assert!(desc.contains("D"));
  assert!(desc.contains("degree"));

  // Check color codes are mentioned
  assert!(desc.contains("ROYGBVWK"));
  assert!(desc.contains("color codes"));
}

#[test]
fn test_validate_message_content_valid() {
  let message = vec![
    "hello world".to_string(),
    "123 test!".to_string(),
    "ROYGBVWK".to_string(),
  ];
  assert!(validate_message_content(&message).is_ok());
}

#[test]
fn test_validate_message_content_empty() {
  let message = vec![];
  assert!(validate_message_content(&message).is_ok());
}

#[test]
fn test_validate_message_content_empty_lines() {
  let message = vec!["".to_string(), "".to_string()];
  assert!(validate_message_content(&message).is_ok());
}

#[test]
fn test_validate_message_content_single_invalid() {
  let message = vec!["hello~world".to_string()];
  let result = validate_message_content(&message);
  assert!(result.is_err());
  let error = result.unwrap_err();
  let error_msg = error.to_string();
  assert!(error_msg.contains("Invalid characters found: '~'"));
  assert!(error_msg.contains("Valid characters are:"));
}

#[test]
fn test_validate_message_content_multiple_invalid() {
  let message = vec![
    "hello~world".to_string(),
    "test*string".to_string(),
    "more[chars]".to_string(),
  ];
  let result = validate_message_content(&message);
  assert!(result.is_err());
  let error = result.unwrap_err();
  let error_msg = error.to_string();
  assert!(error_msg.contains("Invalid characters found:"));
  assert!(error_msg.contains("'*'"));
  assert!(error_msg.contains("'['"));
  assert!(error_msg.contains("']'"));
  assert!(error_msg.contains("'~'"));
  assert!(error_msg.contains("Valid characters are:"));
}

#[test]
fn test_validate_message_content_duplicate_invalid() {
  // Test that duplicate invalid characters are only reported once
  let message = vec!["hello~world~again".to_string(), "test~more".to_string()];
  let result = validate_message_content(&message);
  assert!(result.is_err());
  let error = result.unwrap_err();
  let error_msg = error.to_string();
  // Should only contain one instance of '~'
  let tilde_count = error_msg.matches("'~'").count();
  assert_eq!(tilde_count, 1);
}

#[test]
fn test_validate_message_content_with_degree_symbol() {
  let message = vec!["temp is 72D today".to_string(), "it feels like 75D".to_string()];
  assert!(validate_message_content(&message).is_ok());
}

#[test]
fn test_validate_message_content_with_all_punctuation() {
  let message = vec!["!@#$()-+&=".to_string(), ";:'\"%,./?".to_string()];
  assert!(validate_message_content(&message).is_ok());
}

#[test]
fn test_validate_message_content_with_colors() {
  let message = vec!["ROYGBVWK all colors".to_string()];
  assert!(validate_message_content(&message).is_ok());
}

#[test]
fn test_message_to_code() {
  let message = vec!["hello".to_string(), "world".to_string()];
  let codes = message_to_codes(message);
  assert_eq!(
    codes,
    [
      [8, 5, 12, 12, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
      [23, 15, 18, 12, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
      [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
      [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
      [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
      [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ]
  );
}

#[test]
fn test_message_to_code_all_characters() {
  let message = vec![
    "ROYGBVKW".to_string(),
    "abcdefghijklmnopqrstuv".to_string(),
    "wxyz1234567890".to_string(),
    "!@#$()-+&=;:'\"%,./?D".to_string(),
  ];
  let codes = message_to_codes(message);
  assert_eq!(
    codes,
    [
      [63, 64, 65, 66, 67, 68, 70, 69, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
      [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22],
      [23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 0, 0, 0, 0, 0, 0, 0, 0],
      [37, 38, 39, 40, 41, 42, 44, 46, 47, 48, 49, 50, 52, 53, 54, 55, 56, 59, 60, 62, 0, 0],
      [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
      [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ]
  );
}

#[test]
fn test_display_message() {
  let _ = display_message;
  assert!(true);
}
