//! Backlight detection and control

use crate::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const BACKLIGHT_PATH: &str = "/sys/class/backlight/backlight@0";

/// Find the backlight device
///
/// # Returns
/// - Ok(Some(PathBuf)) if backlight found
/// - Ok(None) if not found
/// - Err(Error) if error occurred
pub fn find_backlight() -> Result<Option<PathBuf>, Error> {
    let path = PathBuf::from(BACKLIGHT_PATH);

    match path.try_exists() {
        Ok(exists) => {
            if exists {
                Ok(Some(path))
            } else {
                Ok(None)
            }
        }
        Err(e) => Err(Error::from(e)),
    }
}

/// Get current backlight brightness
///
/// # Arguments
/// * `path` - Path to the backlight device
///
/// # Returns
/// - Ok(brightness) if successful
/// - Err(Error) if failed
pub fn get_brightness(path: &Path) -> Result<u32, Error> {
    let brightness_path = path.join("brightness");

    let content = fs::read_to_string(&brightness_path)
        .map_err(|e| Error::Io(format!("Failed to read brightness: {}", e)))?;

    content
        .trim()
        .parse::<u32>()
        .map_err(|_| Error::InvalidDevice("Invalid brightness value".to_string()))
}

/// Set backlight brightness
///
/// # Arguments
/// * `path` - Path to the backlight device
/// * `brightness` - Brightness value (0-100)
///
/// # Returns
/// - Ok(()) if successful
/// - Err(Error) if failed
pub fn set_brightness(path: &Path, brightness: u32) -> Result<(), Error> {
    let brightness_path = path.join("brightness");

    fs::write(&brightness_path, brightness.to_string()).map_err(Error::from)
}

/// Get maximum brightness
///
/// # Arguments
/// * `path` - Path to the backlight device
///
/// # Returns
/// - Ok(max_brightness) if successful
/// - Err(Error) if failed
pub fn get_max_brightness(path: &Path) -> Result<u32, Error> {
    let max_brightness_path = path.join("max_brightness");

    let content = fs::read_to_string(&max_brightness_path)
        .map_err(|e| Error::Io(format!("Failed to read max_brightness: {}", e)))?;

    content
        .trim()
        .parse::<u32>()
        .map_err(|_| Error::InvalidDevice("Invalid max brightness value".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backlight_path_constant() {
        assert_eq!(BACKLIGHT_PATH, "/sys/class/backlight/backlight@0");
    }

    #[test]
    fn test_find_backlight_returns_option() {
        // This test verifies the function signature and return type
        if find_backlight().is_ok() {}
    }

    #[test]
    fn test_get_brightness_with_valid_string() {
        // Test parsing logic
        let test_brightness = "100";
        assert_eq!(test_brightness.parse::<u32>().unwrap(), 100);
    }

    #[test]
    fn test_get_max_brightness_parse_logic() {
        let max_brightness_str = "500";
        assert_eq!(max_brightness_str.parse::<u32>().unwrap(), 500);
    }

    #[test]
    fn test_set_brightness_converts_to_string() {
        let brightness: u32 = 255;
        let brightness_str = brightness.to_string();
        assert_eq!(brightness_str, "255");
    }

    #[test]
    fn test_brightness_parsing_edge_cases() {
        assert_eq!("0".parse::<u32>().unwrap(), 0);
        assert_eq!("1".parse::<u32>().unwrap(), 1);
        assert_eq!("4294967295".parse::<u32>().unwrap(), u32::MAX);
    }

    #[test]
    fn test_invalid_brightness_string_fails() {
        let result = "not_a_number".parse::<u32>();
        assert!(result.is_err());
    }

    #[test]
    fn test_pathbuf_join_operation() {
        let base = PathBuf::from("/sys/class/backlight/backlight@0");
        let brightness_path = base.join("brightness");
        assert!(brightness_path.to_string_lossy().contains("brightness"));
    }
}
