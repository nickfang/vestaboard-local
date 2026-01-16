//! Local network transport for Vestaboard.
//!
//! Uses the Vestaboard Local API which requires being on the same network as the device.

use crate::cli_display::{print_error, print_progress, print_success};
use crate::errors::VestaboardError;
use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::json;
use std::env;

use super::common::create_client;

/// Shared HTTP client for local API requests.
/// Uses connection pooling for better performance with repeated requests.
static LOCAL_CLIENT: Lazy<Client> = Lazy::new(create_client);

/// Local network transport for Vestaboard.
///
/// Sends messages via the Local API at `http://{IP}:7000/local-api/message`.
#[derive(Debug)]
pub struct LocalTransport {
  api_key: String,
  ip_address: String,
}

impl LocalTransport {
  /// Creates a new LocalTransport.
  ///
  /// # Errors
  /// Returns an error if required environment variables are not set:
  /// - `LOCAL_API_KEY` - The local API key from your Vestaboard
  /// - `IP_ADDRESS` - The IP address of your Vestaboard on the local network
  pub fn new() -> Result<Self, VestaboardError> {
    // Load .env file first so env vars are available
    dotenv().ok();

    // Get the API key, returning a helpful error if not set or empty
    let api_key = env::var("LOCAL_API_KEY")
      .ok()
      .filter(|s| !s.is_empty())
      .ok_or_else(|| {
        VestaboardError::config_error(
          "LOCAL_API_KEY",
          "Environment variable not set. Set it with: export LOCAL_API_KEY=your-key (or add to .env file). Find your local API key in the Vestaboard app under Settings.",
        )
      })?;

    let ip_address = env::var("IP_ADDRESS")
      .ok()
      .filter(|s| !s.is_empty())
      .ok_or_else(|| {
        VestaboardError::config_error(
          "IP_ADDRESS",
          "Environment variable not set. Set it with: export IP_ADDRESS=192.168.x.x (or add to .env file). Find your Vestaboard's IP address in the Vestaboard app under Settings.",
        )
      })?;

    Ok(Self { api_key, ip_address })
  }

  /// Send character codes to the Vestaboard via local network.
  pub async fn send_codes(&self, codes: [[u8; 22]; 6]) -> Result<(), VestaboardError> {
    let start_time = std::time::Instant::now();
    print_progress("Sending to Vestaboard...");

    let client = &*LOCAL_CLIENT;
    let url = format!("http://{}:7000/local-api/message", &self.ip_address);
    let body = json!(codes);

    log::debug!("Sending local API request to {}", url);
    log::trace!("Request body: {:?}", body);

    let res = client
      .post(&url)
      .header("X-Vestaboard-Local-Api-Key", &self.api_key)
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

  /// Get the current message displayed on the Vestaboard via local network.
  ///
  /// Note: This method is kept for future features but is not yet fully implemented.
  /// The return type should eventually return the actual message data.
  pub async fn get_message(&self) -> Result<(), VestaboardError> {
    let client = &*LOCAL_CLIENT;
    let url = format!("http://{}:7000/local-api/message", &self.ip_address);

    log::debug!("Getting message from local API at {}", url);

    let res = client
      .get(&url)
      .header("X-Vestaboard-Local-Api-Key", &self.api_key)
      .send()
      .await;

    match res {
      Ok(response) => {
        log::debug!("Response: {:?}", response);
        Ok(())
      },
      Err(e) => {
        let error = VestaboardError::reqwest_error(e, "Vestaboard");
        print_error(&error.to_user_message());
        Err(error)
      },
    }
  }
}
