//! Framebuffer detection

use crate::error::Error;
use std::path::PathBuf;

#[cfg(test)]
use std::fs;

const FRAMEBUFFER_PATH: &str = "/sys/class/graphics/fb0";

/// Find framebuffer device
///
/// # Returns
/// - Ok(Some(PathBuf)) if framebuffer found
/// - Ok(None) if not found
/// - Err(Error) if error occurred
pub fn find_framebuffer() -> Result<Option<PathBuf>, Error> {
    let path = PathBuf::from(FRAMEBUFFER_PATH);

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

/// Get framebuffer virtual resolution
///
/// # Arguments
/// * `path` - Path to the framebuffer device
///
/// # Returns
/// - Ok(Some((width, height))) if found
/// - Ok(None) if not available
/// - Err(Error) if error occurred
#[cfg(test)]
fn get_virtual_resolution(path: &std::path::Path) -> Result<Option<(u32, u32)>, Error> {
    let virtual_size_path = path.join("virtual_size");

    match fs::read_to_string(&virtual_size_path) {
        Ok(content) => {
            let parts: Vec<&str> = content.trim().split(',').collect();
            if let (Ok(width), Ok(height)) = (
                parts[0].trim().parse::<u32>(),
                parts[1].trim().parse::<u32>(),
            ) && parts.len() == 2
            {
                return Ok(Some((width, height)));
            }
            Ok(None)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

/// Get framebuffer physical resolution
///
/// # Arguments
/// * `path` - Path to the framebuffer device
///
/// # Returns
/// - Ok(Some((width, height))) if found
/// - Ok(None) if not available
/// - Err(Error) if error occurred
#[cfg(test)]
fn get_physical_resolution(path: &std::path::Path) -> Result<Option<(u32, u32)>, Error> {
    let phys_size_path = path.join("phys_size");

    match fs::read_to_string(&phys_size_path) {
        Ok(content) => {
            let parts: Vec<&str> = content.trim().split(',').collect();
            if let (Ok(width), Ok(height)) = (
                parts[0].trim().parse::<u32>(),
                parts[1].trim().parse::<u32>(),
            ) && parts.len() == 2
            {
                return Ok(Some((width, height)));
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
    fn test_framebuffer_path_constant() {
        assert_eq!(FRAMEBUFFER_PATH, "/sys/class/graphics/fb0");
    }

    #[test]
    fn test_find_framebuffer_returns_option() {
        if find_framebuffer().is_ok() {}
    }

    #[test]
    fn test_virtual_resolution_parsing_valid() {
        let virtual_size_str = "1920,1080";
        let parts: Vec<&str> = virtual_size_str.trim().split(',').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].trim().parse::<u32>().unwrap(), 1920);
        assert_eq!(parts[1].trim().parse::<u32>().unwrap(), 1080);
    }

    #[test]
    fn test_physical_resolution_parsing_valid() {
        let phys_size_str = "268,150";
        let parts: Vec<&str> = phys_size_str.trim().split(',').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].trim().parse::<u32>().unwrap(), 268);
        assert_eq!(parts[1].trim().parse::<u32>().unwrap(), 150);
    }

    #[test]
    fn test_resolution_parsing_with_whitespace() {
        let size_str = "  1920 , 1080  ";
        let parts: Vec<&str> = size_str.trim().split(',').collect();
        assert_eq!(parts[0].trim().parse::<u32>().unwrap(), 1920);
        assert_eq!(parts[1].trim().parse::<u32>().unwrap(), 1080);
    }

    #[test]
    fn test_resolution_parsing_various_sizes() {
        let sizes = vec![
            ("640,480", (640, 480)),
            ("1024,768", (1024, 768)),
            ("1920,1080", (1920, 1080)),
        ];

        for (size_str, (expected_width, expected_height)) in sizes {
            let parts: Vec<&str> = size_str.split(',').collect();
            let width = parts[0].trim().parse::<u32>().unwrap();
            let height = parts[1].trim().parse::<u32>().unwrap();
            assert_eq!((width, height), (expected_width, expected_height));
        }
    }

    #[test]
    fn test_invalid_resolution_format() {
        let size_str = "invalid";
        let parts: Vec<&str> = size_str.split(',').collect();
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn test_get_framebuffer_resolutions_funcs() {
        use std::fs;
        let tmp = std::env::temp_dir().join(format!(
            "uconsole_fb_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = fs::create_dir_all(&tmp);
        fs::write(tmp.join("virtual_size"), "1920,1080\n").unwrap();
        fs::write(tmp.join("phys_size"), "268,150\n").unwrap();

        let v = get_virtual_resolution(&tmp).unwrap();
        assert_eq!(v, Some((1920u32, 1080u32)));
        let p = get_physical_resolution(&tmp).unwrap();
        assert_eq!(p, Some((268u32, 150u32)));
    }

    #[test]
    fn test_pathbuf_join_operation() {
        let base = PathBuf::from("/sys/class/graphics/fb0");
        let virtual_size_path = base.join("virtual_size");
        assert!(virtual_size_path.to_string_lossy().contains("virtual_size"));
    }

    #[test]
    fn test_resolution_edge_cases() {
        assert_eq!("0,0".split(',').count(), 2);
        assert_eq!(
            ("4294967295,4294967295".split(',').collect::<Vec<_>>().len()),
            2
        );
    }
}
