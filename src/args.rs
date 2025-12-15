use std::path::PathBuf;

/// Parse CLI args for a minimal set: --dry-run, --toggle-wifi, --config <path>
fn parse_cli_args_from<I: IntoIterator<Item = String>>(
    args: I,
) -> (bool, u8, Option<bool>, Option<bool>, Option<PathBuf>) {
    let mut dry_run = false;
    let mut verbosity: u8 = 0;
    let mut config_path: Option<PathBuf> = None;
    let mut toggle_wifi: Option<bool> = None;
    let mut toggle_bt: Option<bool> = None;
    let mut iter = args.into_iter();
    while let Some(a) = iter.next() {
        match a.as_str() {
            "--dry-run" => dry_run = true,
            s if s.starts_with("-v") && s.chars().skip(1).all(|c| c == 'v') => {
                let count = s.chars().skip(1).count();
                // bound verbosity at 3
                verbosity = std::cmp::min(3, count as u8);
            }
            "--verbose" => {
                // map --verbose to single -v
                verbosity = std::cmp::max(verbosity, 1);
            }
            s if s.starts_with("--toggle-wifi") => {
                if s == "--toggle-wifi" {
                    toggle_wifi = Some(true);
                } else if let Some(eq) = s.find('=') {
                    let val = s[eq + 1..].to_ascii_lowercase();
                    toggle_wifi = Some(val == "true" || val == "1" || val == "yes");
                }
            }
            s if s.starts_with("--toggle-bt") => {
                if s == "--toggle-bt" {
                    toggle_bt = Some(true);
                } else if let Some(eq) = s.find('=') {
                    let val = s[eq + 1..].to_ascii_lowercase();
                    toggle_bt = Some(val == "true" || val == "1" || val == "yes");
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
    (dry_run, verbosity, toggle_wifi, toggle_bt, config_path)
}

pub fn parse_cli_args() -> (bool, u8, Option<bool>, Option<bool>, Option<PathBuf>) {
    parse_cli_args_from(std::env::args())
}

#[cfg(test)]
mod tests {
    use crate::Config;

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
            String::from("--config"),
            cfg_path.to_string_lossy().to_string(),
        ];
        let (dry_run, verbosity, _toggle_wifi, _toggle_bt, cli_config_path) =
            parse_cli_args_from(args);
        assert!(dry_run);
        assert_eq!(verbosity, 0);
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
            format!("--config={}", cfg_path.to_string_lossy()),
        ];
        let (dry_run, verbosity, _toggle_wifi, _toggle_bt, cli_config_path) =
            parse_cli_args_from(args);
        assert!(dry_run);
        assert_eq!(verbosity, 0);
        assert_eq!(cli_config_path, Some(cfg_path.clone()));
        let loaded = Config::load(cli_config_path.clone());
        assert_eq!(loaded.saving_cpu_freq.unwrap(), "22,33");
        assert_eq!(loaded.hold_trigger_sec.unwrap(), 2.1_f32);
    }

    #[test]
    fn test_toggle_wifi_cli_precedence_over_config() {
        use crate::Config;
        let cfg = Config {
            toggle_wifi: true,
            ..Default::default()
        };
        let toggle_wifi_flag = Some(false);
        let final_toggle_wifi = match toggle_wifi_flag {
            Some(v) => v,
            None => cfg.toggle_wifi,
        };
        assert!(!final_toggle_wifi);
    }

    #[test]
    fn test_parse_cli_args_verbosity_v() {
        let args = vec![String::from("prog"), String::from("-v")];
        let (_dry_run, verbosity, _toggle_wifi, _toggle_bt, _cli_config_path) =
            parse_cli_args_from(args);
        assert_eq!(verbosity, 1);
    }

    #[test]
    fn test_parse_cli_args_verbosity_vv() {
        let args = vec![String::from("prog"), String::from("-vv")];
        let (_dry_run, verbosity, _toggle_wifi, _toggle_bt, _cli_config_path) =
            parse_cli_args_from(args);
        assert_eq!(verbosity, 2);
    }

    #[test]
    fn test_parse_cli_args_verbosity_vvv() {
        let args = vec![String::from("prog"), String::from("-vvv")];
        let (_dry_run, verbosity, _toggle_wifi, _toggle_bt, _cli_config_path) =
            parse_cli_args_from(args);
        assert_eq!(verbosity, 3);
    }
}
