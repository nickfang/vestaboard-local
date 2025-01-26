use std::fs;
use crate::widgets::widget_utils;

pub fn get_text(text: &str) -> Vec<String> {
    match widget_utils::format_message(text) {
        Some(lines) => lines,
        None => {
            let error = vec![
                "Error:".to_string(),
                "message contains invalid characters.".to_string()
            ];
            error
        }
    }
}

pub fn get_text_from_file(file: &str) -> Vec<String> {
    match fs::read_to_string(file) {
        Ok(text) => {
            text.lines()
                .map(|line| line.to_string())
                .collect()
        }
        Err(e) => {
            eprintln!("Error reading file: {:?}", e);
            let error = vec!["Error:".to_string(), "could not read file.".to_string()];
            error
        }
    }
}
