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
