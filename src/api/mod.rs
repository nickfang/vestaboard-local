//! API transport layer for Vestaboard communication.
//!
//! This module provides different transports for sending messages to Vestaboard:
//! - `LocalTransport` - Uses the local network API (requires same network as device)
//! - `InternetTransport` - Uses the Read/Write API over the internet (future)
//!
//! # Architecture
//!
//! ```text
//! Widgets → api_broker (Translation) → api (Transport) → Vestaboard
//! ```

pub mod common;
pub mod local;

use crate::errors::VestaboardError;
use serde::{Deserialize, Serialize};

pub use common::{create_client, DEFAULT_TIMEOUT};
pub use local::{get_message, LocalTransport};

/// Transport type for configuration and CLI selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
  /// Local network transport (default)
  #[default]
  Local,
  /// Internet transport via Read/Write API
  Internet,
}

/// Transport for sending messages to Vestaboard.
///
/// Uses enum dispatch pattern to avoid async_trait dependency while
/// providing polymorphism for different transport implementations.
pub enum Transport {
  /// Local network transport
  Local(LocalTransport),
  // InternetTransport will be added in issue #92
}

impl Transport {
  /// Create a new transport of the specified type.
  ///
  /// # Errors
  /// Returns an error if required environment variables are not set for the transport.
  pub fn new(transport_type: TransportType) -> Result<Self, VestaboardError> {
    match transport_type {
      TransportType::Local => Ok(Transport::Local(LocalTransport::new()?)),
      TransportType::Internet => {
        // InternetTransport will be implemented in issue #92
        Err(VestaboardError::other(
          "Internet transport not yet implemented. This will be available in a future release.",
        ))
      },
    }
  }

  /// Send character codes to the Vestaboard.
  pub async fn send_codes(&self, codes: [[u8; 22]; 6]) -> Result<(), VestaboardError> {
    match self {
      Transport::Local(t) => t.send_codes(codes).await,
    }
  }

  /// Get the name of this transport for logging.
  pub fn name(&self) -> &'static str {
    match self {
      Transport::Local(_) => "local",
    }
  }
}

impl std::fmt::Debug for Transport {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Transport::Local(_) => write!(f, "Transport::Local"),
    }
  }
}

// ============================================================================
// Backward compatibility layer
// These functions maintain the existing API until callers are updated in #93/#95
// ============================================================================

/// Send character codes to the Vestaboard using local transport.
///
/// This is a convenience function that maintains backward compatibility.
/// For new code, prefer using `Transport::new()` and `transport.send_codes()`.
pub async fn send_codes(codes: [[u8; 22]; 6]) -> Result<(), VestaboardError> {
  // Use a static LocalTransport to maintain connection pooling behavior
  use once_cell::sync::Lazy;
  static LOCAL_TRANSPORT: Lazy<LocalTransport> = Lazy::new(|| {
    LocalTransport::new().expect("Failed to initialize local transport")
  });

  LOCAL_TRANSPORT.send_codes(codes).await
}
