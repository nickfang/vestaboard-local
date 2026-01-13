#[derive(Debug)]
pub enum VestaboardError {
  IOError { source: std::io::Error, context: String },
  JsonError { source: serde_json::Error, context: String },
  ReqwestError { source: reqwest::Error, context: String },
  WidgetError { widget: String, message: String },
  ScheduleError { operation: String, message: String },
  ApiError { code: Option<u16>, message: String },
  ConfigError { field: String, message: String },
  Other { message: String },
}

impl PartialEq for VestaboardError {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (VestaboardError::IOError { context: c1, .. }, VestaboardError::IOError { context: c2, .. }) => c1 == c2,
      (VestaboardError::JsonError { context: c1, .. }, VestaboardError::JsonError { context: c2, .. }) => c1 == c2,
      (VestaboardError::ReqwestError { context: c1, .. }, VestaboardError::ReqwestError { context: c2, .. }) => {
        c1 == c2
      },
      (
        VestaboardError::WidgetError {
          widget: w1,
          message: m1,
        },
        VestaboardError::WidgetError {
          widget: w2,
          message: m2,
        },
      ) => w1 == w2 && m1 == m2,
      (
        VestaboardError::ScheduleError {
          operation: o1,
          message: m1,
        },
        VestaboardError::ScheduleError {
          operation: o2,
          message: m2,
        },
      ) => o1 == o2 && m1 == m2,
      (VestaboardError::ApiError { code: c1, message: m1 }, VestaboardError::ApiError { code: c2, message: m2 }) => {
        c1 == c2 && m1 == m2
      },
      (
        VestaboardError::ConfigError { field: f1, message: m1 },
        VestaboardError::ConfigError { field: f2, message: m2 },
      ) => f1 == f2 && m1 == m2,
      (VestaboardError::Other { message: m1 }, VestaboardError::Other { message: m2 }) => m1 == m2,
      _ => false,
    }
  }
}

impl std::fmt::Display for VestaboardError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VestaboardError::IOError { source, context } => {
        write!(f, "IO Error in {}: {}", context, source)
      },
      VestaboardError::JsonError { source, context } => {
        write!(f, "JSON Error in {}: {}", context, source)
      },
      VestaboardError::ReqwestError { source, context } => {
        write!(f, "HTTP Request Error in {}: {}", context, source)
      },
      VestaboardError::WidgetError { widget, message } => {
        write!(f, "Widget Error [{}]: {}", widget, message)
      },
      VestaboardError::ScheduleError { operation, message } => {
        write!(f, "Schedule Error [{}]: {}", operation, message)
      },
      VestaboardError::ApiError { code, message } => match code {
        Some(c) => write!(f, "API Error [{}]: {}", c, message),
        None => write!(f, "API Error: {}", message),
      },
      VestaboardError::ConfigError { field, message } => {
        write!(f, "Configuration Error [{}]: {}", field, message)
      },
      VestaboardError::Other { message } => {
        write!(f, "Error: {}", message)
      },
    }
  }
}

impl std::error::Error for VestaboardError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      VestaboardError::IOError { source, .. } => Some(source),
      VestaboardError::JsonError { source, .. } => Some(source),
      VestaboardError::ReqwestError { source, .. } => Some(source),
      _ => None,
    }
  }
}

// Convenient conversion methods
impl VestaboardError {
  pub fn io_error(source: std::io::Error, context: &str) -> Self {
    VestaboardError::IOError {
      source,
      context: context.to_string(),
    }
  }

  pub fn json_error(source: serde_json::Error, context: &str) -> Self {
    VestaboardError::JsonError {
      source,
      context: context.to_string(),
    }
  }

  pub fn reqwest_error(source: reqwest::Error, context: &str) -> Self {
    VestaboardError::ReqwestError {
      source,
      context: context.to_string(),
    }
  }

  pub fn widget_error(widget: &str, message: &str) -> Self {
    VestaboardError::WidgetError {
      widget: widget.to_string(),
      message: message.to_string(),
    }
  }

  pub fn schedule_error(operation: &str, message: &str) -> Self {
    VestaboardError::ScheduleError {
      operation: operation.to_string(),
      message: message.to_string(),
    }
  }

  pub fn api_error(code: Option<u16>, message: &str) -> Self {
    VestaboardError::ApiError {
      code,
      message: message.to_string(),
    }
  }

  pub fn config_error(field: &str, message: &str) -> Self {
    VestaboardError::ConfigError {
      field: field.to_string(),
      message: message.to_string(),
    }
  }

  pub fn other(message: &str) -> Self {
    VestaboardError::Other {
      message: message.to_string(),
    }
  }
}

// From implementations for automatic conversion
impl From<std::io::Error> for VestaboardError {
  fn from(error: std::io::Error) -> Self {
    VestaboardError::io_error(error, "unknown context")
  }
}

impl From<serde_json::Error> for VestaboardError {
  fn from(error: serde_json::Error) -> Self {
    VestaboardError::json_error(error, "unknown context")
  }
}

impl From<reqwest::Error> for VestaboardError {
  fn from(error: reqwest::Error) -> Self {
    VestaboardError::reqwest_error(error, "unknown context")
  }
}

impl VestaboardError {
  /// Convert error to user-friendly message string
  pub fn to_user_message(&self) -> String {
    match self {
      VestaboardError::IOError { source, context } => {
        if source.kind() == std::io::ErrorKind::NotFound {
          if context.contains("file") {
            // Extract file path from context if possible
            format!("File not found: {}", context)
          } else {
            format!("File not found: {}", context)
          }
        } else {
          format!("Error accessing file: {}", context)
        }
      },
      VestaboardError::JsonError { context, .. } => {
        format!("Error parsing data: {}", context)
      },
      VestaboardError::ReqwestError { source, context } => {
        if source.is_connect() || source.is_timeout() {
          if context.contains("weather") {
            "Network error: Unable to reach weather service".to_string()
          } else if context.contains("Vestaboard") || context.contains("local-api") {
            "Network error: Unable to reach Vestaboard".to_string()
          } else {
            format!("Network error: Unable to connect to {}. Check your internet connection.", context)
          }
        } else if source.status() == Some(reqwest::StatusCode::UNAUTHORIZED)
          || source.status() == Some(reqwest::StatusCode::FORBIDDEN)
        {
          if context.contains("weather") {
            "Authentication error: Check WEATHER_API_KEY".to_string()
          } else if context.contains("Vestaboard") || context.contains("local-api") {
            "Authentication error: Check LOCAL_API_KEY".to_string()
          } else {
            format!("Authentication error: Check API credentials")
          }
        } else if source.status() == Some(reqwest::StatusCode::BAD_GATEWAY)
          || source.status() == Some(reqwest::StatusCode::GATEWAY_TIMEOUT)
        {
          if context.contains("weather") {
            "Weather service temporarily unavailable".to_string()
          } else {
            format!("Service temporarily unavailable: {}", context)
          }
        } else {
          format!("Network error: {}", context)
        }
      },
      VestaboardError::WidgetError { widget, message } => {
        format!("Widget error: {} - {}", widget, message)
      },
      VestaboardError::ScheduleError { operation, message } => {
        format!("Schedule error: {} - {}", operation, message)
      },
      VestaboardError::ApiError { code, message } => {
        if message.contains("Invalid characters") {
          message.clone()
        } else {
          match code {
            Some(c) => format!("API error [{}]: {}", c, message),
            None => message.clone(),
          }
        }
      },
      VestaboardError::ConfigError { field, message } => {
        format!("Configuration error [{}]: {}", field, message)
      },
      VestaboardError::Other { message } => message.clone(),
    }
  }
}
