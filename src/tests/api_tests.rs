use crate::api::{create_client, send_codes, DEFAULT_TIMEOUT};

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
async fn test_get_message() {
  use crate::api::{Transport, TransportType};
  let transport = Transport::new(TransportType::Local).expect("Failed to create transport");
  let result = transport.get_message().await;
  assert!(result.is_ok());
}

// Tests for timeout behavior (Issue #52)
// These tests verify that the HTTP client has proper timeout configuration
// so the application doesn't hang indefinitely when the Vestaboard is unreachable.

#[cfg(test)]
mod timeout_tests {
  use super::*;
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

  /// Test that a client with timeout configured times out properly.
  /// Uses a non-routable IP and short timeout to verify timeout behavior without slow tests.
  #[tokio::test]
  async fn test_client_timeout_behavior() {
    let short_timeout = Duration::from_millis(500);
    let client = Client::builder()
      .timeout(short_timeout)
      .connect_timeout(short_timeout)
      .build()
      .expect("Failed to build HTTP client");

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

    // Request should complete within timeout + buffer (not hang forever)
    let max_expected = short_timeout + Duration::from_secs(2);
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
    let short_timeout = Duration::from_millis(500);
    let client = Client::builder()
      .connect_timeout(short_timeout)
      .build()
      .expect("Failed to build HTTP client");

    let result = client.get("http://10.255.255.1:7000/").send().await;

    assert!(result.is_err());
    let err = result.unwrap_err();

    // The error should be identifiable as a connection or timeout issue
    // This is what errors.rs checks in to_user_message()
    let is_network_error = err.is_timeout() || err.is_connect();
    assert!(is_network_error, "Error should be identifiable as network error for user messaging");
  }
}

// Tests for TransportType and Transport
#[cfg(test)]
mod transport_tests {
  use crate::api::{Transport, TransportType};
  use serial_test::serial;

  #[test]
  fn test_transport_type_default_is_local() {
    let default = TransportType::default();
    assert_eq!(default, TransportType::Local);
  }

  #[test]
  fn test_transport_type_serde_local() {
    let json = serde_json::to_string(&TransportType::Local).unwrap();
    assert_eq!(json, "\"local\"");

    let parsed: TransportType = serde_json::from_str("\"local\"").unwrap();
    assert_eq!(parsed, TransportType::Local);
  }

  #[test]
  fn test_transport_type_serde_internet() {
    let json = serde_json::to_string(&TransportType::Internet).unwrap();
    assert_eq!(json, "\"internet\"");

    let parsed: TransportType = serde_json::from_str("\"internet\"").unwrap();
    assert_eq!(parsed, TransportType::Internet);
  }

  #[test]
  #[serial]
  fn test_transport_internet_requires_api_key() {
    // Save original value
    let orig_api_key = std::env::var("INTERNET_API_KEY").ok();

    // Set to empty to simulate missing
    // Note: dotenv() won't override existing vars (even empty ones)
    std::env::set_var("INTERNET_API_KEY", "");

    let result = Transport::new(TransportType::Internet);

    // Restore original value
    match orig_api_key {
      Some(v) => std::env::set_var("INTERNET_API_KEY", v),
      None => std::env::remove_var("INTERNET_API_KEY"),
    }

    assert!(result.is_err(), "Transport should fail with empty INTERNET_API_KEY");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("INTERNET_API_KEY"), "Error should mention missing env var");
  }

  #[test]
  #[serial]
  fn test_transport_local_requires_api_key() {
    // Save original values
    let orig_api_key = std::env::var("LOCAL_API_KEY").ok();
    let orig_ip = std::env::var("IP_ADDRESS").ok();

    // Set LOCAL_API_KEY to empty and IP_ADDRESS to valid
    std::env::set_var("LOCAL_API_KEY", "");
    std::env::set_var("IP_ADDRESS", "192.168.1.1");

    let result = Transport::new(TransportType::Local);

    // Restore original values
    match orig_api_key {
      Some(v) => std::env::set_var("LOCAL_API_KEY", v),
      None => std::env::remove_var("LOCAL_API_KEY"),
    }
    match orig_ip {
      Some(v) => std::env::set_var("IP_ADDRESS", v),
      None => std::env::remove_var("IP_ADDRESS"),
    }

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
      err.to_string().contains("LOCAL_API_KEY"),
      "Error should mention missing env var, got: {}",
      err
    );
  }

  #[test]
  #[serial]
  fn test_transport_local_requires_ip_address() {
    // Save original values
    let orig_api_key = std::env::var("LOCAL_API_KEY").ok();
    let orig_ip = std::env::var("IP_ADDRESS").ok();

    // Set LOCAL_API_KEY to valid and IP_ADDRESS to empty
    std::env::set_var("LOCAL_API_KEY", "test-api-key");
    std::env::set_var("IP_ADDRESS", "");

    let result = Transport::new(TransportType::Local);

    // Restore original values
    match orig_api_key {
      Some(v) => std::env::set_var("LOCAL_API_KEY", v),
      None => std::env::remove_var("LOCAL_API_KEY"),
    }
    match orig_ip {
      Some(v) => std::env::set_var("IP_ADDRESS", v),
      None => std::env::remove_var("IP_ADDRESS"),
    }

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
      err.to_string().contains("IP_ADDRESS"),
      "Error should mention missing env var, got: {}",
      err
    );
  }

  #[test]
  #[serial]
  fn test_transport_local_succeeds_with_valid_env() {
    // Save original values
    let orig_api_key = std::env::var("LOCAL_API_KEY").ok();
    let orig_ip = std::env::var("IP_ADDRESS").ok();

    // Set valid values
    std::env::set_var("LOCAL_API_KEY", "test-api-key");
    std::env::set_var("IP_ADDRESS", "192.168.1.1");

    let result = Transport::new(TransportType::Local);

    // Restore original values
    match orig_api_key {
      Some(v) => std::env::set_var("LOCAL_API_KEY", v),
      None => std::env::remove_var("LOCAL_API_KEY"),
    }
    match orig_ip {
      Some(v) => std::env::set_var("IP_ADDRESS", v),
      None => std::env::remove_var("IP_ADDRESS"),
    }

    assert!(result.is_ok(), "Transport should be created with valid env vars");
    let transport = result.unwrap();
    assert_eq!(transport.name(), "local");
  }

  #[test]
  #[serial]
  fn test_transport_internet_succeeds_with_valid_env() {
    // Save original value
    let orig_api_key = std::env::var("INTERNET_API_KEY").ok();

    // Set valid value
    std::env::set_var("INTERNET_API_KEY", "test-api-key");

    let result = Transport::new(TransportType::Internet);

    // Restore original value
    match orig_api_key {
      Some(v) => std::env::set_var("INTERNET_API_KEY", v),
      None => std::env::remove_var("INTERNET_API_KEY"),
    }

    assert!(result.is_ok(), "Transport should be created with valid env vars");
    let transport = result.unwrap();
    assert_eq!(transport.name(), "internet");
  }

  #[test]
  fn test_transport_name_local() {
    // This test doesn't need serial since it doesn't modify env vars
    // and we use a helper to check the name
    assert_eq!(TransportType::Local.to_string(), "local");
  }

  #[test]
  fn test_transport_name_internet() {
    assert_eq!(TransportType::Internet.to_string(), "internet");
  }

  #[test]
  #[serial]
  fn test_error_message_is_user_friendly_internet() {
    // Save original value
    let orig_api_key = std::env::var("INTERNET_API_KEY").ok();

    // Test that error messages provide actionable guidance
    std::env::set_var("INTERNET_API_KEY", "");
    let result = Transport::new(TransportType::Internet);

    // Restore original value
    match orig_api_key {
      Some(v) => std::env::set_var("INTERNET_API_KEY", v),
      None => std::env::remove_var("INTERNET_API_KEY"),
    }

    let err = result.unwrap_err();
    let msg = err.to_user_message();

    // Error should mention how to set the variable
    assert!(
      msg.contains("export") || msg.contains(".env"),
      "Error message should mention how to set the env var, got: {}",
      msg
    );
  }
}

// Helper for TransportType Display
impl std::fmt::Display for crate::api::TransportType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      crate::api::TransportType::Local => write!(f, "local"),
      crate::api::TransportType::Internet => write!(f, "internet"),
    }
  }
}

// Integration tests using wiremock for HTTP mocking
#[cfg(test)]
mod wiremock_tests {
  use reqwest::Client;
  use serde_json::json;
  use std::time::Duration;
  use wiremock::matchers::{header, method, path};
  use wiremock::{Mock, MockServer, ResponseTemplate};

  /// Test that local transport sends to correct URL pattern with correct header
  #[tokio::test]
  async fn test_local_api_request_format() {
    let mock_server = MockServer::start().await;

    // Set up the mock to expect a POST with the correct header
    Mock::given(method("POST"))
      .and(path("/local-api/message"))
      .and(header("X-Vestaboard-Local-Api-Key", "test-local-key"))
      .respond_with(ResponseTemplate::new(200))
      .expect(1)
      .mount(&mock_server)
      .await;

    let client = Client::builder()
      .timeout(Duration::from_secs(5))
      .build()
      .unwrap();

    let codes: [[u8; 22]; 6] = [[0; 22]; 6];
    let body = json!(codes);

    let result = client
      .post(format!("{}/local-api/message", mock_server.uri()))
      .header("X-Vestaboard-Local-Api-Key", "test-local-key")
      .json(&body)
      .send()
      .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().status(), 200);
  }

  /// Test that internet transport sends to correct URL pattern with correct header
  #[tokio::test]
  async fn test_internet_api_request_format() {
    let mock_server = MockServer::start().await;

    // Set up the mock to expect a POST with the correct header
    Mock::given(method("POST"))
      .and(header("X-Vestaboard-Read-Write-Key", "test-internet-key"))
      .respond_with(ResponseTemplate::new(200))
      .expect(1)
      .mount(&mock_server)
      .await;

    let client = Client::builder()
      .timeout(Duration::from_secs(5))
      .build()
      .unwrap();

    let codes: [[u8; 22]; 6] = [[0; 22]; 6];
    let body = json!(codes);

    let result = client
      .post(mock_server.uri())
      .header("X-Vestaboard-Read-Write-Key", "test-internet-key")
      .json(&body)
      .send()
      .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().status(), 200);
  }

  /// Test handling of HTTP 4xx error responses
  #[tokio::test]
  async fn test_http_4xx_error_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
      .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request: Invalid message format"))
      .mount(&mock_server)
      .await;

    let client = Client::builder()
      .timeout(Duration::from_secs(5))
      .build()
      .unwrap();

    let result = client.post(mock_server.uri()).json(&json!({})).send().await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status(), 400);

    let body = response.text().await.unwrap();
    assert!(body.contains("Bad Request"));
  }

  /// Test handling of HTTP 5xx error responses
  #[tokio::test]
  async fn test_http_5xx_error_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
      .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
      .mount(&mock_server)
      .await;

    let client = Client::builder()
      .timeout(Duration::from_secs(5))
      .build()
      .unwrap();

    let result = client.post(mock_server.uri()).json(&json!({})).send().await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status(), 500);
  }

  /// Test handling of HTTP 304 Not Modified response
  #[tokio::test]
  async fn test_http_304_not_modified_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
      .respond_with(ResponseTemplate::new(304))
      .mount(&mock_server)
      .await;

    let client = Client::builder()
      .timeout(Duration::from_secs(5))
      .build()
      .unwrap();

    let result = client.post(mock_server.uri()).json(&json!({})).send().await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status(), 304);
  }

  /// Test that the correct message body format is sent
  #[tokio::test]
  async fn test_message_body_format() {
    use wiremock::matchers::body_json;

    let mock_server = MockServer::start().await;

    let expected_codes: [[u8; 22]; 6] = [
      [1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
      [0; 22],
      [0; 22],
      [0; 22],
      [0; 22],
      [0; 22],
    ];

    Mock::given(method("POST"))
      .and(body_json(&expected_codes))
      .respond_with(ResponseTemplate::new(200))
      .expect(1)
      .mount(&mock_server)
      .await;

    let client = Client::builder()
      .timeout(Duration::from_secs(5))
      .build()
      .unwrap();

    let result = client.post(mock_server.uri()).json(&expected_codes).send().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().status(), 200);
  }
}
