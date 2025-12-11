//! Simple logger implementation - no external dependencies

use std::io::{self, Write};

/// Simple logger for console output
pub struct Logger {
    debug_enabled: bool,
}

impl Logger {
    /// Create new logger
    pub fn new(debug_enabled: bool) -> Self {
        Logger { debug_enabled }
    }

    /// Log info level message
    pub fn info(&self, msg: &str) {
        let _ = writeln!(io::stdout(), "[INFO] {}", msg);
    }

    /// Log success level message
    pub fn success(&self, msg: &str) {
        let _ = writeln!(io::stdout(), "{}", msg);
    }

    /// Log warning level message
    pub fn warn(&self, msg: &str) {
        let _ = writeln!(io::stdout(), "[WARN] {}", msg);
    }

    /// Log error level message
    pub fn error(&self, msg: &str) {
        let _ = writeln!(io::stderr(), "[ERROR] {}", msg);
    }

    /// Log debug level message
    pub fn debug(&self, msg: &str) {
        if self.debug_enabled {
            let _ = writeln!(io::stdout(), "[DEBUG] {}", msg);
        }
    }

    /// Return whether debug is enabled
    pub fn is_debug_enabled(&self) -> bool {
        self.debug_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(false);
        assert!(!logger.is_debug_enabled());
    }

    #[test]
    fn test_logger_creation_with_debug() {
        let logger = Logger::new(true);
        assert!(logger.is_debug_enabled());
    }

    #[test]
    fn test_logger_info() {
        let logger = Logger::new(false);
        logger.info("test message"); // Should not panic
    }

    #[test]
    fn test_logger_success() {
        let logger = Logger::new(false);
        logger.success("success message"); // Should not panic
    }

    #[test]
    fn test_logger_warn() {
        let logger = Logger::new(false);
        logger.warn("warning message"); // Should not panic
    }

    #[test]
    fn test_logger_error() {
        let logger = Logger::new(false);
        logger.error("error message"); // Should not panic
    }

    #[test]
    fn test_logger_debug_disabled() {
        let logger = Logger::new(false);
        logger.debug("debug message"); // Should not output
    }

    #[test]
    fn test_logger_debug_enabled() {
        let logger = Logger::new(true);
        logger.debug("debug message"); // Should output
    }
}
