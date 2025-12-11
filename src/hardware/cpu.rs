//! CPU frequency handling under `hardware` namespace
use crate::logger::Logger;
use std::path::PathBuf;

pub const CPU_POLICY_PATH: &str = "/sys/devices/system/cpu/cpufreq/policy0";

#[derive(Clone, Debug)]
pub struct CpuFreqConfig {
    pub policy_path: PathBuf,
    pub default_min: Option<String>,
    pub default_max: Option<String>,
    pub saving_min: Option<String>,
    pub saving_max: Option<String>,
}

impl CpuFreqConfig {
    pub fn new(saving_cpu_freq: Option<String>) -> Self {
        let policy_path = PathBuf::from(CPU_POLICY_PATH);
        Self::with_policy_path(policy_path, saving_cpu_freq)
    }

    pub fn with_policy_path(policy_path: PathBuf, saving_cpu_freq: Option<String>) -> Self {
        let policy_path_clone = policy_path.clone();
        let default_min = std::fs::read_to_string(policy_path_clone.join("scaling_min_freq")).ok();
        let default_max = std::fs::read_to_string(policy_path_clone.join("scaling_max_freq")).ok();

        let (saving_min, saving_max) = if let Some(s) = saving_cpu_freq {
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() == 2 {
                let min = format!("{}000", parts[0].trim());
                let max = format!("{}000", parts[1].trim());
                (Some(min), Some(max))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        CpuFreqConfig {
            policy_path,
            default_min,
            default_max,
            saving_min,
            saving_max,
        }
    }

    pub fn apply_saving_mode(&self, logger: &Logger, dry_run: bool) {
        if let (Some(min), Some(max)) = (&self.saving_min, &self.saving_max) {
            if dry_run {
                logger.debug(&format!(
                    "DRY-RUN: Would write CPU saving mode {}/{} to {}",
                    min,
                    max,
                    self.policy_path.display()
                ));
            } else {
                let _ = std::fs::write(self.policy_path.join("scaling_min_freq"), min);
                let _ = std::fs::write(self.policy_path.join("scaling_max_freq"), max);
            }
            logger.debug(&format!("CPU: saving mode {}/{}", min, max));
        }
    }

    pub fn apply_normal_mode(&self, logger: &Logger, dry_run: bool) {
        if let (Some(min), Some(max)) = (&self.default_min, &self.default_max) {
            if dry_run {
                logger.debug(&format!(
                    "DRY-RUN: Would write CPU normal mode {}/{} to {}",
                    min.trim(),
                    max.trim(),
                    self.policy_path.display()
                ));
            } else {
                let _ = std::fs::write(self.policy_path.join("scaling_min_freq"), min.trim());
                let _ = std::fs::write(self.policy_path.join("scaling_max_freq"), max.trim());
            }
            logger.debug(&format!("CPU: normal mode {}/{}", min.trim(), max.trim()));
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
    fn test_cpu_apply_modes_writes_files() {
        let tmp = env::temp_dir().join(format!(
            "uconsole_sleep_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = fs::create_dir_all(&tmp);

        let cpu = CpuFreqConfig::with_policy_path(tmp.clone(), Some(String::from("100,400")));
        let logger = Logger::new(false);
        cpu.apply_saving_mode(&logger, false);
        let min = fs::read_to_string(tmp.join("scaling_min_freq")).unwrap();
        let max = fs::read_to_string(tmp.join("scaling_max_freq")).unwrap();
        assert_eq!(min, "100000");
        assert_eq!(max, "400000");

        cpu.apply_normal_mode(&logger, false);
        let min2 = fs::read_to_string(tmp.join("scaling_min_freq")).unwrap();
        let max2 = fs::read_to_string(tmp.join("scaling_max_freq")).unwrap();
        assert_eq!(min2.trim(), "100000");
        assert_eq!(max2.trim(), "400000");
    }
}
