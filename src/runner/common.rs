//! Shared utilities for playlist and schedule runners.

use serde_json::Value;

use crate::api_broker::{handle_message, MessageDestination};
use crate::cli_display::{print_error, print_success};
use crate::errors::VestaboardError;
use crate::widgets::resolver::execute_widget;
use crate::widgets::widget_utils::error_to_display_message;

/// Execute a widget and send the result to the appropriate destination.
///
/// This function handles the common pattern of:
/// 1. Executing a widget to generate a message
/// 2. Converting errors to display messages (so the board shows something)
/// 3. Sending to Vestaboard or console based on dry_run mode
/// 4. Logging success/failure
///
/// # Arguments
/// * `widget` - The widget type to execute (e.g., "weather", "text")
/// * `input` - JSON input for the widget
/// * `dry_run` - If true, display to console instead of Vestaboard
/// * `label` - A label for logging (e.g., "task abc123", "item weather")
///
/// # Returns
/// * `Ok(())` - Message was sent successfully
/// * `Err(VestaboardError)` - Failed to send message (widget errors are handled internally)
pub async fn execute_and_send(
  widget: &str,
  input: &Value,
  dry_run: bool,
  label: &str,
) -> Result<(), VestaboardError> {
  // Execute widget, converting errors to display messages
  let message = match execute_widget(widget, input).await {
    Ok(msg) => msg,
    Err(e) => {
      log::error!("Widget '{}' failed: {}", widget, e);
      print_error(&format!("Widget {} failed: {}", widget, e.to_user_message()));
      error_to_display_message(&e)
    },
  };

  // Determine destination based on dry_run mode
  let destination = if dry_run {
    MessageDestination::Console
  } else {
    MessageDestination::Vestaboard
  };

  // Send message
  match handle_message(message, destination).await {
    Ok(_) => {
      log::info!("{} completed successfully", label);
      print_success(&format!("{} completed", label));
      Ok(())
    },
    Err(e) => {
      log::error!("Failed to send message for {}: {}", label, e);
      print_error(&e.to_user_message());
      Err(e)
    },
  }
}
