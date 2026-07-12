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
