// Quick test to see error formatting
use std::path::Path;

fn main() {
    // Include the necessary modules
    println!("Testing error format...");
    
    // Let's just show what the current format looks like
    let header = "api error";
    let centered_header = format!("{:^22}", header);
    let red_line = "R R R R R R R R R R R";
    let empty_line = "";
    let content = "service temporarily down";
    let centered_content = format!("{:^22}", content);
    
    println!("Line 1: '{}'", centered_header);
    println!("Line 2: '{}'", red_line);
    println!("Line 3: '{}'", empty_line);
    println!("Line 4: '{}'", centered_content);
    println!();
    println!("Total lines used: 4 out of 6 available");
    println!("Lines available for content: 3 (if we keep empty line) or 4 (if we remove it)");
}
