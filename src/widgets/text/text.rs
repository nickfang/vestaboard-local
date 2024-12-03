use std::fs;
use crate::widgets::widget_utils;

pub fn get_text(text: &str) -> Vec<String> {
    match widget_utils::format_message(text) {
        None => {
            println!("Error: message contains invalid characters.");
            Vec::new()
        }
        Some(lines) => lines,
    }
}

pub fn get_text_from_file(file: &str) -> Vec<String> {
    let text = fs::read_to_string(file).expect("Unable to read file");
    text.lines()
        .map(|line| line.to_string())
        .collect()
}
