//! Power mode helper - combines display toggling with CPU frequency changes

use crate::hardware::{backlight, drm_panel, framebuffer};
use crate::logger::Logger;
use crate::{WifiConfig, cpu::CpuFreqConfig};
use std::fs;

fn toggle_display(logger: &Logger, dry_run: bool) -> Result<(), String> {
    let backlight_path = match backlight::find_backlight() {
        Ok(Some(p)) => p,
        Ok(None) => return Err("backlight not found".to_string()),
        Err(e) => return Err(format!("failed to find backlight: {}", e)),
    };

    let framebuffer_path = framebuffer::find_framebuffer().ok().flatten();
    let drm_path = drm_panel::find_drm_panel().ok().flatten();

    let bl_state =
        fs::read_to_string(backlight_path.join("bl_power")).unwrap_or_else(|_| "4".to_string());
    let bl_state_trim = bl_state.trim();

    if bl_state_trim == "4" {
        // ON
        logger.info("Turning display ON");
        if !dry_run {
            if let Some(fb) = framebuffer_path {
                let _ = fs::write(fb.join("blank"), "0");
            }
            let _ = fs::write(backlight_path.join("bl_power"), "0");
            if let Some(drm) = drm_path {
                let _ = fs::write(drm.join("status"), "detect");
            }
        } else {
            logger.debug("DRY-RUN: toggle_display ON skipped");
        }
    } else {
        // OFF
        logger.info("Turning display OFF");
        if !dry_run {
            if let Some(drm) = drm_path {
                let _ = fs::write(drm.join("status"), "off");
            }
            if let Some(fb) = framebuffer_path {
                let _ = fs::write(fb.join("blank"), "1");
            }
            let _ = fs::write(backlight_path.join("bl_power"), "4");
        } else {
            logger.debug("DRY-RUN: toggle_display OFF skipped");
        }
    }
    Ok(())
}

/// Power saving mode state
#[derive(Clone, Debug, PartialEq)]
pub enum PowerMode {
    Normal,
    Saving,
}

pub fn enter_saving_mode(
    cpu_config: &CpuFreqConfig,
    logger: &Logger,
    dry_run: bool,
    wifi: Option<&WifiConfig>,
) {
    logger.info("Entering power-saving mode");
    if let Err(e) = toggle_display(logger, dry_run) {
        logger.warn(&format!("toggle_display failed: {}", e));
    }
    cpu_config.apply_saving_mode(logger, dry_run);
    if let Some(w) = wifi {
        w.block(logger, dry_run);
    }
}

/// Exit power-saving mode: restore CPU then turn display on
pub fn exit_saving_mode(
    cpu_config: &CpuFreqConfig,
    logger: &Logger,
    dry_run: bool,
    wifi: Option<&WifiConfig>,
) {
    logger.info("Exiting power-saving mode");
    cpu_config.apply_normal_mode(logger, dry_run);
    if let Err(e) = toggle_display(logger, dry_run) {
        logger.warn(&format!("toggle_display failed: {}", e));
    }
    if let Some(w) = wifi {
        w.unblock(logger, dry_run);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::Logger;
    use std::env;
    use std::fs;

    #[test]
    fn test_enter_exit_saving_mode_dryrun() {
        let tmp = env::temp_dir().join(format!(
            "uconsole_pm_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = fs::create_dir_all(&tmp);
        let cpu = CpuFreqConfig::with_policy_path(tmp.clone(), Some(String::from("100,200")));
        let logger = Logger::new(false);

        // Dry run should not create policy files
        enter_saving_mode(&cpu, &logger, true, None);
        assert!(!tmp.join("scaling_min_freq").exists());
        assert!(!tmp.join("scaling_max_freq").exists());

        // Non-dry-run should write
        enter_saving_mode(&cpu, &logger, false, None);
        assert!(tmp.join("scaling_min_freq").exists());
        assert!(tmp.join("scaling_max_freq").exists());

        // exit - verify it doesn't panic
        exit_saving_mode(&cpu, &logger, false, None);
    }
}
