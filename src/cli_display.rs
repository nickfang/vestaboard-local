use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

// Global state for output control
static QUIET_MODE: AtomicBool = AtomicBool::new(false);
static TTY_DETECTED: AtomicBool = AtomicBool::new(false);

/// Initialize output control settings
pub fn init_output_control(quiet: bool, _verbose: bool) {
  QUIET_MODE.store(quiet, Ordering::Relaxed);
  TTY_DETECTED.store(std::io::stdout().is_terminal(), Ordering::Relaxed);
}

/// Check if we should print progress messages
fn should_print_progress() -> bool {
  !QUIET_MODE.load(Ordering::Relaxed) && TTY_DETECTED.load(Ordering::Relaxed)
}

/// Truncate long messages with ellipsis
fn truncate_message(msg: &str, max_len: usize) -> String {
  if msg.len() <= max_len {
    return msg.to_string();
  }
  if max_len <= 3 {
    return "...".to_string();
  }
  format!("{}...", &msg[..max_len - 3])
}

/// Print a success message with checkmark prefix
pub fn print_success(msg: &str) {
  if QUIET_MODE.load(Ordering::Relaxed) {
    return;
  }
  let truncated = truncate_message(msg, 200);
  println!("✓ {}", truncated);
}

/// Print an error message with cross prefix
/// Errors are always printed, even in quiet mode
pub fn print_error(msg: &str) {
  let truncated = truncate_message(msg, 200);
  eprintln!("✗ {}", truncated);
}

/// Print a progress message (action in progress)
pub fn print_progress(msg: &str) {
  if !should_print_progress() {
    return;
  }
  let truncated = truncate_message(msg, 200);
  println!("{}", truncated);
}

/// Print a warning message with warning prefix
pub fn print_warning(msg: &str) {
  if QUIET_MODE.load(Ordering::Relaxed) {
    return;
  }
  let truncated = truncate_message(msg, 200);
  eprintln!("⚠ {}", truncated);
}

pub fn print_message(message: Vec<String>, title: &str) -> Vec<String> {
  let mut output = Vec::new();
  if title == "" {
    output.push("Vestaboard Display:".to_string());
  } else {
    output.push(format!("{}", title));
  }
  output.push("|----------------------|".to_string());
  message.iter().take(6).for_each(|line| {
    let padded_line = format!("{:<22}", line);
    const SOLID_SQUARE: char = '\u{2588}';
    let modified_line = padded_line
      .chars()
      .map(|c| match c {
        'D' => "°".to_string(),
        'R' => format!("\x1b[{}m{}\x1b[0m", "31", SOLID_SQUARE),
        'O' => format!("\x1b[{}m{}\x1b[0m", "38:5:208", SOLID_SQUARE),
        'Y' => format!("\x1b[{}m{}\x1b[0m", "33", SOLID_SQUARE),
        'G' => format!("\x1b[{}m{}\x1b[0m", "32", SOLID_SQUARE),
        'B' => format!("\x1b[{}m{}\x1b[0m", "34", SOLID_SQUARE),
        'V' => format!("\x1b[{}m{}\x1b[0m", "35", SOLID_SQUARE),
        'W' => format!("\x1b[{}m{}\x1b[0m", "37", SOLID_SQUARE),
        'K' => format!("\x1b[{}m{}\x1b[0m", "30", SOLID_SQUARE),
        _ => c.to_string(),
      })
      .collect::<String>();
    output.push(format!("|{}|", modified_line));
  });
  // Make sure display matches the Vestaboard.
  // Handle if the messag is less than 6 lines.
  while output.len() < 8 {
    output.push("|                      |".to_string());
  }
  output.push("|----------------------|".to_string());
  output.iter().for_each(|line| println!("{}", line));
  output
}
