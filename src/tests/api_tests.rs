#[path = "../api.rs"]
mod api;
use api::{blank_board, clear_board, get_message, send_codes};
// TODO: figure out how to test the api functions
#[cfg(test)]
#[tokio::test]
#[ignore]
async fn test_send_codes() {
  let message = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  ];
  let result = send_codes(message);
  assert!(result.await.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_clear_board() {
  let result = clear_board();
  assert!(result.await.is_ok());
}
#[tokio::test]
#[ignore]
async fn test_blank_board() {
  let result = blank_board();
  assert!(result.await.is_ok());
}
#[tokio::test]
#[ignore]
async fn test_get_message() {
  let result = get_message();
  assert!(result.await.is_ok());
}

// Tests for timeout behavior (Issue #52)
// These tests verify that the HTTP client has proper timeout configuration
// so the application doesn't hang indefinitely when the Vestaboard is unreachable.

#[cfg(test)]
mod timeout_tests {
  use super::api::{create_client, DEFAULT_TIMEOUT};
  use reqwest::Client;
  use std::time::{Duration, Instant};

  /// Test that DEFAULT_TIMEOUT is set to a reasonable value (not too short, not too long).
  #[test]
  fn test_default_timeout_is_reasonable() {
    // Timeout should be at least 5 seconds to allow for slow networks
    assert!(
      DEFAULT_TIMEOUT >= Duration::from_secs(5),
      "DEFAULT_TIMEOUT should be at least 5 seconds, got {:?}",
      DEFAULT_TIMEOUT
    );

    // Timeout should be at most 30 seconds to avoid excessive waits
    assert!(
      DEFAULT_TIMEOUT <= Duration::from_secs(30),
      "DEFAULT_TIMEOUT should be at most 30 seconds, got {:?}",
      DEFAULT_TIMEOUT
    );
  }

  /// Test that create_client() returns a valid client.
  #[test]
  fn test_create_client_returns_valid_client() {
    let client = create_client();
    // If we get here without panic, the client was created successfully
    assert!(std::mem::size_of_val(&client) > 0, "Client should be a valid object");
  }

  /// Test that create_client() produces a client with timeout configured.
  /// Uses a non-routable IP to verify the client times out rather than hanging.
  #[tokio::test]
  async fn test_create_client_has_timeout_configured() {
    let client = create_client();

    let start = Instant::now();
    let result = client
      .post("http://10.255.255.1:7000/local-api/message")
      .header("X-Vestaboard-Local-Api-Key", "test-key")
      .json(&[[0u8; 22]; 6])
      .send()
      .await;

    let elapsed = start.elapsed();

    // Request should fail (not succeed)
    assert!(result.is_err(), "Request to unreachable host should fail");

    // Request should complete within DEFAULT_TIMEOUT + buffer (not hang forever)
    let max_expected = DEFAULT_TIMEOUT + Duration::from_secs(5);
    assert!(elapsed < max_expected, "Request should timeout within {:?}, but took {:?}", max_expected, elapsed);

    // Verify it's a timeout or connection error
    let err = result.unwrap_err();
    assert!(err.is_timeout() || err.is_connect(), "Error should be timeout or connection error, got: {}", err);
  }

  /// Test that a short timeout actually limits connection time.
  #[tokio::test]
  async fn test_connect_timeout_is_applied() {
    let short_timeout = Duration::from_millis(500);
    let client = Client::builder()
      .connect_timeout(short_timeout)
      .build()
      .expect("Failed to build HTTP client");

    let start = Instant::now();
    let result = client.get("http://10.255.255.1:7000/").send().await;
    let elapsed = start.elapsed();

    assert!(result.is_err());
    // Should fail quickly due to connect timeout
    assert!(elapsed < Duration::from_secs(3), "Connect timeout should trigger quickly, took {:?}", elapsed);
  }

  /// Test that the error type is correctly identified as a connection/timeout error
  /// which allows proper error messaging to the user.
  #[tokio::test]
  async fn test_timeout_error_is_identifiable() {
    let client = create_client();

    let result = client.get("http://10.255.255.1:7000/").send().await;

    assert!(result.is_err());
    let err = result.unwrap_err();

    // The error should be identifiable as a connection or timeout issue
    // This is what errors.rs checks in to_user_message()
    let is_network_error = err.is_timeout() || err.is_connect();
    assert!(is_network_error, "Error should be identifiable as network error for user messaging");
  }
}
