//! Sleep remap power key service
//! Power key press toggles between normal and power-saving mode.
//! Power-saving mode: display off, WiFi off(optional), reduced CPU frequency

use nix::sys::epoll::EpollTimeout;
use std::env;
use std::fs::File;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

use nix::sys::epoll::{Epoll, EpollCreateFlags, EpollEvent, EpollFlags};

use uconsole_sleep::hardware::power_key;
use uconsole_sleep::logger::Logger;

use uconsole_sleep::config::Config;
use uconsole_sleep::cpu::CpuFreqConfig;
use uconsole_sleep::hardware::wifi::WifiConfig;
use uconsole_sleep::power_mode::{PowerMode, enter_saving_mode, exit_saving_mode};

// EVIOCGRAB ioctl to grab exclusive access to input device
const EVIOCGRAB: u64 = 0x40044590;

// Use PowerMode and enter/exit functions from the library `power_mode` module.

/// Parse CLI args for a minimal set: --dry-run, --debug, --policy-path <path>, --config <path>
fn parse_cli_args_from<I: IntoIterator<Item = String>>(
    args: I,
) -> (bool, bool, Option<bool>, Option<PathBuf>) {
    let mut dry_run = false;
    let mut debug = false;
    let mut config_path: Option<PathBuf> = None;
    let mut toggle_wifi: Option<bool> = None;
    // --policy-path and --wifi-rfkill are deprecated; config will be used instead
    let mut iter = args.into_iter();
    while let Some(a) = iter.next() {
        match a.as_str() {
            "--dry-run" => dry_run = true,
            "--debug" | "-v" | "--verbose" => debug = true,
            s if s.starts_with("--toggle-wifi") => {
                if s == "--toggle-wifi" {
                    toggle_wifi = Some(true);
                } else if let Some(eq) = s.find('=') {
                    let val = s[eq + 1..].to_ascii_lowercase();
                    toggle_wifi = Some(val == "true" || val == "1" || val == "yes");
                }
            }
            s if s.starts_with("--config") => {
                if s == "--config" {
                    if let Some(p) = iter.next() {
                        config_path = Some(PathBuf::from(p));
                    }
                } else if let Some(eq) = s.find('=') {
                    let p = &s[eq + 1..];
                    if !p.is_empty() {
                        config_path = Some(PathBuf::from(p));
                    }
                }
            }
            _ => {}
        }
    }
    (dry_run, debug, toggle_wifi, config_path)
}

fn parse_cli_args() -> (bool, bool, Option<bool>, Option<PathBuf>) {
    parse_cli_args_from(std::env::args())
}

fn main() {
    // parse basic CLI flags
    let (dry_run, cli_debug_flag, toggle_wifi_flag, cli_config_path) = parse_cli_args();

    // Read configuration (env vars + config file)
    let cfg = Config::load(cli_config_path.clone());

    // Determine final debug flag: CLI flag takes precedence over config file
    let final_debug_flag = cli_debug_flag || cfg.debug;
    let logger = Logger::new(final_debug_flag);
    logger.info("Starting sleep-remap-powerkey (power-saving mode toggle)");

    let hold_trigger = Duration::from_secs_f32(
        cfg.hold_trigger_sec
            .or_else(|| {
                env::var("HOLD_TRIGGER_SEC")
                    .ok()
                    .and_then(|s| s.parse::<f32>().ok())
            })
            .unwrap_or(0.7),
    );

    // Track current power mode (shared between threads)
    let power_mode = Arc::new(Mutex::new(PowerMode::Normal));

    // Setup CPU frequency configuration, prefer CLI flag override if provided
    let saving_cpu_freq = cfg
        .saving_cpu_freq
        .clone()
        .or_else(|| env::var("SAVING_CPU_FREQ").ok());
    let cpu_config = if let Some(path) = cfg.policy_path.clone() {
        CpuFreqConfig::with_policy_path(path, saving_cpu_freq.clone())
    } else {
        CpuFreqConfig::new(saving_cpu_freq.clone())
    };
    // Determine wifi config: CLI flag overrides config file; use clones to avoid moving original variables used for logging
    let final_toggle_wifi = match toggle_wifi_flag {
        Some(v) => v,
        None => cfg.toggle_wifi,
    };
    let final_wifi_rfkill = cfg.wifi_rfkill_path.clone();
    let wifi_config = WifiConfig::new(final_toggle_wifi, final_wifi_rfkill.clone());

    // Print all parameters for startup debugging (capture a string for options to avoid moves)
    let opt_to_str = |p: &Option<PathBuf>| match p {
        Some(pp) => pp.display().to_string(),
        None => "None".to_string(),
    };
    let cli_policy_str = "None".to_string();
    let wifi_rfkill_cli_str = "None".to_string();
    let cli_config_str = opt_to_str(&cli_config_path);
    let cfg_policy_str = opt_to_str(&cfg.policy_path);
    let cfg_wifi_rfkill_str = opt_to_str(&cfg.wifi_rfkill_path);

    // Log start-up parameters only when debug was requested. One parameter per line for readability.
    if final_debug_flag {
        logger.debug(&format!("cli.dry_run={}", dry_run));
        logger.debug(&format!("cli.debug={}", cli_debug_flag));
        logger.debug(&format!("cli.policy_path={}", cli_policy_str));
        logger.debug(&format!("cli.config_path={}", cli_config_str));
        logger.debug(&format!("cli.toggle_wifi={:?}", toggle_wifi_flag));
        logger.debug(&format!("cli.wifi_rfkill={}", wifi_rfkill_cli_str));

        logger.debug(&format!("cfg.dry_run={}", cfg.dry_run));
        logger.debug(&format!("cfg.debug={}", cfg.debug));
        logger.debug(&format!("cfg.policy_path={}", cfg_policy_str));
        logger.debug(&format!("cfg.saving_cpu_freq={:?}", cfg.saving_cpu_freq));
        logger.debug(&format!("cfg.hold_trigger_sec={:?}", cfg.hold_trigger_sec));
        logger.debug(&format!("cfg.toggle_wifi={}", cfg.toggle_wifi));
        logger.debug(&format!("cfg.wifi_rfkill={}", cfg_wifi_rfkill_str));

        logger.debug(&format!(
            "derived.hold_trigger_s={:.3}",
            hold_trigger.as_secs_f32()
        ));
        logger.debug(&format!("derived.saving_cpu_freq={:?}", saving_cpu_freq));
        logger.debug(&format!(
            "derived.cpu_policy_path={}",
            cpu_config.policy_path.display()
        ));
        logger.debug(&format!(
            "derived.cpu_saving_min={:?}",
            cpu_config.saving_min
        ));
        logger.debug(&format!(
            "derived.cpu_saving_max={:?}",
            cpu_config.saving_max
        ));
        logger.debug(&format!(
            "derived.cpu_default_min={:?}",
            cpu_config.default_min
        ));
        logger.debug(&format!(
            "derived.cpu_default_max={:?}",
            cpu_config.default_max
        ));
        logger.debug(&format!("derived.final_toggle_wifi={}", final_toggle_wifi));
        logger.debug(&format!(
            "derived.final_wifi_rfkill={}",
            opt_to_str(&final_wifi_rfkill)
        ));
    }

    let dev = match power_key::find_power_key() {
        Ok(Some(p)) => p,
        Ok(None) => {
            logger.error("Power key device not found, exiting");
            return;
        }
        Err(e) => {
            logger.error(&format!("Failed to find power key: {}", e));
            return;
        }
    };

    logger.info(&format!("Using device {}", dev.display()));

    let mut file = match File::open(&dev) {
        Ok(f) => f,
        Err(e) => {
            logger.error(&format!("Failed to open device {}: {}", dev.display(), e));
            return;
        }
    };

    // Grab exclusive access to prevent LXDE from receiving power key events
    let fd = file.as_raw_fd();
    unsafe {
        let ret = libc::ioctl(fd, EVIOCGRAB as _, 1);
        if ret != 0 {
            logger.warn("Failed to grab exclusive access to power key device");
            logger.warn("LXDE may still receive power key events");
        } else {
            logger.info("Successfully grabbed exclusive access to power key device");
        }
    }

    // input_event struct is 24 bytes (2x i64 + u16 + u16 + i32)
    let mut buf = [0u8; 24];
    let mut last_key_down_timestamp: Option<Instant> = None;

    // Setup epoll
    let epoll = match Epoll::new(EpollCreateFlags::empty()) {
        Ok(e) => e,
        Err(e) => {
            logger.error(&format!("Failed to create epoll instance: {}", e));
            return;
        }
    };

    let event = EpollEvent::new(EpollFlags::EPOLLIN, 0);
    if let Err(e) = epoll.add(&file, event) {
        logger.error(&format!("Failed to add input device to epoll: {}", e));
        return;
    }

    loop {
        let mut events = vec![EpollEvent::new(EpollFlags::empty(), 0); 4];
        match epoll.wait(&mut events, EpollTimeout::NONE) {
            Ok(num) => {
                for ev in &events[..num] {
                    if ev.events().contains(EpollFlags::EPOLLIN) {
                        match file.read_exact(&mut buf) {
                            Ok(_) => {
                                let sec = i64::from_ne_bytes(buf[0..8].try_into().unwrap());
                                let usec = i64::from_ne_bytes(buf[8..16].try_into().unwrap());
                                let etype = u16::from_ne_bytes(buf[16..18].try_into().unwrap());
                                let code = u16::from_ne_bytes(buf[18..20].try_into().unwrap());
                                let value = i32::from_ne_bytes(buf[20..24].try_into().unwrap());

                                logger.debug(&format!(
                                    "event: t={} ms={} type={} code={} value={}",
                                    sec, usec, etype, code, value
                                ));

                                // KEY_POWER is 116
                                if etype == 1 && code == 116 {
                                    if value == 1 {
                                        logger.info("Power key down detected");
                                        last_key_down_timestamp = Some(Instant::now());
                                    } else if value == 0 {
                                        logger.info("Power key up detected");
                                        if let Some(down_ts) = last_key_down_timestamp {
                                            let elapsed = down_ts.elapsed();
                                            if elapsed < hold_trigger {
                                                // short press -> toggle power mode
                                                let mode_clone = Arc::clone(&power_mode);
                                                let cpu_config_clone = cpu_config.clone();
                                                let dry_run_clone = dry_run;
                                                let logger_clone = Logger::new(true);
                                                let wifi_config_clone = wifi_config.clone();

                                                spawn(move || {
                                                    let mut mode = mode_clone.lock().unwrap();
                                                    // read dry-run from env variable to avoid adding a global flag variable
                                                    // `dry_run_clone` is passed in earlier from outer scope
                                                    match *mode {
                                                        PowerMode::Normal => {
                                                            enter_saving_mode(
                                                                &cpu_config_clone,
                                                                &logger_clone,
                                                                dry_run_clone,
                                                                Some(&wifi_config_clone),
                                                            );
                                                            *mode = PowerMode::Saving;
                                                        }
                                                        PowerMode::Saving => {
                                                            exit_saving_mode(
                                                                &cpu_config_clone,
                                                                &logger_clone,
                                                                dry_run_clone,
                                                                Some(&wifi_config_clone),
                                                            );
                                                            *mode = PowerMode::Normal;
                                                        }
                                                    }
                                                });
                                            } else {
                                                logger.info(
                                                    "Long press detected (no action implemented)",
                                                );
                                            }
                                        }
                                        last_key_down_timestamp = None;
                                    }
                                }
                            }
                            Err(e) => {
                                logger.warn(&format!("Error reading event: {}", e));
                                sleep(Duration::from_millis(200));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                logger.warn(&format!("epoll_wait error: {}", e));
                sleep(Duration::from_millis(500));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cli_args_from_flags() {
        let tmp = std::env::temp_dir().join(format!(
            "uconsole_cli_cfg_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = std::fs::create_dir_all(&tmp);
        let cfg_path = tmp.join("cli_cfg");
        std::fs::write(&cfg_path, "SAVING_CPU_FREQ=55,66\nHOLD_TRIGGER_SEC=1.4\n").unwrap();

        let args = vec![
            String::from("prog"),
            String::from("--dry-run"),
            String::from("--debug"),
            String::from("--config"),
            cfg_path.to_string_lossy().to_string(),
        ];
        let (dry_run, debug, _toggle_wifi, cli_config_path) = parse_cli_args_from(args);
        assert!(dry_run);
        assert!(debug);
        assert_eq!(cli_config_path, Some(cfg_path.clone()));

        // ensure the Config::load uses this file when provided
        let loaded = Config::load(cli_config_path.clone());
        assert_eq!(loaded.saving_cpu_freq.unwrap(), "55,66");
        assert_eq!(loaded.hold_trigger_sec.unwrap(), 1.4_f32);
        // no-op; this used to check default examples
    }

    #[test]
    fn test_parse_cli_args_from_flags_eq_form() {
        let tmp = std::env::temp_dir().join(format!(
            "uconsole_cli_cfg_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = std::fs::create_dir_all(&tmp);
        let cfg_path = tmp.join("cli_cfg2");
        std::fs::write(&cfg_path, "SAVING_CPU_FREQ=22,33\nHOLD_TRIGGER_SEC=2.1\n").unwrap();

        let args = vec![
            String::from("prog"),
            String::from("--dry-run"),
            String::from("--debug"),
            format!("--config={}", cfg_path.to_string_lossy()),
        ];
        let (dry_run, debug, _toggle_wifi, cli_config_path) = parse_cli_args_from(args);
        assert!(dry_run);
        assert!(debug);
        assert_eq!(cli_config_path, Some(cfg_path.clone()));
        let loaded = Config::load(cli_config_path.clone());
        assert_eq!(loaded.saving_cpu_freq.unwrap(), "22,33");
        assert_eq!(loaded.hold_trigger_sec.unwrap(), 2.1_f32);
    }

    #[test]
    fn test_toggle_wifi_cli_precedence_over_config() {
        use uconsole_sleep::config::Config;
        let mut cfg = Config::default();
        cfg.toggle_wifi = true; // config enables wifi
        let toggle_wifi_flag = Some(false);
        let final_toggle_wifi = match toggle_wifi_flag {
            Some(v) => v,
            None => cfg.toggle_wifi,
        };
        assert_eq!(final_toggle_wifi, false);
    }
}
