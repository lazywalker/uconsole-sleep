# uconsole-sleep

<!-- Repository badges -->
[![Rust](https://img.shields.io/badge/rust-2024--edition-orange)](https://www.rust-lang.org/)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://opensource.org/licenses/GPL-3.0)
[![CI](https://github.com/lazywalker/uconsole-sleep/actions/workflows/ci.yml/badge.svg)](https://github.com/lazywalker/uconsole-sleep/actions)
[![crates.io](https://img.shields.io/crates/v/uconsole-sleep.svg)](https://crates.io/crates/uconsole-sleep)
[![docs.rs](https://docs.rs/uconsole-sleep/badge.svg)](https://docs.rs/uconsole-sleep)
[![codecov](https://codecov.io/gh/lazywalker/uconsole-sleep/branch/master/graph/badge.svg)](https://codecov.io/gh/lazywalker/uconsole-sleep)
[![Dependabot](https://img.shields.io/badge/Dependabot-enabled-brightgreen.svg)](https://github.com/lazywalker/uconsole-sleep/network/updates)
[![Maintenance](https://img.shields.io/maintenance/yes/2025)](https://github.com/lazywalker/uconsole-sleep)


This is a Rust port of [uConsole-sleep](https://github.com/qkdxorjs1002/uConsole-sleep). It provides a power key monitor that toggles between normal and power-saving modes. Passed tests on uConsole cm4 with RPI Trixie OS.

Binary:
- Monitor the power key and toggle power-saving mode on short press.

Power-saving mode includes:
- Display off (backlight control via sysfs)
- Reduced CPU frequency (configurable via `SAVING_CPU_FREQ`)
- Future extensibility: WiFi control, Bluetooth control, etc.

Environment variables:
- `SAVING_CPU_FREQ` — set to `min,max` in MHz (e.g. `100,600`) to apply when in power-saving mode
- `HOLD_TRIGGER_SEC` — float seconds to treat as a long press (default 0.7)

Build:
```bash
cargo build --release
```

Usage (run as root to write sysfs, grab input device, and manage power):
```bash
sudo ./target/release/uconsole-sleep
```

# To override the configuration location

default `/etc/uconsole-sleep/config` or repo `./etc/uconsole-sleep/config.default`

sudo ./target/release/uconsole-sleep --config /path/to/config
 - Use `RUST_LOG` environment variable to control logging level (e.g. `RUST_LOG=debug`) or CLI flags `-v` (info), `-vv` (debug), `-vvv` (trace).
 - Run `uconsole-sleep -h` or `uconsole-sleep --help` to print usage and available options such as `--dry-run`, `--toggle-wifi`, `--toggle-bt`, and `--config`.

Examples:
```bash
# Dry run (no writes)
sudo ./target/release/uconsole-sleep --dry-run
# Show help
sudo ./target/release/uconsole-sleep --help

```

How it works:
- Press power key (short press < 0.7s): toggle between normal and power-saving mode
- Power-saving mode: turns off display, reduces CPU frequency
- Normal mode: turns on display, restores default CPU frequency
- The program grabs exclusive access to the power key device to prevent LXDE from triggering shutdown dialogs

Notes:
- This implementation uses sysfs writes to toggle display and CPU frequency
- The power key device is grabbed (EVIOCGRAB) to prevent desktop environment conflicts
- Tests cover hardware detection helpers
