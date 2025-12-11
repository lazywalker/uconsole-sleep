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

use log::{Level, LevelFilter, debug, error, info, warn};
use uconsole_sleep::hardware::power_key;

use uconsole_sleep::CpuFreqConfig;
use uconsole_sleep::WifiConfig;
use uconsole_sleep::args::parse_cli_args;
use uconsole_sleep::config::Config;
use uconsole_sleep::power_mode::{PowerMode, enter_saving_mode, exit_saving_mode};

// EVIOCGRAB ioctl to grab exclusive access to input device
const EVIOCGRAB: u64 = 0x40044590;

// Use PowerMode and enter/exit functions from the library `power_mode` module.

fn resolve_log_level(
    rust_log_env: Option<String>,
    verbosity: u8,
    cfg_level: Option<Level>,
) -> Option<LevelFilter> {
    // RUST_LOG present and non-empty has highest priority (return None to indicate env value should be used)
    if let Some(ref s) = rust_log_env
        && !s.trim().is_empty()
    {
        return None;
    }
    // CLI verbosity next
    match verbosity {
        1 => Some(LevelFilter::Info),
        2 => Some(LevelFilter::Debug),
        3 => Some(LevelFilter::Trace),
        _ => cfg_level.map(|l| l.to_level_filter()),
    }
}

fn main() {
    // parse basic CLI flags
    let (dry_run, verbosity, toggle_wifi_flag, cli_config_path) = parse_cli_args();

    // Read configuration (env vars + config file)
    let cfg = Config::load(cli_config_path.clone());

    // Initialize env_logger; precedence: RUST_LOG (env) > CLI verbosity (-v) > config.log_level
    let mut builder = env_logger::builder();
    let rust_log_env = std::env::var("RUST_LOG").ok();
    let resolved = resolve_log_level(rust_log_env.clone(), verbosity, cfg.log_level);
    if let Some(ref rust_val) = rust_log_env
        && !rust_val.trim().is_empty()
    {
        builder.parse_filters(rust_val);
    } else if let Some(l) = resolved {
        builder.filter_level(l);
    }
    let _ = builder.try_init();
    info!("Starting sleep-remap-powerkey (power-saving mode toggle)");

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

    debug!("cli.dry_run={}", dry_run);
    debug!("cli.policy_path={}", cli_policy_str);
    debug!("cli.config_path={}", cli_config_str);
    debug!("cli.toggle_wifi={:?}", toggle_wifi_flag);
    debug!("cli.wifi_rfkill={}", wifi_rfkill_cli_str);

    debug!("cfg.dry_run={}", cfg.dry_run);
    debug!("cfg.policy_path={}", cfg_policy_str);
    debug!("cfg.saving_cpu_freq={:?}", cfg.saving_cpu_freq);
    debug!("cfg.hold_trigger_sec={:?}", cfg.hold_trigger_sec);
    debug!("cfg.toggle_wifi={}", cfg.toggle_wifi);
    debug!("cfg.wifi_rfkill={}", cfg_wifi_rfkill_str);

    debug!("derived.hold_trigger_s={:.3}", hold_trigger.as_secs_f32());
    debug!("derived.saving_cpu_freq={:?}", saving_cpu_freq);
    debug!(
        "derived.cpu_policy_path={}",
        cpu_config.policy_path.display()
    );
    debug!("derived.cpu_saving_min={:?}", cpu_config.saving_min);
    debug!("derived.cpu_saving_max={:?}", cpu_config.saving_max);
    debug!("derived.cpu_default_min={:?}", cpu_config.default_min);
    debug!("derived.cpu_default_max={:?}", cpu_config.default_max);
    debug!("derived.final_toggle_wifi={}", final_toggle_wifi);
    debug!(
        "derived.final_wifi_rfkill={}",
        opt_to_str(&final_wifi_rfkill)
    );

    let dev = match power_key::find_power_key() {
        Ok(Some(p)) => p,
        Ok(None) => {
            error!("Power key device not found, exiting");
            return;
        }
        Err(e) => {
            error!("Failed to find power key: {}", e);
            return;
        }
    };

    info!("Using device {}", dev.display());

    let mut file = match File::open(&dev) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open device {}: {}", dev.display(), e);
            return;
        }
    };

    // Grab exclusive access to prevent LXDE from receiving power key events
    let fd = file.as_raw_fd();
    unsafe {
        let ret = libc::ioctl(fd, EVIOCGRAB as _, 1);
        if ret != 0 {
            warn!("Failed to grab exclusive access to power key device");
            warn!("LXDE may still receive power key events");
        } else {
            info!("Successfully grabbed exclusive access to power key device");
        }
    }

    // input_event struct is 24 bytes (2x i64 + u16 + u16 + i32)
    let mut buf = [0u8; 24];
    let mut last_key_down_timestamp: Option<Instant> = None;

    // Setup epoll
    let epoll = match Epoll::new(EpollCreateFlags::empty()) {
        Ok(e) => e,
        Err(e) => {
            error!("Failed to create epoll instance: {}", e);
            return;
        }
    };

    let event = EpollEvent::new(EpollFlags::EPOLLIN, 0);
    if let Err(e) = epoll.add(&file, event) {
        error!("Failed to add input device to epoll: {}", e);
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

                                debug!(
                                    "event: t={} ms={} type={} code={} value={}",
                                    sec, usec, etype, code, value
                                );

                                // KEY_POWER is 116
                                if etype == 1 && code == 116 {
                                    if value == 1 {
                                        info!("Power key down detected");
                                        last_key_down_timestamp = Some(Instant::now());
                                    } else if value == 0 {
                                        info!("Power key up detected");
                                        if let Some(down_ts) = last_key_down_timestamp {
                                            let elapsed = down_ts.elapsed();
                                            if elapsed < hold_trigger {
                                                // short press -> toggle power mode
                                                let mode_clone = Arc::clone(&power_mode);
                                                let cpu_config_clone = cpu_config.clone();
                                                let dry_run_clone = dry_run;
                                                /* no logger clone needed, using log macros */
                                                let wifi_config_clone = wifi_config.clone();

                                                spawn(move || {
                                                    let mut mode = mode_clone.lock().unwrap();
                                                    // read dry-run from env variable to avoid adding a global flag variable
                                                    // `dry_run_clone` is passed in earlier from outer scope
                                                    match *mode {
                                                        PowerMode::Normal => {
                                                            enter_saving_mode(
                                                                &cpu_config_clone,
                                                                dry_run_clone,
                                                                Some(&wifi_config_clone),
                                                            );
                                                            *mode = PowerMode::Saving;
                                                        }
                                                        PowerMode::Saving => {
                                                            exit_saving_mode(
                                                                &cpu_config_clone,
                                                                dry_run_clone,
                                                                Some(&wifi_config_clone),
                                                            );
                                                            *mode = PowerMode::Normal;
                                                        }
                                                    }
                                                });
                                            } else {
                                                info!(
                                                    "Long press detected (no action implemented)",
                                                );
                                            }
                                        }
                                        last_key_down_timestamp = None;
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Error reading event: {}", e);
                                sleep(Duration::from_millis(200));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("epoll_wait error: {}", e);
                sleep(Duration::from_millis(500));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{Level, LevelFilter};

    #[test]
    fn resolve_prefers_rust_env() {
        let resolved = resolve_log_level(Some("info".to_string()), 3, Some(Level::Debug));
        assert_eq!(resolved, None);
    }

    #[test]
    fn resolve_verbosity_over_config() {
        let resolved = resolve_log_level(None, 1, Some(Level::Debug));
        assert_eq!(resolved, Some(LevelFilter::Info));
    }

    #[test]
    fn resolve_config_when_no_env_no_verbosity() {
        let resolved = resolve_log_level(None, 0, Some(Level::Warn));
        assert_eq!(resolved, Some(LevelFilter::Warn));
    }
}
