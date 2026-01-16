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

static LOCAL_API_KEY: Lazy<String> = Lazy::new(|| {
  dotenv().ok();
  env::var("LOCAL_API_KEY").expect("LOCAL_API_KEY not set")
});

static IP_ADDRESS: Lazy<String> = Lazy::new(|| {
  dotenv().ok();
  env::var("IP_ADDRESS").expect("IP_ADDRESS not set")
});

/// Shared HTTP client for local API requests.
/// Uses connection pooling for better performance with repeated requests.
static LOCAL_CLIENT: Lazy<Client> = Lazy::new(create_client);

/// Local network transport for Vestaboard.
///
/// Sends messages via the Local API at `http://{IP}:7000/local-api/message`.
#[derive(Debug)]
pub struct LocalTransport;

impl LocalTransport {
  /// Creates a new LocalTransport.
  ///
  /// # Errors
  /// Returns an error if required environment variables are not set:
  /// - `LOCAL_API_KEY` - The local API key from your Vestaboard
  /// - `IP_ADDRESS` - The IP address of your Vestaboard on the local network
  pub fn new() -> Result<Self, VestaboardError> {
    // Force lazy initialization to validate env vars early
    // This provides a clear error at transport creation time rather than first use
    if env::var("LOCAL_API_KEY").is_err() {
      return Err(VestaboardError::config_error(
        "LOCAL_API_KEY",
        "Environment variable not set. Set it to your Vestaboard's local API key.",
      ));
    }
    if env::var("IP_ADDRESS").is_err() {
      return Err(VestaboardError::config_error(
        "IP_ADDRESS",
        "Environment variable not set. Set it to your Vestaboard's IP address.",
      ));
    }
    Ok(Self)
  }

  /// Send character codes to the Vestaboard via local network.
  pub async fn send_codes(&self, codes: [[u8; 22]; 6]) -> Result<(), VestaboardError> {
    let start_time = std::time::Instant::now();
    print_progress("Sending to Vestaboard...");

    let client = &*LOCAL_CLIENT;
    let url = format!("http://{}:7000/local-api/message", &*IP_ADDRESS);
    let body = json!(codes);

    log::debug!("Sending local API request to {}", url);
    log::trace!("Request body: {:?}", body);

    let res = client
      .post(&url)
      .header("X-Vestaboard-Local-Api-Key", &*LOCAL_API_KEY)
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
    let url = format!("http://{}:7000/local-api/message", &*IP_ADDRESS);

    log::debug!("Getting message from local API at {}", url);

    let res = client
      .get(&url)
      .header("X-Vestaboard-Local-Api-Key", &*LOCAL_API_KEY)
      .send()
      .await;

    match res {
      Ok(response) => {
        println!("Response: {:?}", response);
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
