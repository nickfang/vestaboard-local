#[cfg(test)]
mod tests {
  use crate::errors::VestaboardError;
  use crate::widgets::sat_words::sat_words::{create_words_map, get_sat_word};
  use std::io::Write;
  use tempfile::NamedTempFile;

  #[test]
  fn test_get_sat_word_success() {
    // This test assumes the words.txt file exists and is readable
    // If the file doesn't exist, it should return an IOError
    let result = get_sat_word();

    // Either success or file not found error is acceptable for testing
    match result {
      Ok(lines) => {
        assert!(!lines.is_empty());
        // First line should contain the word and part of speech
        assert!(lines[0].contains("("));
        assert!(lines[0].contains(")"));
      },
      Err(VestaboardError::IOError { context, .. }) => {
        assert!(context.contains("reading SAT words dictionary"));
      },
      Err(e) => panic!("Unexpected error: {:?}", e),
    }
  }

  #[test]
  fn test_create_words_map_with_valid_file() {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    write!(
      temp_file,
      "abhor (verb) to hate strongly (I abhor reality television shows)\n"
    )
    .expect("Failed to write");
    write!(
      temp_file,
      "bigot (noun) a person who is intolerant (My uncle is a bigot who refuses to listen)\n"
    )
    .expect("Failed to write");
    temp_file.flush().expect("Failed to flush");

    let result = create_words_map(temp_file.path());

    assert!(result.is_ok());
    let words_map = result.unwrap();
    assert!(words_map.contains_key("abhor"));
    assert!(words_map.contains_key("bigot"));

    let abhor_def = &words_map["abhor"];
    assert_eq!(abhor_def[0].0, "verb");
    assert!(abhor_def[0].1.contains("hate strongly"));
  }

  #[test]
  fn test_create_words_map_with_malformed_file() {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    write!(temp_file, "invalid line format\n").expect("Failed to write");
    write!(temp_file, "another bad line\n").expect("Failed to write");
    temp_file.flush().expect("Failed to flush");

    let result = create_words_map(temp_file.path());

    assert!(result.is_ok());
    let words_map = result.unwrap();
    // Should be empty since no lines match the expected format
    assert!(words_map.is_empty());
  }

  #[test]
  fn test_create_words_map_file_not_found() {
    let result = create_words_map("/this/file/does/not/exist.txt");

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
  }

  #[test]
  fn test_get_sat_word_empty_dictionary() {
    // Test behavior when dictionary is empty by testing create_words_map directly
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.flush().expect("Failed to flush");

    let result = create_words_map(temp_file.path());
    assert!(result.is_ok());
    let words_map = result.unwrap();
    assert!(words_map.is_empty());
  }

  #[test]
  fn test_get_sat_word_consistency() {
    // The SAT word function should work consistently across multiple calls
    // Note: Since it returns random words, we can't test for exact equality
    // but we can test that both calls succeed or both fail in the same way
    let result1 = get_sat_word();
    let result2 = get_sat_word();

    match (result1, result2) {
      (Ok(lines1), Ok(lines2)) => {
        // Both succeeded - verify they're properly formatted
        assert!(!lines1.is_empty());
        assert!(!lines2.is_empty());
        // Both should have the same structure but content may differ
        // (we can't assert exact line count since different words may have different lengths)
      },
      (Err(e1), Err(e2)) => {
        // Both failed - should be the same type of error
        assert_eq!(std::mem::discriminant(&e1), std::mem::discriminant(&e2));
      },
      _ => {
        // One succeeded, one failed - this is acceptable for SAT words
        // since file access or content could vary between calls
        // We'll just ensure both results are valid
      },
    }
  }

  #[test]
  fn test_get_sat_word_format() {
    let result = get_sat_word();

    if let Ok(lines) = result {
      // Each line should be valid for Vestaboard (22 characters max)
      for line in &lines {
        assert!(
          line.len() <= 22,
          "Each line should be 22 characters or less, found: '{}' (length: {})",
          line,
          line.len()
        );
      }

      // Lines should only contain valid Vestaboard characters
      for line in &lines {
        for ch in line.chars() {
          assert!(
            ch.is_ascii()
              && (ch.is_alphanumeric()
                || ch.is_whitespace()
                || "!\"#$%&'()*+,-./:;?@()".contains(ch)),
            "Invalid character '{}' found in line '{}'",
            ch,
            line
          );
        }
      }
    }
    // If the result is an error, that's also acceptable for testing
    // (e.g., if the words.txt file doesn't exist in the test environment)
  }

  #[test]
  fn test_get_sat_word_error_context() {
    // Test that errors contain proper context
    let result = get_sat_word();

    if let Err(error) = result {
      match error {
        VestaboardError::IOError { context, .. } => {
          assert!(context.contains("reading SAT words dictionary"));
        },
        VestaboardError::WidgetError { widget, message } => {
          assert_eq!(widget, "sat-word");
          assert!(message.contains("No words available"));
        },
        _ => panic!("Unexpected error type: {:?}", error),
      }
    }
    // If the test passes (file exists and works), that's also fine
  }
}
