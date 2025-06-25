#[cfg(test)]
mod tests {
    use crate::widgets::text::{ get_text, get_text_from_file };
    use crate::errors::VestaboardError;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_get_text_success() {
        let text = "hello world"; // Use lowercase which is valid for Vestaboard
        let result = get_text(text);

        assert!(result.is_ok());
        let lines = result.unwrap();
        assert!(!lines.is_empty());

        // Text should be somewhere in the lines (likely centered)
        let combined = lines.join("");
        assert!(combined.contains("hello world"));
    }

    #[test]
    fn test_get_text_with_invalid_characters() {
        // Test with uppercase letters which are invalid for Vestaboard
        // The widget should format the message successfully - validation happens at main level
        let text = "Hello World"; // "H" and "W" are invalid uppercase characters
        let result = get_text(text);

        assert!(result.is_ok());
        let lines = result.unwrap();
        assert!(!lines.is_empty());

        // The formatted message should contain the original text (even with invalid chars)
        let combined = lines.join("");
        assert!(combined.contains("Hello World"));
    }

    #[test]
    fn test_get_text_from_file_success() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file, "Line 1\nLine 2\nLine 3").expect("Failed to write to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let path = temp_file.path().to_path_buf();
        let result = get_text_from_file(path);

        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
        assert_eq!(lines[2], "Line 3");
    }

    #[test]
    fn test_get_text_from_file_not_found() {
        let non_existent_path = PathBuf::from("/this/file/does/not/exist.txt");
        let result = get_text_from_file(non_existent_path.clone());

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            VestaboardError::IOError { context, .. } => {
                assert!(context.contains("reading text file"));
                assert!(context.contains("/this/file/does/not/exist.txt"));
            }
            _ => panic!("Expected IOError"),
        }
    }

    #[test]
    fn test_get_text_from_empty_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let path = temp_file.path().to_path_buf();
        let result = get_text_from_file(path);

        assert!(result.is_ok());
        let lines = result.unwrap();
        // Empty file should return empty vector or vector with empty string
        // depending on fs::read_to_string behavior
        assert!(lines.is_empty() || (lines.len() == 1 && lines[0].is_empty()));
    }

    #[test]
    fn test_get_text_with_long_lines() {
        // Test text that will need to be wrapped (using lowercase)
        let long_text =
            "this is a very long line that should be wrapped because it exceeds the maximum length";
        let result = get_text(long_text);

        assert!(result.is_ok());
        let lines = result.unwrap();
        assert!(lines.len() > 1); // Should be wrapped into multiple lines

        // Check that no line exceeds the maximum length
        for line in lines {
            assert!(line.len() <= 22); // MAX_MESSAGE_LENGTH
        }
    }
}
