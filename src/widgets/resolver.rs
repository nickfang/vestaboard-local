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
  widget_utils::error_to_display_message,
};
use crate::{log_widget_error, log_widget_start, log_widget_success};
use chrono::{DateTime, Utc};

/// Execute a widget by type string with unified error handling and logging
///
/// This function provides a single entry point for executing all widget types,
/// eliminating code duplication across main.rs, daemon.rs, and scheduler.rs.
/// 
/// Note: This function does NOT validate message content - that should be done
/// at the application layer by the caller using validate_message_content().
///
/// # Arguments
/// * `widget_type` - The type of widget to execute ("text", "file", "weather", etc.)
/// * `input` - JSON value containing widget-specific input parameters
/// * `dry_run` - If true, errors return error messages instead of failing
///
/// # Returns
/// * `Ok(Vec<String>)` - The generated message lines (NOT validated)
/// * `Err(VestaboardError)` - Widget execution error
pub async fn execute_widget(
  widget_type: &str,
  input: &Value,
  dry_run: bool,
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
      if dry_run {
        // In dry-run mode, we still want to show the error message
        error_to_display_message(&e)
      } else {
        return Err(e);
      }
    },
  };

  log::debug!(
    "Widget '{}' execution successful, message length: {} lines",
    widget_type,
    message.len()
  );

  Ok(message)
}

/// Execute a widget for dry-run display with timestamp
///
/// This function executes a widget in preview mode and displays the result
/// using the existing print_message functionality. Used by schedule dry-run.
/// Validation is performed here for preview mode, showing validation errors
/// as display messages rather than failing.
///
/// # Arguments
/// * `widget_type` - The type of widget to execute
/// * `input` - JSON value containing widget input parameters
/// * `scheduled_time` - Optional timestamp to display with the message
///
/// # Returns
/// * `Vec<String>` - The generated message (always succeeds, shows errors as messages)
pub async fn execute_widget_for_preview(
  widget_type: &str,
  input: &Value,
  scheduled_time: Option<DateTime<Utc>>,
) -> Vec<String> {
  // First execute the widget to get the raw message
  let raw_message = match execute_widget(widget_type, input, true).await {
    Ok(msg) => msg,
    Err(e) => {
      log::error!("Widget execution failed in preview: {}", e);
      let error_msg = error_to_display_message(&e);
      let time_str = scheduled_time
        .map(|t| datetime_to_local(t))
        .unwrap_or_else(|| "".to_string());
      print_message(error_msg.clone(), &time_str);
      return error_msg;
    },
  };

  // Validate the message content for preview
  let message = if let Err(validation_error) = validate_message_content(&raw_message) {
    log::error!(
      "Message validation failed for widget '{}' in preview: {}",
      widget_type,
      validation_error
    );
    // Show validation error as a display message in preview mode
    error_to_display_message(&VestaboardError::other(&validation_error))
  } else {
    log::debug!(
      "Widget '{}' preview validation successful, message length: {} lines",
      widget_type,
      raw_message.len()
    );
    raw_message
  };

  // Display the message using existing preview functionality
  let time_str = scheduled_time
    .map(|t| datetime_to_local(t))
    .unwrap_or_else(|| "".to_string());

  print_message(message.clone(), &time_str);
  message
}
