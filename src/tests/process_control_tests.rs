#[cfg(test)]
mod tests {
  use crate::process_control::ProcessController;

  #[test]
  fn test_new_controller_default_state() {
    let controller = ProcessController::new();

    assert!(!controller.should_shutdown());
    assert!(!controller.is_signal_handler_registered());
  }

  #[test]
  fn test_manual_shutdown_request() {
    let controller = ProcessController::new();

    assert!(!controller.should_shutdown());

    controller.request_shutdown();
    assert!(controller.should_shutdown());
  }

  #[test]
  fn test_reset_shutdown_flag() {
    let controller = ProcessController::new();

    controller.request_shutdown();
    assert!(controller.should_shutdown());

    controller.reset();
    assert!(!controller.should_shutdown());
  }

  #[test]
  fn test_clone_shares_state() {
    let controller = ProcessController::new();
    let cloned_controller = controller.clone();

    controller.request_shutdown();
    assert!(cloned_controller.should_shutdown());

    cloned_controller.reset();
    assert!(!controller.should_shutdown());
  }

  // Note: Signal handler testing is challenging due to the global nature
  // of signal handlers and ctrlc library limitations. In a real application,
  // integration tests would be more appropriate for this functionality.
}
