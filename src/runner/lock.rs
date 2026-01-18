//! Instance lock for preventing multiple concurrent executions.
//!
//! This module provides a lock mechanism to ensure only one instance of
//! schedule or playlist execution can run at a time.
//!
//! # Implementation Notes
//!
//! This implementation uses a JSON lock file with PID checking. While not
//! atomic (there's a small race window between checking and acquiring),
//! it's sufficient for the single-user CLI use case.
//!
//! For production use with multiple concurrent access attempts, consider
//! upgrading to OS-level file locking (flock on Unix, LockFileEx on Windows).

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::VestaboardError;

/// Data stored in the lock file
#[derive(Debug, Serialize, Deserialize)]
struct LockData {
  /// What mode is running ("playlist" or "schedule")
  mode: String,
  /// Process ID of the lock holder
  pid: u32,
  /// When the lock was acquired
  started_at: DateTime<Utc>,
}

/// An exclusive lock that prevents multiple instances from running.
///
/// The lock is automatically released when dropped (RAII pattern).
#[derive(Debug)]
pub struct InstanceLock {
  path: PathBuf,
}

impl InstanceLock {
  /// Acquire an exclusive lock at the default location.
  ///
  /// # Arguments
  /// * `mode` - The mode name ("playlist" or "schedule") for error messages
  ///
  /// # Returns
  /// * `Ok(InstanceLock)` - Lock acquired successfully
  /// * `Err(VestaboardError)` - Lock could not be acquired (another instance running)
  pub fn acquire(mode: &str) -> Result<Self, VestaboardError> {
    Self::acquire_at(mode, &PathBuf::from("data/vestaboard.lock"))
  }

  /// Acquire an exclusive lock at a specific path (useful for testing).
  ///
  /// # Arguments
  /// * `mode` - The mode name ("playlist" or "schedule") for error messages
  /// * `path` - Path to the lock file
  ///
  /// # Returns
  /// * `Ok(InstanceLock)` - Lock acquired successfully
  /// * `Err(VestaboardError)` - Lock could not be acquired
  pub fn acquire_at(mode: &str, path: &PathBuf) -> Result<Self, VestaboardError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)
        .map_err(|e| VestaboardError::lock_error(&format!("Cannot create lock directory: {}", e)))?;
    }

    // Check if lock file exists and is held by a running process
    if path.exists() {
      match fs::read_to_string(path) {
        Ok(content) => {
          if let Ok(lock_data) = serde_json::from_str::<LockData>(&content) {
            // Check if the PID is still running
            if is_pid_running(lock_data.pid) {
              return Err(VestaboardError::lock_error(&format!(
                "{} already running (PID {}, started {})",
                lock_data.mode,
                lock_data.pid,
                lock_data.started_at.format("%H:%M:%S")
              )));
            }
            // PID is not running, lock is stale
            log::info!("Stale lock detected (PID {} not running), taking over", lock_data.pid);
          }
          // Invalid JSON, treat as stale
        },
        Err(e) => {
          log::warn!("Cannot read lock file, treating as stale: {}", e);
        },
      }
    }

    // Write our lock data
    let lock_data = LockData {
      mode: mode.to_string(),
      pid: std::process::id(),
      started_at: Utc::now(),
    };

    let content = serde_json::to_string_pretty(&lock_data)
      .map_err(|e| VestaboardError::lock_error(&format!("Cannot serialize lock: {}", e)))?;

    let mut file =
      File::create(path).map_err(|e| VestaboardError::lock_error(&format!("Cannot create lock file: {}", e)))?;

    file
      .write_all(content.as_bytes())
      .map_err(|e| VestaboardError::lock_error(&format!("Cannot write lock file: {}", e)))?;

    log::info!("Lock acquired for {} at {}", mode, path.display());

    Ok(Self { path: path.clone() })
  }

  /// Get the path to the lock file
  #[allow(dead_code)]
  pub fn path(&self) -> &PathBuf {
    &self.path
  }
}

impl Drop for InstanceLock {
  fn drop(&mut self) {
    // Remove the lock file when the lock is released
    if let Err(e) = fs::remove_file(&self.path) {
      // Only warn if file exists but couldn't be removed
      if e.kind() != std::io::ErrorKind::NotFound {
        log::warn!("Cannot remove lock file: {}", e);
      }
    } else {
      log::info!("Lock released at {}", self.path.display());
    }
  }
}

/// Check if a process with the given PID is running.
///
/// This is a cross-platform implementation that attempts to detect if a process exists.
fn is_pid_running(pid: u32) -> bool {
  #[cfg(unix)]
  {
    // On Unix, sending signal 0 checks if process exists without affecting it
    // Returns 0 if process exists and we have permission to signal it
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
  }

  #[cfg(windows)]
  {
    // On Windows, try to open the process with minimal permissions
    use std::ptr::null_mut;
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

    let handle = unsafe {
      winapi::um::processthreadsapi::OpenProcess(
        PROCESS_QUERY_LIMITED_INFORMATION,
        0, // don't inherit handle
        pid,
      )
    };

    if handle.is_null() {
      false
    } else {
      unsafe { winapi::um::handleapi::CloseHandle(handle) };
      true
    }
  }

  #[cfg(not(any(unix, windows)))]
  {
    // On other platforms, assume the process is not running (fail open)
    // This allows lock acquisition but may allow concurrent instances
    log::warn!("Cannot check PID on this platform, assuming process not running");
    false
  }
}
