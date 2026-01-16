//! API transport layer for Vestaboard communication.
//!
//! This module provides different transports for sending messages to Vestaboard:
//! - `LocalTransport` - Uses the local network API (requires same network as device)
//! - `InternetTransport` - Uses the Read/Write API over the internet
//!
//! # Architecture
//!
//! ```text
//! Widgets → api_broker (Translation) → api (Transport) → Vestaboard
//! ```

pub mod common;
pub mod internet;
pub mod local;

use crate::errors::VestaboardError;
use serde::{Deserialize, Serialize};

pub use internet::InternetTransport;
pub use local::LocalTransport;

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
  /// Internet transport via Read/Write API
  Internet(InternetTransport),
}

impl Transport {
  /// Create a new transport of the specified type.
  ///
  /// # Errors
  /// Returns an error if required environment variables are not set for the transport.
  pub fn new(transport_type: TransportType) -> Result<Self, VestaboardError> {
    let transport = match transport_type {
      TransportType::Local => Transport::Local(LocalTransport::new()?),
      TransportType::Internet => Transport::Internet(InternetTransport::new()?),
    };
    log::info!("Created {} transport", transport.name());
    Ok(transport)
  }

  /// Send character codes to the Vestaboard.
  pub async fn send_codes(&self, codes: [[u8; 22]; 6]) -> Result<(), VestaboardError> {
    log::debug!("Sending codes via {} transport", self.name());
    match self {
      Transport::Local(t) => t.send_codes(codes).await,
      Transport::Internet(t) => t.send_codes(codes).await,
    }
  }

  /// Get the current message displayed on the Vestaboard.
  ///
  /// Note: This method is kept for future features but is not yet fully implemented.
  /// The return type should eventually return the actual message data.
  pub async fn get_message(&self) -> Result<(), VestaboardError> {
    log::debug!("Getting message via {} transport", self.name());
    match self {
      Transport::Local(t) => t.get_message().await,
      Transport::Internet(t) => t.get_message().await,
    }
  }

  /// Get the name of this transport for logging.
  pub fn name(&self) -> &'static str {
    match self {
      Transport::Local(_) => "local",
      Transport::Internet(_) => "internet",
    }
  }
}

impl std::fmt::Debug for Transport {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Transport::Local(_) => write!(f, "Transport::Local"),
      Transport::Internet(_) => write!(f, "Transport::Internet"),
    }
  }
}
