use crate::errors::VestaboardError;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// ProcessController manages the lifecycle and graceful shutdown of long-running processes.
///
/// This controller provides a centralized way to handle:
/// - Signal management (Ctrl+C)
/// - Graceful shutdown coordination
/// - Thread-safe shutdown flag management
///
/// Following SOLID principles, this has a single responsibility for process lifecycle management
/// and can be reused across schedule, playlist, and other long-running commands.
///
/// # Examples
///
/// Basic usage in a long-running command:
/// ```rust
/// use crate::process_control::ProcessController;
/// use std::time::Duration;
/// use std::thread;
///
/// async fn long_running_command() -> Result<(), VestaboardError> {
///   // Create and setup process controller
///   let process_controller = ProcessController::new();
///   process_controller.setup_signal_handler()?;
///
///   println!("Starting long-running process...");
///
///   loop {
///     // Check for shutdown request
///     if process_controller.should_shutdown() {
///       println!("Shutting down gracefully...");
///       break;
///     }
///
///     // Do work here
///     thread::sleep(Duration::from_secs(1));
///   }
///
///   println!("Process completed.");
///   Ok(())
/// }
/// ```
///
/// Advanced usage with manual shutdown triggers:
/// ```rust
/// async fn advanced_command() -> Result<(), VestaboardError> {
///   let process_controller = ProcessController::new();
///   process_controller.setup_signal_handler()?;
///
///   let mut error_count = 0;
///
///   loop {
///     if process_controller.should_shutdown() {
///       break;
///     }
///
///     // Simulate error condition
///     if error_count > 5 {
///       println!("Too many errors, requesting shutdown");
///       process_controller.request_shutdown();
///       continue;
///     }
///
///     // Do work...
///   }
///
///   Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ProcessController {
  shutdown_flag: Arc<AtomicBool>,
  signal_handler_registered: Arc<AtomicBool>,
}

impl ProcessController {
  /// Creates a new ProcessController with default settings.
  ///
  /// The shutdown flag starts as false, and no signal handler is registered yet.
  pub fn new() -> Self {
    log::trace!("Creating new ProcessController");

    Self {
      shutdown_flag: Arc::new(AtomicBool::new(false)),
      signal_handler_registered: Arc::new(AtomicBool::new(false)),
    }
  }

  /// Sets up the Ctrl+C signal handler for graceful shutdown.
  ///
  /// This can only be called once per process due to ctrlc library limitations.
  /// Subsequent calls will return an error if already registered.
  ///
  /// # Returns
  /// - `Ok(())` if signal handler was successfully registered
  /// - `Err(VestaboardError)` if handler registration failed or already registered
  ///
  /// # Examples
  /// ```rust
  /// let controller = ProcessController::new();
  /// controller.setup_signal_handler()?;
  /// ```
  pub fn setup_signal_handler(&self) -> Result<(), VestaboardError> {
    // Check if signal handler is already registered
    if self.signal_handler_registered.load(Ordering::SeqCst) {
      let error_msg = "Signal handler already registered for this process";
      log::warn!("{}", error_msg);
      return Err(VestaboardError::Other {
        message: error_msg.to_string(),
      });
    }

    log::info!("Setting up Ctrl+C signal handler for graceful shutdown");
    println!("Press Ctrl+C to stop the process.");

    // Clone the shutdown flag for the signal handler closure
    let shutdown_flag = Arc::clone(&self.shutdown_flag);
    let signal_handler_registered = Arc::clone(&self.signal_handler_registered);

    ctrlc::set_handler(move || {
      log::info!("Ctrl+C signal received, initiating graceful shutdown");
      println!("\nCtrl+C received, shutting down gracefully...");
      shutdown_flag.store(true, Ordering::SeqCst);
    })
    .map_err(|e| {
      let error_msg = format!("Failed to set Ctrl+C handler: {}", e);
      log::error!("{}", error_msg);
      eprintln!("Error setting up signal handler: {}", e);
      VestaboardError::Other { message: error_msg }
    })?;

    // Mark signal handler as registered
    signal_handler_registered.store(true, Ordering::SeqCst);
    log::debug!("Signal handler successfully registered");

    Ok(())
  }

  /// Checks if a shutdown has been requested.
  ///
  /// This is the primary method long-running loops should call to determine
  /// if they should continue running or begin shutdown procedures.
  ///
  /// # Returns
  /// - `true` if shutdown has been requested (via Ctrl+C or manual trigger)
  /// - `false` if the process should continue running
  ///
  /// # Examples
  /// ```rust
  /// let controller = ProcessController::new();
  /// loop {
  ///   if controller.should_shutdown() {
  ///     println!("Shutting down...");
  ///     break;
  ///   }
  ///   // Do work...
  /// }
  /// ```
  pub fn should_shutdown(&self) -> bool {
    let should_shutdown = self.shutdown_flag.load(Ordering::SeqCst);
    if should_shutdown {
      log::trace!("Shutdown flag detected as true");
    }
    should_shutdown
  }

  /// Manually triggers a shutdown request.
  ///
  /// This allows programmatic shutdown requests in addition to signal-based ones.
  /// Useful for testing, error conditions, or other shutdown triggers.
  ///
  /// # Examples
  /// ```rust
  /// let controller = ProcessController::new();
  /// // Trigger shutdown due to critical error
  /// controller.request_shutdown();
  /// ```
  #[allow(dead_code)]
  pub fn request_shutdown(&self) {
    log::info!("Manual shutdown request received");
    println!("Shutdown requested programmatically.");
    self.shutdown_flag.store(true, Ordering::SeqCst);
  }

  /// Resets the shutdown flag to false.
  ///
  /// This is primarily useful for testing scenarios where you want to reuse
  /// a ProcessController instance. In production, typically a new controller
  /// would be created for each process run.
  ///
  /// Note: This does not unregister the signal handler, which cannot be undone
  /// due to ctrlc library limitations.
  #[allow(dead_code)]
  pub fn reset(&self) {
    log::debug!("Resetting shutdown flag to false");
    self.shutdown_flag.store(false, Ordering::SeqCst);
  }

  /// Returns whether the signal handler has been registered.
  ///
  /// Useful for debugging and ensuring proper initialization.
  #[allow(dead_code)]
  pub fn is_signal_handler_registered(&self) -> bool {
    self.signal_handler_registered.load(Ordering::SeqCst)
  }
}

impl Default for ProcessController {
  fn default() -> Self {
    Self::new()
  }
}
