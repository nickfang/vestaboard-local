use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};

pub fn datetime_to_utc(time_str: &str) -> Result<DateTime<Utc>, String> {
  let naive_datetime = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S")
    .map_err(|e| format!("Invalid time format for '{}'. Please use YYYY-MM-DD HH:MM:SS. Details: {}", time_str, e))?;
  let local_datetime = Local.from_local_datetime(&naive_datetime)
        .single()
        .ok_or_else(|| {
            format!("Could not convert '{}' to a valid local time. It might be an ambiguous or non-existent time (e.g., during DST change).", time_str)
        })?;
  Ok(local_datetime.with_timezone(&Utc))
}

/// Converts a UTC datetime to local timezone with user-friendly format (dots, 12-hour)
/// Format: YYYY.MM.DD HH:MM AM/PM
pub fn datetime_to_local(dt: DateTime<Utc>) -> String {
  let local_time = dt.with_timezone(&Local::now().timezone());
  local_time.format("%Y.%m.%d %I:%M %p").to_string()
}

/// Returns the current local time in 24-hour format for logging
/// Format: YYYY-MM-DD HH:MM:SS
pub fn now_local_24() -> String {
  Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
