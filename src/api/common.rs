//! Shared utilities for API transports.

use reqwest::Client;
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
