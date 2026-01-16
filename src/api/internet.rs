//! Internet transport for Vestaboard.
//!
//! Uses the Vestaboard Read/Write API which works over the internet
//! without requiring local network access to the device.

use crate::cli_display::{print_error, print_progress, print_success};
use crate::errors::VestaboardError;
use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::json;
use std::env;

use super::common::create_client;

/// Vestaboard Read/Write API endpoint
const INTERNET_API_URL: &str = "https://rw.vestaboard.com/";

static INTERNET_API_KEY: Lazy<String> = Lazy::new(|| {
  dotenv().ok();
  env::var("INTERNET_API_KEY").expect("INTERNET_API_KEY not set")
});

/// Shared HTTP client for internet API requests.
/// Uses connection pooling for better performance with repeated requests.
static INTERNET_CLIENT: Lazy<Client> = Lazy::new(create_client);

/// Internet transport for Vestaboard.
///
/// Sends messages via the Read/Write API at `https://rw.vestaboard.com/`.
/// This transport works from anywhere with internet access.
#[derive(Debug)]
pub struct InternetTransport;

impl InternetTransport {
  /// Creates a new InternetTransport.
  ///
  /// # Errors
  /// Returns an error if the required environment variable is not set:
  /// - `INTERNET_API_KEY` - The Read/Write API key from your Vestaboard app
  pub fn new() -> Result<Self, VestaboardError> {
    // Validate env var exists early for clear error messages
    if env::var("INTERNET_API_KEY").is_err() {
      return Err(VestaboardError::config_error(
        "INTERNET_API_KEY",
        "Environment variable not set. Get your Read/Write API key from the Vestaboard app.",
      ));
    }
    Ok(Self)
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
      .header("X-Vestaboard-Read-Write-Key", &*INTERNET_API_KEY)
      .json(&body)
      .send()
      .await;

    let duration = start_time.elapsed();

    match res {
      Ok(response) => {
        let status = response.status();
        log::info!("API response received: {} in {:?}", status, duration);
        log::debug!("Response: {:?}", response);
        if status.is_success() {
          print_success("Sent to Vestaboard");
        } else {
          print_error(&format!("Vestaboard error: HTTP {}", status));
        }
        Ok(())
      },
      Err(e) => {
        log::error!("API request failed after {:?}: {}", duration, e);
        let error = VestaboardError::reqwest_error(e, "Vestaboard");
        print_error(&error.to_user_message());
        Err(error)
      },
    }
  }
}
