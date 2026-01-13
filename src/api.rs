use crate::cli_display::{print_error, print_progress, print_success};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::json;
use std::env;
use std::time::Duration;

/// Default timeout for Vestaboard API requests (10 seconds)
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Creates an HTTP client configured with appropriate timeouts for Vestaboard API requests.
/// This prevents the application from hanging indefinitely when the Vestaboard is unreachable.
pub fn create_client() -> Client {
  Client::builder()
    .timeout(DEFAULT_TIMEOUT)
    .connect_timeout(DEFAULT_TIMEOUT)
    .build()
    .expect("Failed to build HTTP client")
}

static API_KEY: Lazy<String> = Lazy::new(|| {
  dotenv().ok();
  env::var("LOCAL_API_KEY").expect("LOCAL_API_KEY not set")
});
static IP_ADDRESS: Lazy<String> = Lazy::new(|| {
  dotenv().ok();
  env::var("IP_ADDRESS").expect("IP_ADDRESS not set")
});

/// Shared HTTP client for all Vestaboard API requests.
/// Uses connection pooling for better performance with repeated requests.
static CLIENT: Lazy<Client> = Lazy::new(create_client);

pub async fn send_codes(message: [[u8; 22]; 6]) -> Result<(), reqwest::Error> {
  let start_time = std::time::Instant::now();
  print_progress("Sending to Vestaboard...");

  let client = &*CLIENT;
  let url = format!("http://{}:7000/local-api/message", &*IP_ADDRESS);
  let body = json!(message);

  log::debug!("Sending API request to {}", url);
  log::trace!("Request body: {:?}", body);

  let res = client
    .post(&url)
    .header("X-Vestaboard-Local-Api-Key", &*API_KEY)
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
      let error = crate::errors::VestaboardError::reqwest_error(e, "Vestaboard");
      print_error(&error.to_user_message());
      // Extract the original reqwest error from the VestaboardError
      match error {
        crate::errors::VestaboardError::ReqwestError { source, .. } => Err(source),
        _ => unreachable!(),
      }
    },
  }
}

#[allow(dead_code)]
pub async fn clear_board() -> Result<(), reqwest::Error> {
  log::info!("Clearing Vestaboard");
  let message = [[0; 22]; 6];
  match send_codes(message).await {
    Ok(_) => {
      log::info!("Board cleared successfully");
      Ok(())
    },
    Err(e) => {
      log::error!("Failed to clear board: {}", e);
      eprintln!("Error: {:?}", e);
      Err(e)
    },
  }
}

#[allow(dead_code)]
pub async fn blank_board() -> Result<(), reqwest::Error> {
  let message = [[70; 22]; 6];
  match send_codes(message).await {
    Ok(_) => Ok(()),
    Err(e) => {
      eprintln!("Error: {:?}", e);
      Err(e)
    },
  }
}

#[allow(dead_code)]
pub async fn get_message() -> Result<(), reqwest::Error> {
  let client = &*CLIENT;
  let url = format!("http://{}:7000/local-api/message", &*IP_ADDRESS);

  let res = client
    .get(&url)
    .header("X-Vestaboard-Local-Api-Key", &*API_KEY)
    .send()
    .await;

  match res {
    Ok(response) => {
      println!("Response: {:?}", response);
      Ok(())
    },
    Err(e) => {
      eprintln!("Error: {:?}", e);
      Err(e)
    },
  }
}
