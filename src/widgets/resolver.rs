use std::path::PathBuf;
use std::time::Instant;
use serde_json::Value;

use crate::cli_display::{print_error, print_progress};
use crate::errors::VestaboardError;
use crate::widgets::{
  jokes::get_joke,
  sat_words::get_sat_word,
  text::{get_text, get_text_from_file},
  weather::get_weather,
};
use crate::{log_widget_error, log_widget_start, log_widget_success};

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

  // Print user-facing widget start message
  match widget_type {
    "text" => print_progress("Creating message..."),
    "file" => {
      let file_path = input.as_str().unwrap_or("");
      print_progress(&format!("Reading file: {}...", file_path));
    },
    "weather" => print_progress("Fetching weather for Austin, TX..."),
    "jokes" => print_progress("Getting joke..."),
    "sat-word" => print_progress("Selecting SAT word..."),
    "clear" => print_progress("Clearing board..."),
    _ => {},
  }

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
      print_error(&e.to_user_message());
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
