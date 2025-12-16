//! Power mode helper - combines display toggling with CPU frequency changes

use crate::hardware::{backlight, drm_panel, framebuffer};
use crate::{BTConfig, CpuFreqConfig, WifiConfig};
use log::{debug, info, warn};
use std::fs;

fn set_display_on(dry_run: bool) -> Result<(), String> {
    let backlight_path = match backlight::find_backlight() {
        Ok(Some(p)) => p,
        Ok(None) => return Err("backlight not found".to_string()),
        Err(e) => return Err(format!("failed to find backlight: {}", e)),
    };

    let framebuffer_path = framebuffer::find_framebuffer().ok().flatten();
    let drm_path = drm_panel::find_drm_panel().ok().flatten();

    info!("Turning display ON");
    if !dry_run {
        if let Some(fb) = framebuffer_path {
            let _ = fs::write(fb.join("blank"), "0");
        }
        let _ = fs::write(backlight_path.join("bl_power"), "0");
        if let Some(drm) = drm_path {
            let _ = fs::write(drm.join("status"), "detect");
        }
    } else {
        debug!("DRY-RUN: display ON skipped");
    }
    Ok(())
}

fn set_display_off(dry_run: bool) -> Result<(), String> {
    let backlight_path = match backlight::find_backlight() {
        Ok(Some(p)) => p,
        Ok(None) => return Err("backlight not found".to_string()),
        Err(e) => return Err(format!("failed to find backlight: {}", e)),
    };

    let framebuffer_path = framebuffer::find_framebuffer().ok().flatten();
    let drm_path = drm_panel::find_drm_panel().ok().flatten();

    info!("Turning display OFF");
    if !dry_run {
        if let Some(drm) = drm_path {
            let _ = fs::write(drm.join("status"), "off");
        }
        if let Some(fb) = framebuffer_path {
            let _ = fs::write(fb.join("blank"), "1");
        }
        let _ = fs::write(backlight_path.join("bl_power"), "4");
    } else {
        debug!("DRY-RUN: display OFF skipped");
    }
    Ok(())
}

#[allow(dead_code)]
/// Toggle display based on current hardware state
fn toggle_display(dry_run: bool) -> Result<(), String> {
    let backlight_path = match backlight::find_backlight() {
        Ok(Some(p)) => p,
        Ok(None) => return Err("backlight not found".to_string()),
        Err(e) => return Err(format!("failed to find backlight: {}", e)),
    };

    let bl_state =
        fs::read_to_string(backlight_path.join("bl_power")).unwrap_or_else(|_| "4".to_string());
    let bl_state_trim = bl_state.trim();

    if bl_state_trim == "4" {
        // Currently reports ON -> ensure it's ON
        set_display_on(dry_run)
    } else {
        // Currently reports OFF -> ensure it's OFF
        set_display_off(dry_run)
    }
}

/// Power saving mode state
#[derive(Clone, Debug, PartialEq)]
pub enum PowerMode {
    Normal,
    Saving,
}

pub fn enter_saving_mode(
    cpu_config: &CpuFreqConfig,
    dry_run: bool,
    wifi: Option<&WifiConfig>,
    bt: Option<&BTConfig>,
) {
    info!("Entering power-saving mode");
    if let Err(e) = set_display_off(dry_run) {
        warn!("set_display_off failed: {}", e);
    }
    cpu_config.apply_saving_mode(dry_run);
    if let Some(w) = wifi {
        w.block(dry_run);
    }
    if let Some(b) = bt {
        b.block(dry_run);
    }
}

/// Exit power-saving mode: restore CPU then turn display on
pub fn exit_saving_mode(
    cpu_config: &CpuFreqConfig,
    dry_run: bool,
    wifi: Option<&WifiConfig>,
    bt: Option<&BTConfig>,
) {
    info!("Exiting power-saving mode");
    cpu_config.apply_normal_mode(dry_run);
    if let Err(e) = set_display_on(dry_run) {
        warn!("set_display_on failed: {}", e);
    }
    if let Some(w) = wifi {
        w.unblock(dry_run);
    }
    if let Some(b) = bt {
        b.unblock(dry_run);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        // Dry run should not create policy files
        enter_saving_mode(&cpu, true, None, None);
        assert!(!tmp.join("scaling_min_freq").exists());
        assert!(!tmp.join("scaling_max_freq").exists());

        // Non-dry-run should write
        enter_saving_mode(&cpu, false, None, None);
        assert!(tmp.join("scaling_min_freq").exists());
        assert!(tmp.join("scaling_max_freq").exists());

        // exit - verify it doesn't panic
        exit_saving_mode(&cpu, false, None, None);
    }
}
