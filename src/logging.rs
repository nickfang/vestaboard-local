use std::fs::OpenOptions;
use std::io::Write;
use env_logger::{Builder, Target};
use crate::vblconfig::VblConfig;
use crate::errors::VestaboardError;

pub fn init_logging() -> Result<(), VestaboardError> {
    let config = VblConfig::load()?;
    
    // Ensure log directory exists
    let log_file_path = config.get_log_file_path();
    if let Some(parent) = log_file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| VestaboardError::io_error(e, "creating log directory"))?;
    }

    // Create file logger
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .map_err(|e| VestaboardError::io_error(e, "opening log file"))?;

    let mut builder = Builder::new();
    
    // Configure file logging
    builder
        .target(Target::Pipe(Box::new(log_file)))
        .filter_level(config.get_log_level())
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] [{}:{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        });

    // Initialize the logger
    builder.try_init()
        .map_err(|e| VestaboardError::other(&format!("Failed to initialize logger: {}", e)))?;

    // Also set up console logging
    setup_console_logging(&config)?;

    log::info!("Logging initialized - file: {}, level: {}", 
               log_file_path.display(), config.log_level);

    Ok(())
}

fn setup_console_logging(config: &VblConfig) -> Result<(), VestaboardError> {
    // For console logging, we'll use a separate approach since env_logger can only have one target
    // We'll use the log macros and manually handle console output for key messages
    log::info!("Console logging level: {:?}", config.get_console_log_level());
    Ok(())
}

// Utility macros for consistent logging patterns
#[macro_export]
macro_rules! log_widget_start {
    ($widget:expr, $input:expr) => {
        log::info!("Widget '{}' starting with input: {:?}", $widget, $input);
    };
}

#[macro_export]
macro_rules! log_widget_success {
    ($widget:expr, $duration:expr) => {
        log::info!("Widget '{}' completed successfully in {:?}", $widget, $duration);
    };
}

#[macro_export]
macro_rules! log_widget_error {
    ($widget:expr, $error:expr, $duration:expr) => {
        log::error!("Widget '{}' failed after {:?}: {}", $widget, $duration, $error);
    };
}

#[macro_export]
macro_rules! log_api_request {
    ($method:expr, $url:expr) => {
        log::debug!("API Request: {} {}", $method, $url);
    };
}

#[macro_export]
macro_rules! log_api_response {
    ($status:expr, $duration:expr) => {
        log::debug!("API Response: {} in {:?}", $status, $duration);
    };
}

#[macro_export]
macro_rules! log_api_error {
    ($error:expr, $duration:expr) => {
        log::error!("API Error after {:?}: {}", $duration, $error);
    };
}

// Console output functions for user-facing messages
#[allow(dead_code)]
pub fn console_info(msg: &str) {
    println!("[INFO] {}", msg);
    log::info!("{}", msg);
}

#[allow(dead_code)]
pub fn console_warn(msg: &str) {
    eprintln!("[WARN] {}", msg);
    log::warn!("{}", msg);
}

#[allow(dead_code)]
pub fn console_error(msg: &str) {
    eprintln!("[ERROR] {}", msg);
    log::error!("{}", msg);
}

#[allow(dead_code)]
pub fn console_debug(msg: &str) {
    if log::log_enabled!(log::Level::Debug) {
        println!("[DEBUG] {}", msg);
    }
    log::debug!("{}", msg);
}
