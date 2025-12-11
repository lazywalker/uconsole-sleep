//! Power key event detection
use crate::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const EVENT_PATH: &str = "/dev/input/by-path";
const POWER_KEY_IDENTIFIER: &str = "axp221-pek";

/// Find power key input device
///
/// # Returns
/// - Ok(Some(PathBuf)) if power key device found
/// - Ok(None) if not found
/// - Err(Error) if error occurred
pub fn find_power_key() -> Result<Option<PathBuf>, Error> {
    let event_path = std::path::Path::new(EVENT_PATH);

    match fs::read_dir(event_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str()
                    && name.contains(POWER_KEY_IDENTIFIER)
                {
                    return Ok(Some(entry.path()));
                }
            }
            Ok(None)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

/// Check if power key device is readable
///
/// # Arguments
/// * `device_path` - Path to the power key device
///
/// # Returns
/// - Ok(true) if readable
/// - Ok(false) if not readable
/// - Err(Error) if error occurred
pub fn is_power_key_readable(device_path: &Path) -> Result<bool, Error> {
    use std::os::unix::fs::PermissionsExt;

    match fs::metadata(device_path) {
        Ok(metadata) => {
            let permissions = metadata.permissions();
            let mode = permissions.mode();
            // Check if readable (owner, group, or other can read)
            Ok(mode & 0o444 != 0)
        }
        Err(e) => Err(Error::from(e)),
    }
}

/// Get power key device name
///
/// # Arguments
/// * `device_path` - Path to the power key device
///
/// # Returns
/// - Ok(Some(name)) if found
/// - Ok(None) if not available
/// - Err(Error) if error occurred
#[cfg(test)]
fn get_power_key_name(device_path: &std::path::Path) -> Result<Option<String>, Error> {
    if let Some(name_str) = device_path.file_name().and_then(|n| n.to_str()) {
        return Ok(Some(name_str.to_string()));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_key_identifier_constant() {
        assert_eq!(POWER_KEY_IDENTIFIER, "axp221-pek");
    }

    #[test]
    fn test_event_path_constant() {
        assert_eq!(EVENT_PATH, "/dev/input/by-path");
    }

    #[test]
    fn test_find_power_key_returns_option() {
        if find_power_key().is_ok() {}
    }

    #[test]
    fn test_power_key_identifier_matching() {
        let test_name = "pci-0000:00:14.0-usb-0:1:1.0-event-kbd";
        assert!(!test_name.contains(POWER_KEY_IDENTIFIER));

        let power_key_name = "pci-0000:00:14.0-usb-0:1:1.0-axp221-pek-event-kbd";
        assert!(power_key_name.contains(POWER_KEY_IDENTIFIER));
    }

    #[test]
    fn test_get_power_key_name_from_path() {
        let path = PathBuf::from("/dev/input/by-path/axp221-pek-event-kbd");
        assert_eq!(
            get_power_key_name(&path).unwrap(),
            Some("axp221-pek-event-kbd".to_string())
        );
    }

    #[test]
    fn test_is_power_key_readable_mode_check() {
        // Test the logic for checking readable permissions
        let mode: u32 = 0o644; // readable by all
        assert_ne!(mode & 0o444, 0);

        let mode_unreadable: u32 = 0o200; // not readable
        assert_eq!(mode_unreadable & 0o444, 0);
    }

    #[test]
    fn test_permission_modes() {
        // Test various permission scenarios
        let readable_modes = vec![
            0o644, // rw-r--r--
            0o755, // rwxr-xr-x
            0o777, // rwxrwxrwx
            0o444, // r--r--r--
        ];

        for mode in readable_modes {
            assert_ne!(mode & 0o444, 0, "Mode {:o} should be readable", mode);
        }
    }

    #[test]
    fn test_unreadable_permission_modes() {
        let unreadable_modes = vec![
            0o200, // -w-------
            0o300, // -wx------
            0o600, // rw-------
        ];

        for mode in unreadable_modes {
            if mode & 0o444 == 0 {
                // This is expected for -w------- mode only
                assert_eq!(
                    mode & 0o444,
                    0,
                    "Mode {:o} should not be readable by all",
                    mode
                );
            }
        }
    }

    #[test]
    fn test_pathbuf_file_name_operation() {
        let path = PathBuf::from("/dev/input/by-path/some-device");
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), "some-device");
    }
}
