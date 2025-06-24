use std::process::Command;

fn main() {
    // Create a simple test to see the actual output
    let output = Command::new("cargo")
        .args(&["test", "test_error_to_display_message_widget_error", "--", "--nocapture"])
        .env("RUST_BACKTRACE", "1")
        .output()
        .expect("Failed to run cargo test");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
}
