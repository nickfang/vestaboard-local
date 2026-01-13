#[path = "../widget_utils.rs"]
mod widget_utils;
use crate::errors::VestaboardError;
use std::io::{Error as IoError, ErrorKind};
use widget_utils::{center_line, error_to_display_message, format_error, format_message, full_justify_line};

#[cfg(test)]
mod tests {
  use super::*;

  // Original widget_utils tests
  #[test]
  fn test_center_line() {
    let line = "hello world".to_string();
    let centered = center_line(line);
    let expected = "     hello world      ";
    assert_eq!(centered, expected);
  }

  #[test]
  fn test_format_message_centered() {
    let message = "hello world";
    let formatted = format_message(message);
    let expected = vec!["", "", "     hello world      ", "", "", ""];
    assert_eq!(formatted, expected);
  }

  #[test]
  fn test_format_message_long_word() {
    let message = "thisisaverylongwordthatshouldwrap";
    let formatted = format_message(message);
    let expected = vec!["", "", "thisisaverylongwordtha", "     tshouldwrap      ", "", ""];
    assert_eq!(formatted, expected);
  }

  #[test]
  fn test_format_message_long_word_2() {
    let message = "1 1234567890123456789012 12345678901234567890123 1234567890 12345";
    let formatted = format_message(message);
    let expected = vec![
      "          1           ",
      "1234567890123456789012",
      "          3           ",
      "1234567890123456789012",
      "   1234567890 12345   ",
      "",
    ];
    assert_eq!(formatted, expected);
  }

  #[test]
  fn test_format_message_colors() {
    let message = "ROYGBVWKF";
    let formatted = format_message(message);
    let expected = vec!["", "", "      ROYGBVWKF       ", "", "", ""];
    assert_eq!(formatted, expected);
  }

  #[test]
  fn test_format_message_full_colors() {
    let message =
            "ROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYG";
    let formatted = format_message(message);
    let expected = vec![
      "ROYGBVWKROYGBVWKROYGBV",
      "WKROYGBVWKROYGBVWKROYG",
      "BVWKROYGBVWKROYGBVWKRO",
      "YGBVWKROYGBVWKROYGBVWK",
      "ROYGBVWKROYGBVWKROYGBV",
      "WKROYGBVWKROYGBVWKROYG",
    ];
    assert_eq!(formatted, expected);
  }

  #[test]
  fn test_format_error() {
    let message = "This is an error message to display on the Vestaboard.";
    let formatted = format_error(message);
    let expected = vec![
      "        error         ".to_string(),
      "R R R R R R R R R R R".to_string(),
      "   this is an error   ".to_string(),
      "message to display on ".to_string(),
      "   the vestaboard.    ".to_string(),
      "".to_string(),
    ];
    assert_eq!(formatted, expected);
  }

  #[test]
  fn test_full_justify_line() {
    let s1 = "hello".to_string();
    let s2 = "world".to_string();
    let justified = full_justify_line(s1, s2);
    let expected = "hello            world".to_string();
    assert_eq!(justified, expected);
    assert_eq!(expected.chars().count(), 22);
  }

  #[test]
  fn test_full_justify_line_long_words() {
    let longs1 = "thisisaverylongword".to_string();
    let longs2 = "thatshouldwrap".to_string();
    let justified = full_justify_line(longs1, longs2);
    let expected = "thisisaverylongword thatshouldwrap";
    assert_eq!(justified, expected);
  }

  #[test]
  fn test_full_justify_line_empty_strings() {
    let emptys1 = "".to_string();
    let s2 = "world".to_string();
    let justified = full_justify_line(emptys1, s2);
    let expected = "                 world".to_string();
    assert_eq!(justified, expected);
    assert_eq!(expected.chars().count(), 22);

    let s1 = "hello".to_string();
    let emptys2 = "".to_string();
    let justified = full_justify_line(s1, emptys2);
    let expected = "hello                 ".to_string();
    assert_eq!(justified, expected);
    assert_eq!(expected.chars().count(), 22);
  }

  // Error handling tests
  #[test]
  fn test_error_to_display_message_io_error() {
    let io_err = IoError::new(ErrorKind::NotFound, "file not found");
    let error = VestaboardError::io_error(io_err, "reading config file");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "      file error      ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // With vertical centering, 1-line content should be on line 3 (middle of 4 available lines)
    assert_eq!(display[2], ""); // Empty padding line
    assert_eq!(display[3], "   'file' not found   ");
    assert_eq!(display[4], ""); // Empty padding line
    assert_eq!(display[5], ""); // Empty padding line
  }

  #[test]
  fn test_error_to_display_message_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("{invalid json").unwrap_err();
    let error = VestaboardError::json_error(json_err, "parsing schedule data");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "      data error      ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // With vertical centering, 1-line content should be on line 3
    assert_eq!(display[2], ""); // Empty padding line
    assert_eq!(display[3], " invalid data format  ");
    assert_eq!(display[4], ""); // Empty padding line
    assert_eq!(display[5], ""); // Empty padding line
  }

  // We'll skip the reqwest error test as it's complex to create reqwest::Error in tests
  // The functionality is covered by integration tests in weather widget

  #[test]
  fn test_error_to_display_message_widget_error() {
    let error = VestaboardError::widget_error("weather", "API key missing");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "     widget error     ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // The message might be split across multiple lines, so check the combined text without spaces
    let combined_message = display[2..].join("").replace(" ", "");
    assert!(combined_message.contains("weatherdataunavailable"));
  }

  #[test]
  fn test_error_to_display_message_text_widget_error() {
    let error = VestaboardError::widget_error("text", "Invalid characters");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "     widget error     ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // 1-line content is vertically centered at line 3
    assert_eq!(display[2], ""); // Empty padding line
    assert_eq!(display[3], "text processing error ");
    assert_eq!(display[4], ""); // Empty padding line
    assert_eq!(display[5], ""); // Empty padding line
  }

  #[test]
  fn test_error_to_display_message_sat_word_widget_error() {
    let error = VestaboardError::widget_error("sat-word", "Dictionary not found");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "     widget error     ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // 1-line content is vertically centered at line 3
    assert_eq!(display[2], ""); // Empty padding line
    assert_eq!(display[3], "dictionary unavailable");
    assert_eq!(display[4], ""); // Empty padding line
    assert_eq!(display[5], ""); // Empty padding line
  }

  #[test]
  fn test_error_to_display_message_unknown_widget_error() {
    let error = VestaboardError::widget_error("unknown", "Some error");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "     widget error     ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // 1-line content is vertically centered at line 3
    assert_eq!(display[2], ""); // Empty padding line
    assert_eq!(display[3], "    unknown error     ");
    assert_eq!(display[4], ""); // Empty padding line
    assert_eq!(display[5], ""); // Empty padding line
  }

  #[test]
  fn test_error_to_display_message_schedule_error() {
    let error = VestaboardError::schedule_error("save_schedule", "Disk full");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "    schedule error    ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // 1-line content is vertically centered at line 3
    assert_eq!(display[2], ""); // Empty padding line
    assert_eq!(display[3], "    schedule error    ");
    assert_eq!(display[4], ""); // Empty padding line
    assert_eq!(display[5], ""); // Empty padding line
  }

  #[test]
  fn test_error_to_display_message_api_error_404() {
    let error = VestaboardError::api_error(Some(404), "Not found");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "      api error       ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // 1-line content is vertically centered at line 3
    assert_eq!(display[2], ""); // Empty padding line
    assert_eq!(display[3], "  service not found   ");
    assert_eq!(display[4], ""); // Empty padding line
    assert_eq!(display[5], ""); // Empty padding line
  }

  #[test]
  fn test_error_to_display_message_api_error_401() {
    let error = VestaboardError::api_error(Some(401), "Unauthorized");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "      api error       ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // 1-line content is vertically centered at line 3
    assert_eq!(display[2], ""); // Empty padding line
    assert_eq!(display[3], "    access denied     ");
    assert_eq!(display[4], ""); // Empty padding line
    assert_eq!(display[5], ""); // Empty padding line
  }

  #[test]
  fn test_error_to_display_message_api_error_500() {
    let error = VestaboardError::api_error(Some(500), "Internal server error");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "      api error       ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // This message spans 2 lines and is vertically centered in 4 available lines
    assert_eq!(display[2], ""); // Empty padding line
    assert_eq!(display[3], " service temporarily  ");
    assert_eq!(display[4], "         down         ");
    assert_eq!(display[5], ""); // Empty padding line
  }

  #[test]
  fn test_error_to_display_message_config_error() {
    let error = VestaboardError::config_error("API_KEY", "Environment variable not set");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "     config error     ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // This message spans 2 lines and is vertically centered in 4 available lines
    assert_eq!(display[2], ""); // Empty padding line
    assert_eq!(display[3], "   config: api_key    ");
    assert_eq!(display[4], "       missing        ");
    assert_eq!(display[5], ""); // Empty padding line
  }

  #[test]
  fn test_error_to_display_message_other_error_short() {
    let error = VestaboardError::other("short message");
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "        error         ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    assert_eq!(display[2], ""); // Empty line for vertical centering
    assert_eq!(display[3], "    short message     ");
    assert_eq!(display[4], ""); // Empty line for vertical centering
    assert_eq!(display[5], ""); // Empty line for vertical centering
  }

  #[test]
  fn test_error_to_display_message_other_error_long() {
    let long_message = "This is a very long error message that exceeds forty characters and should be truncated";
    let error = VestaboardError::other(long_message);
    let display = error_to_display_message(&error);

    assert!(!display.is_empty());
    assert_eq!(display[0], "        error         ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // Check that the content contains truncated message
    let combined_message = display[2..].join("").replace(" ", "");
    assert!(combined_message.contains("..."));
  }

  #[test]
  fn test_error_to_display_message_format_consistency() {
    // Test that all error messages follow the error format pattern
    let error = VestaboardError::widget_error("test", "test message");
    let display = error_to_display_message(&error);

    // Should follow the same format as format_error()
    assert!(!display.is_empty());
    assert_eq!(display[0], "     widget error     ");
    assert_eq!(display[1], "R R R R R R R R R R R");
    // Should have at least one content line
    assert!(display.len() >= 3);
  }
}
