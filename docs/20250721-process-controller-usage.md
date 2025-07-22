# ProcessController Usage Guide

The `ProcessController` provides a standardized way to handle graceful shutdown and signal management across all long-running commands in the Vestaboard application.

## Basic Usage Pattern

All long-running commands should follow this pattern:

```rust
use crate::process_control::ProcessController;
use crate::errors::VestaboardError;
use std::thread;
use std::time::Duration;

pub async fn example_long_running_command() -> Result<(), VestaboardError> {
  log::info!("Starting example long-running command");
  println!("Starting long-running process...");

  // Create and setup process controller
  let process_controller = ProcessController::new();
  process_controller.setup_signal_handler().map_err(|e| {
    eprintln!("Error setting up signal handler: {:?}", e);
    e
  })?;

  log::info!("Example command started successfully");
  println!("Process started. Processing...");

  let mut iteration_count = 0;
  let sleep_duration = Duration::from_secs(2);

  loop {
    // Check for shutdown request
    if process_controller.should_shutdown() {
      log::info!("Shutdown request detected, stopping example command");
      println!("Example command shutting down...");
      break;
    }

    // Do some work
    iteration_count += 1;
    log::debug!("Example command iteration {}", iteration_count);
    println!("Processing iteration {}...", iteration_count);

    // Simulate work
    thread::sleep(sleep_duration);

    // Example: Stop after 10 iterations for demo purposes
    if iteration_count >= 10 {
      log::info!("Example command completed 10 iterations, requesting shutdown");
      process_controller.request_shutdown();
    }
  }

  log::info!("Example command shutdown complete");
  println!("Example command finished.");
  Ok(())
}
```

## Integration with main.rs

When adding a new long-running command to the CLI, integrate it like this:

```rust
// In main.rs
Command::Cycle => {
  log::info!("Starting cycle mode");
  match example_long_running_command().await {
    Ok(_) => log::info!("Cycle completed successfully"),
    Err(e) => {
      log::error!("Cycle failed: {}", e);
      eprintln!("Cycle error: {}", e);
    }
  }
}
```

## Key Points

1. **Always call `setup_signal_handler()`** early in your function
2. **Check `should_shutdown()`** in your main loop
3. **Use programmatic shutdown** with `request_shutdown()` for error conditions
4. **Follow the 3-tier logging pattern**: `log::` for files, `println!` for console, `display_message()` for Vestaboard
5. **Handle setup errors gracefully** by propagating them up

## ProcessController API

### Core Methods

- `ProcessController::new()` - Create a new controller
- `setup_signal_handler()` - Register Ctrl+C handler (call once per process)
- `should_shutdown()` - Check if shutdown was requested
- `request_shutdown()` - Manually trigger shutdown
- `reset()` - Reset shutdown flag (mainly for testing)

### Thread Safety

The ProcessController is thread-safe and can be cloned to share shutdown state across threads:

```rust
let controller = ProcessController::new();
let controller_clone = controller.clone();

// Both controllers share the same shutdown state
controller.request_shutdown();
assert!(controller_clone.should_shutdown());
```

## Examples of Long-Running Commands

### Daemon Mode
Already implemented in `src/daemon.rs` - monitors schedule file and executes tasks.

### Future Cycle Mode
Could be implemented to repeatedly execute a sequence of widgets on a timer.

### Future Watch Mode  
Could monitor file changes and automatically update the Vestaboard.

All of these should use the ProcessController for consistent shutdown behavior.
