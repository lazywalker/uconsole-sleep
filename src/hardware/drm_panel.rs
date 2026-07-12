//! DRM panel detection

use crate::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const DRM_PATH: &str = "/sys/class/drm";

/// Find DRM panel (DSI display)
///
/// # Returns
/// - Ok(Some(PathBuf)) if DSI panel found
/// - Ok(None) if not found
/// - Err(Error) if error occurred
pub fn find_drm_panel() -> Result<Option<PathBuf>, Error> {
    let drm_path = std::path::Path::new(DRM_PATH);

    match fs::read_dir(drm_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str()
                    && name.contains("DSI")
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

/// Check if a DRM device is connected
///
/// # Arguments
/// * `device_path` - Path to the DRM device
///
/// # Returns
/// - Ok(true) if connected
/// - Ok(false) if not connected
/// - Err(Error) if error occurred
pub fn is_drm_connected(device_path: &Path) -> Result<bool, Error> {
    let status_path = device_path.join("status");

    match fs::read_to_string(&status_path) {
        Ok(content) => Ok(content.trim() == "connected"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(Error::from(e)),
    }
}
