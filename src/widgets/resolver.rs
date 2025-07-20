use std::path::PathBuf;
use std::time::Instant;
use serde_json::Value;

use crate::api_broker::validate_message_content;
use crate::cli_display::print_message;
use crate::datetime::datetime_to_local;
use crate::errors::VestaboardError;
use crate::widgets::{
  jokes::get_joke,
  sat_words::get_sat_word,
  text::{get_text, get_text_from_file},
  weather::get_weather,
  widget_utils,
};
use crate::{log_widget_error, log_widget_start, log_widget_success};
use chrono::{DateTime, Utc};

/// Execute a widget by type string with unified error handling and logging
///
/// This function provides a single entry point for executing all widget types,
/// eliminating code duplication across main.rs, daemon.rs, and scheduler.rs.
///
/// # Arguments
/// * `widget_type` - The type of widget to execute ("text", "file", "weather", etc.)
/// * `input` - JSON value containing widget-specific input parameters
///
/// # Returns
/// * `Ok(Vec<String>)` - The generated message lines (NOT validated)
/// * `Err(VestaboardError)` - Widget execution error
pub async fn execute_widget(
  widget_type: &str,
  input: &Value,
) -> Result<Vec<String>, VestaboardError> {
  let start_time = Instant::now();

  // Extract input string for logging
  let input_str = match widget_type {
    "text" | "file" => input.as_str().unwrap_or(""),
    _ => "",
  };

  log_widget_start!(widget_type, input_str);

  let message_result = match widget_type {
    "text" => {
      let text_input = input.as_str().unwrap_or("");
      get_text(text_input)
    },
    "file" => {
      let file_path = input.as_str().unwrap_or("");
      get_text_from_file(PathBuf::from(file_path))
    },
    "weather" => get_weather().await,
    "jokes" => get_joke(),
    "sat-word" => get_sat_word(),
    "clear" => Ok(vec![String::from("")]), // Clear command
    _ => {
      let error = VestaboardError::widget_error(
        widget_type,
        &format!("Unknown widget type: {}", widget_type),
      );
      return Err(error);
    },
  };

  let duration = start_time.elapsed();

  let message = match message_result {
    Ok(msg) => {
      log_widget_success!(widget_type, duration);
      msg
    },
    Err(e) => {
      log_widget_error!(widget_type, e, duration);
      return Err(e);
    },
  };

  log::debug!(
    "Widget '{}' execution successful, message length: {} lines",
    widget_type,
    message.len()
  );

  Ok(message)
}

/// Execute a widget and print the result with timestamp (for schedule previews)
///
/// This function executes a widget and immediately prints the result using the
/// existing print_message functionality. Errors are converted to display messages
/// at the application layer.
///
/// # Arguments
/// * `widget_type` - The type of widget to execute
/// * `input` - JSON value containing widget input parameters
/// * `scheduled_time` - Optional timestamp to display with the message
///
/// # Returns
/// * `Vec<String>` - The generated message (always succeeds, shows errors as messages)
pub async fn print_widget_with_timestamp(
  widget_type: &str,
  input: &Value,
  scheduled_time: Option<DateTime<Utc>>,
) -> Vec<String> {
  // Execute widget and handle errors at application layer
  let message = match execute_widget(widget_type, input).await {
    Ok(msg) => {
      // Validate the message and convert validation errors to display messages
      match validate_message_content(&msg) {
        Ok(_) => msg,
        Err(validation_error) => {
          log::error!(
            "Message validation failed for widget '{}' in preview: {}",
            widget_type,
            validation_error
          );
          widget_utils::error_to_display_message(&VestaboardError::other(&validation_error))
        }
      }
    },
    Err(e) => {
      log::error!("Widget execution failed for '{}' in preview: {}", widget_type, e);
      widget_utils::error_to_display_message(&e)
    },
  };

  // Display the message using existing preview functionality
  let time_str = scheduled_time
    .map(|t| datetime_to_local(t))
    .unwrap_or_else(|| "".to_string());

  print_message(message.clone(), &time_str);
  message
}
