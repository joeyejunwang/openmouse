use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

/// Appends a message to app.log with timestamp
pub fn log_to_file(message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("app.log")
    {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = writeln!(file, "[{}] {}", timestamp, message);
    }
}