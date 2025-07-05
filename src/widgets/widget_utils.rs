use crate::errors::VestaboardError;

pub type WidgetOutput = Vec<String>;
pub const MAX_MESSAGE_LENGTH: usize = 22;
pub const MAX_MESSAGE_HEIGHT: usize = 6;

pub fn full_justify_line(s1: String, s2: String) -> String {
  let len1 = s1.chars().count();
  let len2 = s2.chars().count();
  let mut padding = 1;
  if len1 + len2 < MAX_MESSAGE_LENGTH {
    padding = MAX_MESSAGE_LENGTH - len1 - len2;
  }
  return format!("{}{:padding$}{}", s1, "", s2);
}

pub fn split_into_lines(text: &str) -> WidgetOutput {
  let mut formatted_message: Vec<String> = Vec::new();
  let words: Vec<&str> = text.split_whitespace().collect();
  let mut current_line = String::new();

  for word in words {
    if word.len() > MAX_MESSAGE_LENGTH {
      let mut split_word = word.to_string();
      while !split_word.is_empty() {
        let split_index = split_word
          .char_indices()
          .nth(MAX_MESSAGE_LENGTH)
          .map(|(i, _)| i)
          .unwrap_or(split_word.len());
        let split = split_word.split_off(split_index);
        formatted_message.push(split_word);
        split_word = split;
      }
      continue;
    }
    if current_line.len() + word.len() + 1 > MAX_MESSAGE_LENGTH {
      // if next word doesn't fit, add to formatted_message
      formatted_message.push(current_line);
      current_line = String::new();
    }

    if !current_line.is_empty() {
      // add space between words
      current_line.push(' ');
    }
    current_line.push_str(word);
  }

  if !current_line.is_empty() {
    formatted_message.push(current_line);
  }

  formatted_message
}

pub fn center_line(line: String) -> String {
  format!("{:^1$}", line, MAX_MESSAGE_LENGTH)
  // let half_padding = (22 - line.len()) / 2;
  // if half_padding > 0 {
  //     format!("{}{}", " ".repeat(half_padding), line)
  // } else {
  //     line
  // }
}

pub fn center_message(mut message: Vec<String>, height: usize) -> WidgetOutput {
  if message.len() < height {
    let half_padding = (height - message.len()) / 2;
    for _ in 0..half_padding {
      message.insert(0, String::new());
    }
    while message.len() < height {
      message.push(String::new());
    }
  }
  message
}

pub fn format_message(message: &str) -> WidgetOutput {
  // Widget just formats the message - validation happens at the main level
  let mut formatted_message: Vec<String> = Vec::new();
  split_into_lines(message).iter().for_each(|line| {
    formatted_message.push(center_line(line.to_string()));
  });
  center_message(formatted_message, MAX_MESSAGE_HEIGHT)
}

// There is only room for 4 lines of error message on the Vestaboard
pub fn format_error(error: &str) -> WidgetOutput {
  format_error_with_header(error, "error")
}

pub fn format_error_with_header(error: &str, header: &str) -> WidgetOutput {
  let mut formatted_message: Vec<String> = Vec::new();
  let lowercase_error = error.to_lowercase();
  let words: Vec<&str> = lowercase_error.split_whitespace().collect();
  let mut current_line = String::new();
  let mut content_lines: Vec<String> = Vec::new();

  // Build content lines first
  for word in words {
    if current_line.len() + word.len() + 1 > MAX_MESSAGE_LENGTH {
      let padded_line = center_line(current_line);
      content_lines.push(padded_line);
      current_line = String::new();
    }
    if !current_line.is_empty() {
      current_line.push(' ');
    }
    current_line.push_str(word);
  }

  if !current_line.is_empty() {
    content_lines.push(center_line(current_line));
  }

  // Center content within 4 available lines (6 total - 2 header lines)
  let centered_content = center_message(content_lines, 4);

  // Create final message: header + red line + centered content
  formatted_message.push(center_line(header.to_lowercase()));
  formatted_message.push("R R R R R R R R R R R".to_string());
  formatted_message.extend(centered_content);

  formatted_message
}

/// Converts a VestaboardError to a display message for the Vestaboard
pub fn error_to_display_message(error: &VestaboardError) -> Vec<String> {
  match error {
    VestaboardError::IOError { context, .. } => {
      // Extract more meaningful info from the context
      if context.contains("reading") && context.contains("file") {
        // Try to extract filename
        let parts: Vec<&str> = context.split(' ').collect();
        if let Some(filename) = parts.last() {
          format_error_with_header(&format!("'{}' not found", filename), "file error")
        } else {
          format_error_with_header("File not found", "file error")
        }
      } else if context.contains("creating") || context.contains("writing") {
        format_error_with_header("Cannot write file", "file error")
      } else {
        format_error_with_header("File operation failed", "file error")
      }
    },
    VestaboardError::JsonError { context, .. } => {
      if context.contains("parsing") {
        format_error_with_header("Invalid data format", "data error")
      } else {
        format_error_with_header("Data processing error", "data error")
      }
    },
    VestaboardError::ReqwestError { context, .. } => {
      if context.contains("weather") {
        format_error_with_header("Weather service unavailable", "network error")
      } else {
        format_error_with_header("Network error", "network error")
      }
    },
    VestaboardError::WidgetError { widget, message: _ } => match widget.as_str() {
      "weather" => format_error_with_header("Weather data unavailable", "widget error"),
      "text" => format_error_with_header("Text processing error", "widget error"),
      "sat-word" => format_error_with_header("Dictionary unavailable", "widget error"),
      _ => format_error_with_header(&format!("{} error", widget), "widget error"),
    },
    VestaboardError::ScheduleError { .. } => {
      format_error_with_header("Schedule error", "schedule error")
    },
    VestaboardError::ApiError { code, .. } => match code {
      Some(404) => format_error_with_header("Service not found", "api error"),
      Some(401) | Some(403) => format_error_with_header("Access denied", "api error"),
      Some(500..=599) => format_error_with_header("Service temporarily down", "api error"),
      _ => format_error_with_header("Service error", "api error"),
    },
    VestaboardError::ConfigError { field, .. } => {
      format_error_with_header(&format!("Config: {} missing", field), "config error")
    },
    VestaboardError::Other { message } => {
      // Truncate long messages for display, but be more generous
      let display_msg = if message.len() > 60 {
        message[..57].to_string() + "..."
      } else {
        message.clone()
      };
      format_error_with_header(&display_msg, "error")
    },
  }
}
