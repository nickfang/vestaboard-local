use std::{ fs, path::PathBuf };
use crate::widgets::widget_utils;
use crate::errors::VestaboardError;

pub fn get_text(text: &str) -> Result<Vec<String>, VestaboardError> {
    match widget_utils::format_message(text) {
        Some(lines) => Ok(lines),
        None => Err(VestaboardError::widget_error("text", "Message contains invalid characters")),
    }
}

pub fn get_text_from_file(file: PathBuf) -> Result<Vec<String>, VestaboardError> {
    match fs::read_to_string(&file) {
        Ok(text) => {
            Ok(
                text
                    .lines()
                    .map(|line| line.to_string())
                    .collect()
            )
        }
        Err(e) => {
            eprintln!("Error reading file: {:?}", e);
            Err(VestaboardError::io_error(e, &format!("reading text file {}", file.display())))
        }
    }
}
