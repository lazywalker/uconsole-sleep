//! Error handling - pure Rust implementation
//! No external dependencies

use std::fmt;
use std::io;

/// Custom error type
#[derive(Debug, Clone)]
pub enum Error {
    /// IO error with message
    Io(String),
    /// Device not found
    NotFound(String),
    /// Invalid path or device
    InvalidDevice(String),
    /// Permission denied
    PermissionDenied(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(msg) => write!(f, "IO error: {}", msg),
            Error::NotFound(msg) => write!(f, "Not found: {}", msg),
            Error::InvalidDevice(msg) => write!(f, "Invalid device: {}", msg),
            Error::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_io() {
        let err = Error::Io("test error".to_string());
        assert_eq!(err.to_string(), "IO error: test error");
    }

    #[test]
    fn test_error_display_not_found() {
        let err = Error::NotFound("device".to_string());
        assert_eq!(err.to_string(), "Not found: device");
    }

    #[test]
    fn test_error_display_invalid_device() {
        let err = Error::InvalidDevice("bad device".to_string());
        assert_eq!(err.to_string(), "Invalid device: bad device");
    }

    #[test]
    fn test_error_display_permission_denied() {
        let err = Error::PermissionDenied("access denied".to_string());
        assert_eq!(err.to_string(), "Permission denied: access denied");
    }

    #[test]
    fn test_error_clone() {
        let err = Error::NotFound("test".to_string());
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }

    #[test]
    fn test_error_debug() {
        let err = Error::NotFound("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NotFound"));
    }
}
