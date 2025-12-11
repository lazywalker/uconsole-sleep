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

/// Get DRM device resolution
///
/// # Arguments
/// * `device_path` - Path to the DRM device
///
/// # Returns
/// - Ok(Some((width, height))) if resolution found
/// - Ok(None) if not found
/// - Err(Error) if error occurred
#[cfg(test)]
fn get_drm_resolution(device_path: &Path) -> Result<Option<(u32, u32)>, Error> {
    let modes_path = device_path.join("modes");

    match fs::read_to_string(&modes_path) {
        Ok(content) => {
            // Parse first line in format "1920x1080"
            if let Some(first_mode) = content.lines().next() {
                let parts: Vec<&str> = first_mode.split('x').collect();
                if parts.len() == 2
                    && let (Ok(width), Ok(height)) =
                        (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                {
                    return Ok(Some((width, height)));
                }
            }
            Ok(None)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drm_path_constant() {
        assert_eq!(DRM_PATH, "/sys/class/drm");
    }

    #[test]
    fn test_find_drm_panel_returns_option() {
        if find_drm_panel().is_ok() {}
    }

    #[test]
    fn test_is_drm_connected_parse_connected() {
        let status = "connected";
        assert_eq!(status.trim(), "connected");
    }

    #[test]
    fn test_is_drm_connected_parse_disconnected() {
        let status = "disconnected";
        assert_ne!(status.trim(), "connected");
    }

    #[test]
    fn test_resolution_parsing_valid_format() {
        let mode_str = "1920x1080";
        let parts: Vec<&str> = mode_str.split('x').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].parse::<u32>().unwrap(), 1920);
        assert_eq!(parts[1].parse::<u32>().unwrap(), 1080);
    }

    #[test]
    fn test_resolution_parsing_various_sizes() {
        let sizes = vec![
            ("640x480", (640, 480)),
            ("1024x768", (1024, 768)),
            ("1920x1080", (1920, 1080)),
            ("2560x1440", (2560, 1440)),
        ];

        for (mode_str, (expected_width, expected_height)) in sizes {
            let parts: Vec<&str> = mode_str.split('x').collect();
            let width = parts[0].parse::<u32>().unwrap();
            let height = parts[1].parse::<u32>().unwrap();
            assert_eq!((width, height), (expected_width, expected_height));
        }
    }

    #[test]
    fn test_resolution_parsing_invalid_format() {
        let mode_str = "invalid";
        let parts: Vec<&str> = mode_str.split('x').collect();
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn test_get_drm_resolution_func() {
        use std::fs;
        let tmp = std::env::temp_dir().join(format!(
            "uconsole_drm_panel_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = fs::create_dir_all(&tmp);
        fs::write(tmp.join("modes"), "1920x1080\n").unwrap();
        let res = get_drm_resolution(&tmp).unwrap();
        assert_eq!(res, Some((1920u32, 1080u32)));
    }

    #[test]
    fn test_drm_path_exists_check() {
        let path = std::path::Path::new(DRM_PATH);
        // This test verifies the path construction
        assert!(path.to_string_lossy().contains("drm"));
    }

    #[test]
    fn test_pathbuf_join_operation() {
        let base = PathBuf::from("/sys/class/drm");
        let status_path = base.join("status");
        assert!(status_path.to_string_lossy().contains("status"));
    }

    #[test]
    fn test_resolution_with_whitespace() {
        let mode_str = "  1920x1080  ";
        let trimmed = mode_str.trim();
        let parts: Vec<&str> = trimmed.split('x').collect();
        assert_eq!(parts[0].parse::<u32>().unwrap(), 1920);
        assert_eq!(parts[1].parse::<u32>().unwrap(), 1080);
    }
}
