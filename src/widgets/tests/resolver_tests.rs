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
}
