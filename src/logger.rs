//! Logger module for Rancer
//!
//! Provides file-based logging with timestamps and log levels.
//! Writes to `rancer.log` in the platform-specific data directory.
//! Also provides timing utilities for profiling.

use chrono::Local;
use std::time::Instant;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// Global logger instance
static LOGGER: Mutex<Option<File>> = Mutex::new(None);

/// Get the platform-specific log file path
fn get_log_path() -> PathBuf {
    if let Some(data_dir) = dirs::data_local_dir() {
        let rancer_dir = data_dir.join("rancer");
        let _ = std::fs::create_dir_all(&rancer_dir);
        rancer_dir.join("rancer.log")
    } else {
        // Fallback to CWD if platform directory unavailable
        PathBuf::from("rancer.log")
    }
}

/// Initialize the logger
/// Creates or overwrites the `rancer.log` file
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = get_log_path();
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&log_path)?;

    let mut logger = LOGGER.lock().unwrap_or_else(|e| e.into_inner());
    *logger = Some(file);

    let log_path_str = log_path.display();
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!(
        "[{}] INFO  - Logger initialized - writing to {}\n",
        timestamp, log_path_str
    );
    if let Some(f) = logger.as_mut() {
        let _ = f.write_all(log_line.as_bytes());
        let _ = f.flush();
    }
    // Drop the lock before printing to stdout
    drop(logger);
    print!("{}", log_line);

    Ok(())
}

/// Write a log message with timestamp and level
fn write_log(level: LogLevel, msg: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!("[{}] {:5} - {}\n", timestamp, level.as_str(), msg);

    if let Ok(mut logger) = LOGGER.lock()
        && let Some(file) = logger.as_mut()
    {
        let _ = file.write_all(log_line.as_bytes());
        let _ = file.flush();
    }

    print!("{}", log_line);
}

/// Log a debug message
pub fn debug(msg: &str) {
    write_log(LogLevel::Debug, msg);
}

/// Log an info message
pub fn info(msg: &str) {
    write_log(LogLevel::Info, msg);
}

/// Log a warning message
pub fn warn(msg: &str) {
    write_log(LogLevel::Warn, msg);
}

/// Log an error message
pub fn error(msg: &str) {
    write_log(LogLevel::Error, msg);
}

/// Timing scope for profiling - prints elapsed time when dropped
pub struct Timer {
    name: &'static str,
    start: Instant,
}

impl Timer {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_secs_f64();
        write_log(
            LogLevel::Debug,
            &format!("{} took {:.3}ms", self.name, elapsed * 1000.0),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn test_logger_initialization() {
        // This test verifies that init() can be called without deadlock
        let result = init();
        assert!(result.is_ok(), "Logger initialization should succeed");

        // Verify we can write logs after initialization
        info("Test log message");
        warn("Test warning");
        error("Test error");

        // If we reach here without hanging, the test passes
        // (deadlock would cause the test to hang forever)
    }
}
