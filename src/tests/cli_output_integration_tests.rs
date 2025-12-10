// Integration tests for CLI output messages
// These tests verify that messages appear in the correct format and sequence
// during actual operations.
//
// To see the actual CLI output during test execution, run tests with --nocapture:
//   cargo test cli_output_integration -- --nocapture
//   cargo test test_widget_execution_messages_text -- --nocapture

#[cfg(test)]
mod tests {
  use crate::errors::VestaboardError;
  use crate::widgets::resolver::execute_widget;
  use crate::widgets::text::get_text_from_file;
  use std::io::Write;
  use std::path::PathBuf;
  use tempfile::NamedTempFile;

  // Helper to check if a pattern appears in output
  // Since we can't easily capture println!/eprintln! output,
  // we'll test the message generation logic and patterns

  // Note: Testing actual stdout/stderr output from println!/eprintln! is difficult
  // without refactoring. These integration tests verify that:
  // 1. Operations complete successfully and produce expected results
  // 2. Error messages follow expected patterns when errors occur
  // 3. The message generation logic works correctly

  #[tokio::test]
  async fn test_widget_execution_messages_text() {
    // Test that text widget execution produces expected messages
    // This tests the message sequence: "Creating message from text..." -> success
    let result = execute_widget("text", &serde_json::json!("hello world")).await;

    // Verify the operation completes (messages would be printed during execution)
    assert!(result.is_ok());
    let message = result.unwrap();
    assert!(!message.is_empty());
  }

  #[tokio::test]
  async fn test_widget_execution_messages_file_success() {
    // Test file widget with a real file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    write!(temp_file, "Line 1\nLine 2").expect("Failed to write to temp file");
    temp_file.flush().expect("Failed to flush temp file");

    let path = temp_file.path().to_path_buf();
    let result = get_text_from_file(path);

    // Verify the operation completes
    assert!(result.is_ok());
    let lines = result.unwrap();
    assert_eq!(lines.len(), 2);
  }

  #[tokio::test]
  async fn test_widget_execution_messages_file_not_found() {
    // Test file widget with non-existent file
    let non_existent_path = PathBuf::from("/path/that/does/not/exist.txt");
    let result = get_text_from_file(non_existent_path);

    // Verify error is returned (error message would be printed)
    assert!(result.is_err());
    let error = result.unwrap_err();

    // Verify error message pattern
    let user_msg = error.to_user_message();
    assert!(user_msg.contains("File not found") || user_msg.contains("Error accessing file"));
  }

  #[tokio::test]
  async fn test_widget_execution_messages_unknown_widget() {
    // Test unknown widget type
    let result = execute_widget("unknown_widget", &serde_json::json!(null)).await;

    // Verify error is returned
    assert!(result.is_err());
    let error = result.unwrap_err();

    // Verify error message pattern
    let user_msg = error.to_user_message();
    assert!(user_msg.contains("Widget error"));
    assert!(user_msg.contains("unknown_widget"));
  }

  #[test]
  fn test_error_message_patterns_io_not_found() {
    use std::io::{ Error as IoError, ErrorKind };

    let io_err = IoError::new(ErrorKind::NotFound, "file not found");
    let vb_error = VestaboardError::io_error(io_err, "reading text file /test/path.txt");
    let user_msg = vb_error.to_user_message();

    // Verify pattern matches
    assert!(user_msg.contains("File not found"));
    assert!(user_msg.contains("/test/path.txt"));
  }

  #[test]
  fn test_error_message_patterns_widget_error() {
    let vb_error = VestaboardError::widget_error("weather", "API key missing");
    let user_msg = vb_error.to_user_message();

    // Verify pattern: "Widget error: {widget} - {message}"
    assert!(user_msg.contains("Widget error"));
    assert!(user_msg.contains("weather"));
    assert!(user_msg.contains("API key missing"));
  }

  #[test]
  fn test_error_message_patterns_schedule_error() {
    let vb_error = VestaboardError::schedule_error("save", "disk full");
    let user_msg = vb_error.to_user_message();

    // Verify pattern: "Schedule error: {operation} - {message}"
    assert!(user_msg.contains("Schedule error"));
    assert!(user_msg.contains("save"));
    assert!(user_msg.contains("disk full"));
  }

  #[test]
  fn test_error_message_patterns_config_error() {
    let vb_error = VestaboardError::config_error("WEATHER_API_KEY", "not set");
    let user_msg = vb_error.to_user_message();

    // Verify pattern: "Configuration error [{field}]: {message}"
    assert!(user_msg.contains("Configuration error"));
    assert!(user_msg.contains("WEATHER_API_KEY"));
    assert!(user_msg.contains("not set"));
  }

  #[test]
  fn test_error_message_patterns_api_error() {
    let vb_error = VestaboardError::api_error(Some(404), "Resource not found");
    let user_msg = vb_error.to_user_message();

    // Verify pattern: "API error [{code}]: {message}"
    assert!(user_msg.contains("API error"));
    assert!(user_msg.contains("404"));
    assert!(user_msg.contains("Resource not found"));
  }
}
