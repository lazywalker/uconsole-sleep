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

    /// Unique temp dir helper scoped to a test by name, so tests don't collide.
    fn tmp_dir(name: &str) -> std::path::PathBuf {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let dir = env::temp_dir().join(format!("uconsole_{name}_{ms}"));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    /// Drive `enter_saving_mode` / `exit_saving_mode` against cpu + wifi + bt backed by
    /// temp directories and assert each subsystem's final on-disk state. This verifies
    /// the orchestration without depending on real sysfs display paths.
    #[test]
    fn test_enter_exit_full_state_cpu_wifi_bt() {
        // CPU policy dir: seed the *default* values so exit_saving_mode can restore them.
        let cpu_dir = tmp_dir("pm_cpu");
        fs::write(cpu_dir.join("scaling_min_freq"), "600000\n").unwrap();
        fs::write(cpu_dir.join("scaling_max_freq"), "1800000\n").unwrap();
        let cpu = CpuFreqConfig::with_policy_path(cpu_dir.clone(), Some(String::from("100,600")));

        // WiFi and BT rfkill dirs backed by a writable "state" file.
        let wifi_dir = tmp_dir("pm_wifi");
        let bt_dir = tmp_dir("pm_bt");
        // default (unblocked) state
        fs::write(wifi_dir.join("state"), "1").unwrap();
        fs::write(bt_dir.join("state"), "1").unwrap();
        let wifi = WifiConfig::new(true, Some(wifi_dir.clone()));
        let bt = BTConfig::new(true, Some(bt_dir.clone()));

        // --- enter saving mode ---
        enter_saving_mode(&cpu, false, Some(&wifi), Some(&bt));

        // CPU clamped to saving range
        assert_eq!(
            fs::read_to_string(cpu_dir.join("scaling_min_freq")).unwrap(),
            "100000"
        );
        assert_eq!(
            fs::read_to_string(cpu_dir.join("scaling_max_freq")).unwrap(),
            "600000"
        );
        // WiFi and BT blocked
        assert_eq!(fs::read_to_string(wifi_dir.join("state")).unwrap(), "0");
        assert_eq!(fs::read_to_string(bt_dir.join("state")).unwrap(), "0");

        // --- exit saving mode ---
        exit_saving_mode(&cpu, false, Some(&wifi), Some(&bt));

        // CPU restored to the defaults seeded above
        assert_eq!(
            fs::read_to_string(cpu_dir.join("scaling_min_freq"))
                .unwrap()
                .trim(),
            "600000"
        );
        assert_eq!(
            fs::read_to_string(cpu_dir.join("scaling_max_freq"))
                .unwrap()
                .trim(),
            "1800000"
        );
        // WiFi and BT unblocked
        assert_eq!(fs::read_to_string(wifi_dir.join("state")).unwrap(), "1");
        assert_eq!(fs::read_to_string(bt_dir.join("state")).unwrap(), "1");
    }

    /// With wifi/bt enabled but no rfkill path, enter/exit must not panic and CPU still
    /// transitions (the missing rfkill only produces a warning at runtime).
    #[test]
    fn test_enter_exit_with_disabled_rf() {
        let cpu_dir = tmp_dir("pm_norf");
        fs::write(cpu_dir.join("scaling_min_freq"), "400000\n").unwrap();
        fs::write(cpu_dir.join("scaling_max_freq"), "1400000\n").unwrap();
        let cpu = CpuFreqConfig::with_policy_path(cpu_dir.clone(), Some(String::from("100,400")));
        // rfkill disabled: no path
        let wifi = WifiConfig::new(false, None);
        let bt = BTConfig::new(false, None);

        enter_saving_mode(&cpu, false, Some(&wifi), Some(&bt));
        assert_eq!(
            fs::read_to_string(cpu_dir.join("scaling_max_freq")).unwrap(),
            "400000"
        );
        exit_saving_mode(&cpu, false, Some(&wifi), Some(&bt));
        assert_eq!(
            fs::read_to_string(cpu_dir.join("scaling_max_freq"))
                .unwrap()
                .trim(),
            "1400000"
        );
    }

    /// Dry-run must leave every subsystem untouched: no CPU writes, no rfkill writes.
    #[test]
    fn test_dry_run_writes_nothing() {
        let cpu_dir = tmp_dir("pm_dry");
        let cpu = CpuFreqConfig::with_policy_path(cpu_dir.clone(), Some(String::from("100,400")));
        let wifi_dir = tmp_dir("pm_dry_wifi");
        let bt_dir = tmp_dir("pm_dry_bt");
        fs::write(wifi_dir.join("state"), "1").unwrap();
        fs::write(bt_dir.join("state"), "1").unwrap();
        let wifi = WifiConfig::new(true, Some(wifi_dir.clone()));
        let bt = BTConfig::new(true, Some(bt_dir.clone()));

        enter_saving_mode(&cpu, true, Some(&wifi), Some(&bt));
        assert!(!cpu_dir.join("scaling_min_freq").exists());
        assert_eq!(fs::read_to_string(wifi_dir.join("state")).unwrap(), "1");
        assert_eq!(fs::read_to_string(bt_dir.join("state")).unwrap(), "1");
    }
}
