//! RF (rfkill) helpers
use std::{
    fs,
    path::{Path, PathBuf},
};

use log::{debug, info, warn};

pub const RFKILL_PATH_BT: &str = "/sys/class/rfkill/rfkill0";
pub const RFKILL_PATH_WIFI: &str = "/sys/class/rfkill/rfkill1";

pub fn rfkill_state_path(path: &std::path::Path) -> PathBuf {
    path.join("state")
}

pub fn write_rfkill_state(path: &Path, block: bool, dry_run: bool) {
    let state = rfkill_state_path(path);
    if dry_run {
        debug!(
            "DRY-RUN: would write '{}' to {}",
            if block { "0" } else { "1" },
            state.display()
        );
        return;
    }
    let _ = std::fs::write(&state, if block { "0" } else { "1" });
    info!(
        "WiFi: {} via {}",
        if block { "blocked" } else { "unblocked" },
        state.display()
    );
}

pub fn find_default_rfkill_path() -> Option<PathBuf> {
    let p = PathBuf::from(RFKILL_PATH_WIFI);
    if p.exists() { Some(p) } else { None }
}

pub fn find_default_rfkill_path_bt() -> Option<PathBuf> {
    let p = PathBuf::from(RFKILL_PATH_BT);
    if p.exists() { Some(p) } else { None }
}

/// Enter power-saving mode: turn display off and reduce CPU freq (order matters)
/// RF toggling configuration
#[derive(Clone, Debug)]
pub struct WifiConfig {
    pub enabled: bool,
    pub rfkill_path: Option<PathBuf>,
}

impl WifiConfig {
    pub fn new(enabled: bool, rfkill_path: Option<PathBuf>) -> Self {
        let mut p = rfkill_path;
        if enabled && p.is_none() {
            p = Some(PathBuf::from(RFKILL_PATH_WIFI));
        }
        WifiConfig {
            enabled,
            rfkill_path: p,
        }
    }

    pub fn block(&self, dry_run: bool) {
        if !self.enabled {
            return;
        }
        if let Some(path) = &self.rfkill_path {
            let state = path.join("state");
            if dry_run {
                debug!("DRY-RUN: would write '0' to {}", state.display());
                return;
            }
            let _ = fs::write(&state, "0");
            debug!("WiFi: blocked via {}", state.display());
        } else {
            warn!("WiFi toggling enabled but no rfkill path provided");
        }
    }

    pub fn unblock(&self, dry_run: bool) {
        if !self.enabled {
            return;
        }
        if let Some(path) = &self.rfkill_path {
            let state = path.join("state");
            if dry_run {
                debug!("DRY-RUN: would write '1' to {}", state.display());
                return;
            }
            let _ = fs::write(&state, "1");
            debug!("WiFi: unblocked via {}", state.display());
        } else {
            warn!("WiFi toggling enabled but no rfkill path provided");
        }
    }
}

/// Bluetooth (BT) toggling configuration
#[derive(Clone, Debug)]
pub struct BTConfig {
    pub enabled: bool,
    pub rfkill_path: Option<PathBuf>,
}

impl BTConfig {
    pub fn new(enabled: bool, rfkill_path: Option<PathBuf>) -> Self {
        let mut p = rfkill_path;
        if enabled && p.is_none() {
            p = Some(PathBuf::from(RFKILL_PATH_BT));
        }
        BTConfig {
            enabled,
            rfkill_path: p,
        }
    }

    pub fn block(&self, dry_run: bool) {
        if !self.enabled {
            return;
        }
        if let Some(path) = &self.rfkill_path {
            let state = path.join("state");
            if dry_run {
                debug!("DRY-RUN: would write '0' to {}", state.display());
                return;
            }
            let _ = fs::write(&state, "0");
            debug!("BT: blocked via {}", state.display());
        } else {
            warn!("BT toggling enabled but no rfkill path provided");
        }
    }

    pub fn unblock(&self, dry_run: bool) {
        if !self.enabled {
            return;
        }
        if let Some(path) = &self.rfkill_path {
            let state = path.join("state");
            if dry_run {
                debug!("DRY-RUN: would write '1' to {}", state.display());
                return;
            }
            let _ = fs::write(&state, "1");
            debug!("BT: unblocked via {}", state.display());
        } else {
            warn!("BT toggling enabled but no rfkill path provided");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_find_default_rfkill_path() {
        let _ = find_default_rfkill_path();
    }

    #[test]
    fn test_find_default_rfkill_path_bt() {
        let _ = find_default_rfkill_path_bt();
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
        write_rfkill_state(&tmp, true, true);
        // dry run should not change
        let s = fs::read_to_string(tmp.join("state")).unwrap();
        assert_eq!(s, "0");
    }

    #[test]
    fn test_write_rfkill_state_dry_run_bt() {
        let tmp = env::temp_dir().join(format!(
            "uconsole_bt_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = fs::create_dir_all(&tmp);
        fs::write(tmp.join("state"), "0").unwrap();
        write_rfkill_state(&tmp, true, true);
        // dry run should not change
        let s = fs::read_to_string(tmp.join("state")).unwrap();
        assert_eq!(s, "0");
    }
}
