//! Framebuffer detection

use crate::error::Error;
use std::path::PathBuf;

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
