//! Internal keyboard detection

use crate::error::Error;
use std::fs;
use std::path::PathBuf;

const USB_DEVICES_PATH: &str = "/sys/bus/usb/devices";

/// Find internal keyboard USB device
///
/// # Arguments
/// * `ids` - Slice of USB device IDs to search for (format: "vid:pid")
///
/// # Returns
/// - Ok(Some(PathBuf)) if device found
/// - Ok(None) if not found
/// - Err(Error) if error occurred
pub fn find_internal_kb(ids: &[&str]) -> Result<Option<PathBuf>, Error> {
    let usb_path = std::path::Path::new(USB_DEVICES_PATH);

    match fs::read_dir(usb_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let device_path = entry.path();
                let vendor_path = device_path.join("idVendor");
                let product_path = device_path.join("idProduct");

                if !vendor_path.exists() || !product_path.exists() {
                    continue;
                }

                let vid_res = fs::read_to_string(&vendor_path);
                let pid_res = fs::read_to_string(&product_path);

                if let (Ok(vid), Ok(pid)) = (vid_res, pid_res) {
                    let device_id = format!("{}:{}", vid.trim(), pid.trim());
                    if ids.contains(&device_id.as_str()) {
                        return Ok(Some(device_path));
                    }
                }
            }
            Ok(None)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

/// Get USB device vendor ID
///
/// # Arguments
/// * `device_path` - Path to the USB device
///
/// # Returns
/// - Ok(Some(vendor_id)) if found
/// - Ok(None) if not available
/// - Err(Error) if error occurred
#[cfg(test)]
fn get_vendor_id(device_path: &std::path::Path) -> Result<Option<String>, Error> {
    let vendor_path = device_path.join("idVendor");

    match fs::read_to_string(&vendor_path) {
        Ok(content) => Ok(Some(content.trim().to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

/// Get USB device product ID
///
/// # Arguments
/// * `device_path` - Path to the USB device
///
/// # Returns
/// - Ok(Some(product_id)) if found
/// - Ok(None) if not available
/// - Err(Error) if error occurred
#[cfg(test)]
fn get_product_id(device_path: &std::path::Path) -> Result<Option<String>, Error> {
    let product_path = device_path.join("idProduct");

    match fs::read_to_string(&product_path) {
        Ok(content) => Ok(Some(content.trim().to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

/// Get USB device manufacturer name
///
/// # Arguments
/// * `device_path` - Path to the USB device
///
/// # Returns
/// - Ok(Some(manufacturer)) if found
/// - Ok(None) if not available
/// - Err(Error) if error occurred
#[cfg(test)]
fn get_manufacturer(device_path: &std::path::Path) -> Result<Option<String>, Error> {
    let manufacturer_path = device_path.join("manufacturer");

    match fs::read_to_string(&manufacturer_path) {
        Ok(content) => Ok(Some(content.trim().to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_devices_path_constant() {
        assert_eq!(USB_DEVICES_PATH, "/sys/bus/usb/devices");
    }

    #[test]
    fn test_find_internal_kb_returns_option() {
        let ids = ["feed:0000", "1eaf:0003"];
        if find_internal_kb(&ids).is_ok() {}
    }

    #[test]
    fn test_device_id_format_creation() {
        let vid = "feed";
        let pid = "0000";
        let device_id = format!("{}:{}", vid, pid);
        assert_eq!(device_id, "feed:0000");
    }

    #[test]
    fn test_device_id_matching() {
        let test_ids = ["feed:0000", "1eaf:0003"];
        let device_id = "feed:0000";

        let found = test_ids.contains(&device_id);
        assert!(found);
    }

    #[test]
    fn test_device_id_no_match() {
        let test_ids = ["feed:0000", "1eaf:0003"];
        let device_id = "9999:9999";

        let found = test_ids.contains(&device_id);
        assert!(!found);
    }

    #[test]
    fn test_vendor_id_parsing() {
        let vendor_str = "feed";
        assert_eq!(vendor_str.trim(), "feed");
    }

    #[test]
    fn test_product_id_parsing() {
        let product_str = "0000";
        assert_eq!(product_str.trim(), "0000");
    }

    #[test]
    fn test_device_id_with_leading_trailing_spaces() {
        let vid = "  feed  ";
        let pid = "  0000  ";
        let device_id = format!("{}:{}", vid.trim(), pid.trim());
        assert_eq!(device_id, "feed:0000");
    }

    #[test]
    fn test_pathbuf_join_operations() {
        let base = PathBuf::from("/sys/bus/usb/devices/1-1");

        let vendor_path = base.join("idVendor");
        assert!(vendor_path.to_string_lossy().contains("idVendor"));

        let product_path = base.join("idProduct");
        assert!(product_path.to_string_lossy().contains("idProduct"));

        let manufacturer_path = base.join("manufacturer");
        assert!(manufacturer_path.to_string_lossy().contains("manufacturer"));
    }

    #[test]
    fn test_multiple_device_ids_check() {
        let ids = ["feed:0000", "1eaf:0003", "1234:5678"];
        let device_id = "1eaf:0003";

        let found = ids.contains(&device_id);
        assert!(found);
    }

    #[test]
    fn test_empty_id_slice() {
        let ids: Vec<&str> = vec![];
        let device_id = "feed:0000";

        let found = ids.contains(&device_id);
        assert!(!found);
    }

    #[test]
    fn test_vendor_product_id_format() {
        let vendor = "0x1234";
        let product = "0xabcd";
        let device_id = format!("{}:{}", vendor, product);
        assert_eq!(device_id, "0x1234:0xabcd");
    }

    #[test]
    fn test_vendor_product_helper_functions() {
        use std::fs;
        let tmp = std::env::temp_dir().join(format!(
            "uconsole_internal_kb_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = fs::create_dir_all(&tmp);
        fs::write(tmp.join("idVendor"), "feed").unwrap();
        fs::write(tmp.join("idProduct"), "0000").unwrap();
        fs::write(tmp.join("manufacturer"), "Test Manufacturer").unwrap();

        assert_eq!(get_vendor_id(&tmp).unwrap(), Some("feed".to_string()));
        assert_eq!(get_product_id(&tmp).unwrap(), Some("0000".to_string()));
        assert_eq!(
            get_manufacturer(&tmp).unwrap(),
            Some("Test Manufacturer".to_string())
        );
    }
}
