#[path = "../resolver.rs"]
mod resolver;

#[cfg(test)]
mod tests {
  use crate::widgets::resolver::{
      execute_widget,
  };

  #[tokio::test]
  async fn test_execute_text_widget() {
    let result = execute_widget("text", &serde_json::json!("hello world"), false).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_execute_unknown_widget() {
    let result = execute_widget("unknown", &serde_json::json!(null), false).await;
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_execute_clear_widget() {
    let result = execute_widget("clear", &serde_json::json!(null), false).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![String::from("")]);
  }

  #[tokio::test]
  async fn test_execute_text_widget_with_null_input() {
    let result = execute_widget("text", &serde_json::json!(null), false).await;
    assert!(result.is_ok());
    // Should handle null as empty string
    let message = result.unwrap();
    assert_eq!(message.len(), 6); // 6 lines for Vestaboard display
  }

  #[tokio::test]
  async fn test_execute_text_widget_with_invalid_json() {
    // Test with number instead of string
    let result = execute_widget("text", &serde_json::json!(123), false).await;
    assert!(result.is_ok());
    // Should handle non-string as empty string
    let message = result.unwrap();
    assert_eq!(message.len(), 6); // 6 lines for Vestaboard display
  }

  #[tokio::test]
  async fn test_execute_file_widget_with_nonexistent_file() {
    let result = execute_widget("file", &serde_json::json!("/path/that/does/not/exist.txt"), false).await;
    assert!(result.is_err());
    // Should return a VestaboardError for file not found
  }

  #[tokio::test]
  async fn test_execute_file_widget_with_null_input() {
    let result = execute_widget("file", &serde_json::json!(null), false).await;
    assert!(result.is_err());
    // Should return error for empty file path
  }

  #[tokio::test]
  async fn test_execute_widget_dry_run_with_invalid_input() {
    // Test that dry run mode handles errors gracefully
    let result = execute_widget("file", &serde_json::json!("/invalid/path.txt"), true).await;
    assert!(result.is_ok()); // Dry run should return Ok with error message
    let message = result.unwrap();
    assert_eq!(message.len(), 6); // Should format error as 6-line message
    // Should contain some indication of the error
    let combined_message = message.join(" ");
    assert!(combined_message.to_lowercase().contains("error") || 
            combined_message.to_lowercase().contains("file") ||
            combined_message.to_lowercase().contains("not found"));
  }

  #[tokio::test]
  async fn test_execute_unknown_widget_dry_run() {
    // Test that unknown widget type in dry run returns formatted error
    let result = execute_widget("nonexistent_widget", &serde_json::json!(null), true).await;
    assert!(result.is_ok()); // Dry run should return Ok with error message
    let message = result.unwrap();
    assert_eq!(message.len(), 6); // Should format error as 6-line message
    let combined_message = message.join(" ");
    assert!(combined_message.to_lowercase().contains("unknown") || 
            combined_message.to_lowercase().contains("error"));
  }

  #[tokio::test]
  async fn test_execute_widget_with_empty_string_input() {
    let result = execute_widget("text", &serde_json::json!(""), false).await;
    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.len(), 6); // Should still format as 6 lines
  }

  #[tokio::test]
  async fn test_execute_widget_with_array_input() {
    // Test with invalid input type (array instead of string)
    let result = execute_widget("text", &serde_json::json!(["invalid", "array"]), false).await;
    assert!(result.is_ok());
    // Should handle invalid input gracefully
    let message = result.unwrap();
    assert_eq!(message.len(), 6);
  }
}
