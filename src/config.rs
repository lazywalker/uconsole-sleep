//! Simple config file parsing helpers
//!
//! Supports reading simple KEY=VALUE pairs from a config file (shell-style
//! comments with #). Loads environment variables first and then overlays the
//! values from a config file if present. This is intentionally lightweight.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::wifi;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub dry_run: bool,
    pub debug: bool,
    pub policy_path: Option<PathBuf>,
    pub saving_cpu_freq: Option<String>,
    pub hold_trigger_sec: Option<f32>,
    pub toggle_wifi: bool,
    pub wifi_rfkill_path: Option<PathBuf>,
}

// Default impl derived via #[derive(Default)]

fn parse_bool(s: &str) -> bool {
    matches!(s.to_ascii_lowercase().as_str(), "1" | "true" | "yes")
}

fn parse_value_map(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim().to_string();
            let val = line[eq + 1..].trim().to_string();
            map.insert(key, val);
        }
    }
    map
}

impl Config {
    /// Load config by overlaying env variables with values from config file.
    /// If `path` is None, we try repo-local `./etc/uconsole-sleep/config.default` first,
    /// then `/etc/uconsole-sleep/config`.
    pub fn load(path: Option<PathBuf>) -> Self {
        let mut cfg = Config::default();

        // Overlay from environment variables
        if let Ok(v) = std::env::var("DRY_RUN") {
            cfg.dry_run = parse_bool(&v);
        }
        if let Ok(v) = std::env::var("DEBUG") {
            cfg.debug = parse_bool(&v);
        }
        if let Ok(v) = std::env::var("POLICY_PATH") {
            cfg.policy_path = Some(PathBuf::from(v));
        }
        if let Ok(v) = std::env::var("SAVING_CPU_FREQ") {
            cfg.saving_cpu_freq = Some(v);
        }
        if let Ok(v) = std::env::var("HOLD_TRIGGER_SEC") {
            cfg.hold_trigger_sec = v.parse::<f32>().ok();
        }
        if let Ok(v) = std::env::var("TOGGLE_WIFI") {
            cfg.toggle_wifi = parse_bool(&v);
        }
        if let Ok(v) = std::env::var("WIFI_RFKILL") {
            cfg.wifi_rfkill_path = Some(PathBuf::from(v));
        }

        // Determine config file path
        let cfg_path = if let Some(p) = path {
            p
        } else if PathBuf::from("./etc/uconsole-sleep/config.default").exists() {
            PathBuf::from("./etc/uconsole-sleep/config.default")
        } else {
            PathBuf::from("/etc/uconsole-sleep/config")
        };

        if let Ok(content) = fs::read_to_string(&cfg_path) {
            let map = parse_value_map(&content);
            if let Some(v) = map.get("DRY_RUN") {
                cfg.dry_run = parse_bool(v);
            }
            if let Some(v) = map.get("DEBUG") {
                cfg.debug = parse_bool(v);
            }
            if let Some(v) = map.get("POLICY_PATH") {
                cfg.policy_path = Some(PathBuf::from(v));
            }
            if let Some(v) = map.get("SAVING_CPU_FREQ") {
                cfg.saving_cpu_freq = Some(v.clone());
            }
            if let Some(v) = map.get("HOLD_TRIGGER_SEC") {
                cfg.hold_trigger_sec = v.parse::<f32>().ok();
            }
            if let Some(v) = map.get("TOGGLE_WIFI") {
                cfg.toggle_wifi = parse_bool(v);
            }
            if let Some(v) = map.get("WIFI_RFKILL") {
                cfg.wifi_rfkill_path = Some(PathBuf::from(v));
            }
        }

        // final: if wifi enabled and no rfkill path provided, set default
        if cfg.toggle_wifi && cfg.wifi_rfkill_path.is_none() {
            cfg.wifi_rfkill_path = Some(PathBuf::from(wifi::RFKILL_PATH));
        }

        cfg
    }

    #[cfg(test)]
    pub fn load_test_file(path: &std::path::Path) -> Self {
        Config::load(Some(path.to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    use crate::wifi;

    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_load_from_repo_default() {
        // `./etc/uconsole-sleep/config.default` exists in repo and contains values
        let c = Config::load(None);
        assert!(c.saving_cpu_freq.is_some());
        assert_eq!(c.saving_cpu_freq.unwrap(), "100,600");
        assert_eq!(c.hold_trigger_sec.unwrap(), 0.7_f32);
    }

    #[test]
    fn test_wifi_default_rfkill() {
        let tmp = env::temp_dir().join(format!(
            "uconsole_cfg_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = fs::create_dir_all(&tmp);
        let cfg_file = tmp.join("cfg");
        fs::write(&cfg_file, "TOGGLE_WIFI=true\n").unwrap();
        let cfg = Config::load(Some(cfg_file.clone()));
        assert!(cfg.toggle_wifi);
        assert_eq!(
            cfg.wifi_rfkill_path.unwrap(),
            PathBuf::from(wifi::RFKILL_PATH)
        );
    }
}
