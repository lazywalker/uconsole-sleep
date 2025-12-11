//! Wifi (rfkill) helpers
use crate::logger::Logger;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const RFKILL_PATH: &str = "/sys/class/rfkill/rfkill0";

pub fn rfkill_state_path(path: &std::path::Path) -> PathBuf {
    path.join("state")
}

pub fn write_rfkill_state(logger: &Logger, path: &Path, block: bool, dry_run: bool) {
    let state = rfkill_state_path(path);
    if dry_run {
        logger.debug(&format!(
            "DRY-RUN: would write '{}' to {}",
            if block { "1" } else { "0" },
            state.display()
        ));
        return;
    }
    let _ = std::fs::write(&state, if block { "1" } else { "0" });
    logger.debug(&format!(
        "WiFi: {} via {}",
        if block { "blocked" } else { "unblocked" },
        state.display()
    ));
}

pub fn find_default_rfkill_path() -> Option<PathBuf> {
    let p = PathBuf::from(RFKILL_PATH);
    if p.exists() { Some(p) } else { None }
}

/// Enter power-saving mode: turn display off and reduce CPU freq (order matters)
/// Wifi toggling configuration
#[derive(Clone, Debug)]
pub struct WifiConfig {
    pub enabled: bool,
    pub rfkill_path: Option<PathBuf>,
}

impl WifiConfig {
    pub fn new(enabled: bool, rfkill_path: Option<PathBuf>) -> Self {
        let mut p = rfkill_path;
        if enabled && p.is_none() {
            p = Some(PathBuf::from(RFKILL_PATH));
        }
        WifiConfig {
            enabled,
            rfkill_path: p,
        }
    }

    pub fn block(&self, logger: &Logger, dry_run: bool) {
        if !self.enabled {
            return;
        }
        if let Some(path) = &self.rfkill_path {
            let state = path.join("state");
            if dry_run {
                logger.debug(&format!("DRY-RUN: would write '1' to {}", state.display()));
                return;
            }
            let _ = fs::write(&state, "1");
            logger.debug(&format!("WiFi: blocked via {}", state.display()));
        } else {
            logger.warn("WiFi toggling enabled but no rfkill path provided");
        }
    }

    pub fn unblock(&self, logger: &Logger, dry_run: bool) {
        if !self.enabled {
            return;
        }
        if let Some(path) = &self.rfkill_path {
            let state = path.join("state");
            if dry_run {
                logger.debug(&format!("DRY-RUN: would write '0' to {}", state.display()));
                return;
            }
            let _ = fs::write(&state, "0");
            logger.debug(&format!("WiFi: unblocked via {}", state.display()));
        } else {
            logger.warn("WiFi toggling enabled but no rfkill path provided");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::Logger;
    use std::env;
    use std::fs;

    #[test]
    fn test_find_default_rfkill_path() {
        let _ = find_default_rfkill_path();
    }

    #[test]
    fn test_write_rfkill_state_dry_run() {
        let tmp = env::temp_dir().join(format!(
            "uconsole_wifi_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = fs::create_dir_all(&tmp);
        fs::write(tmp.join("state"), "0").unwrap();
        let logger = Logger::new(false);
        write_rfkill_state(&logger, &tmp, true, true);
        // dry run should not change
        let s = fs::read_to_string(tmp.join("state")).unwrap();
        assert_eq!(s, "0");
    }
}
