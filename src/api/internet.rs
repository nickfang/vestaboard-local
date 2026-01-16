//! Internet transport for Vestaboard.
//!
//! Uses the Vestaboard Read/Write API which works over the internet
//! without requiring local network access to the device.

use crate::cli_display::{ print_error, print_progress, print_success };
use crate::errors::VestaboardError;
use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::json;
use std::env;

use super::common::create_client;

/// Vestaboard Read/Write API endpoint
const INTERNET_API_URL: &str = "https://rw.vestaboard.com/";

/// Shared HTTP client for internet API requests.
/// Uses connection pooling for better performance with repeated requests.
static INTERNET_CLIENT: Lazy<Client> = Lazy::new(create_client);

/// Internet transport for Vestaboard.
///
/// Sends messages via the Read/Write API at `https://rw.vestaboard.com/`.
/// This transport works from anywhere with internet access.
#[derive(Debug)]
pub struct InternetTransport {
  api_key: String,
}

impl InternetTransport {
  /// Creates a new InternetTransport.
  ///
  /// # Errors
  /// Returns an error if the required environment variable is not set:
  /// - `INTERNET_API_KEY` - The Read/Write API key from your Vestaboard app
  pub fn new() -> Result<Self, VestaboardError> {
    // Load .env file first so env vars are available
    dotenv().ok();

    // Get the API key, returning a helpful error if not set or empty
    let api_key = env
      ::var("INTERNET_API_KEY")
      .ok()
      .filter(|s| !s.is_empty())
      .ok_or_else(|| {
        VestaboardError::config_error(
          "INTERNET_API_KEY",
          "Environment variable not set. Set it with: export INTERNET_API_KEY=your-key (or add to .env file)."
        )
      })?;

    Ok(Self { api_key })
  }

  /// Send character codes to the Vestaboard via internet.
  pub async fn send_codes(&self, codes: [[u8; 22]; 6]) -> Result<(), VestaboardError> {
    let start_time = std::time::Instant::now();
    print_progress("Sending to Vestaboard (internet)...");

    let client = &*INTERNET_CLIENT;
    let body = json!(codes);

    log::debug!("Sending internet API request to {}", INTERNET_API_URL);
    log::trace!("Request body: {:?}", body);

    let res = client
      .post(INTERNET_API_URL)
      .header("X-Vestaboard-Read-Write-Key", &self.api_key)
      .json(&body)
      .send().await;

    let duration = start_time.elapsed();

    match res {
      Ok(response) => {
        let status = response.status();
        log::info!("API response received: {} in {:?}", status, duration);

        // Get response body for all cases
        let response_body = response
          .text().await
          .unwrap_or_else(|_| "Unable to read response".to_string());

        if status.is_success() {
          print_success("Sent to Vestaboard");
          Ok(())
        } else if status.as_u16() == 304 {
          // 304 Not Modified - the internet API already has this message
          // (This compares against the last message sent via internet API, not what's currently displayed)
          print_success("Message unchanged (already sent via internet API)");
          Ok(())
        } else {
          log::error!("API error response: {}", response_body);
          print_error(&format!("Vestaboard error: HTTP {} - {}", status, response_body));
          Err(VestaboardError::api_error(Some(status.as_u16()), &response_body))
        }
      }
      Err(e) => {
        log::error!("API request failed after {:?}: {}", duration, e);
        let error = VestaboardError::reqwest_error(e, "Vestaboard");
        print_error(&error.to_user_message());
        Err(error)
      }
    }
  }

  /// Get the current message displayed on the Vestaboard via internet.
  ///
  /// Note: This method is kept for future features but is not yet fully implemented.
  /// The return type should eventually return the actual message data.
  pub async fn get_message(&self) -> Result<(), VestaboardError> {
    let client = &*INTERNET_CLIENT;

    log::debug!("Getting message from internet API at {}", INTERNET_API_URL);

    let res = client
      .get(INTERNET_API_URL)
      .header("X-Vestaboard-Read-Write-Key", &self.api_key)
      .send().await;

    match res {
      Ok(response) => {
        log::debug!("Response: {:?}", response);
        Ok(())
      }
      Err(e) => {
        let error = VestaboardError::reqwest_error(e, "Vestaboard");
        print_error(&error.to_user_message());
        Err(error)
      }
    }
  }
}
