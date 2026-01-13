use crate::errors::VestaboardError;
use crate::widgets::widget_utils;
use std::{fs, path::PathBuf};

pub fn get_text(text: &str) -> Result<Vec<String>, VestaboardError> {
  log::debug!("Text widget starting with {} characters", text.len());
  // Widget just formats the message - validation happens at the main level
  let formatted = widget_utils::format_message(text);
  log::debug!("Text widget completed successfully, {} lines generated", formatted.len());
  Ok(formatted)
}

pub fn get_text_from_file(file: PathBuf) -> Result<Vec<String>, VestaboardError> {
  log::debug!("File widget starting, reading from: {}", file.display());
  match fs::read_to_string(&file) {
    Ok(text) => {
      log::info!("Successfully read {} characters from file: {}", text.len(), file.display());
      let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
      log::debug!("File widget completed successfully, {} lines read", lines.len());
      Ok(lines)
    },
    Err(e) => {
      log::error!("Failed to read file {}: {}", file.display(), e);
      let error = VestaboardError::io_error(e, &format!("reading file {}", file.display()));
      Err(error)
    },
  }
}
