# uconsole-sleep

This is a Rust port of [uConsole-sleep](https://github.com/qkdxorjs1002/uConsole-sleep). It provides a power key monitor that toggles between normal and power-saving modes. Passed tests on uConsole cm4 with RPI Trixie OS.

Binary:
- Monitor the power key and toggle power-saving mode on short press.

Power-saving mode includes:
- Display off (backlight control via sysfs)
- Reduced CPU frequency (configurable via `SAVING_CPU_FREQ`)
- Future extensibility: WiFi control, etc.

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

CLI flags:
- `--dry-run`: log actions without writing to sysfs (useful for debugging)
- `--debug` or `-v`: enable debug logging
- `--policy-path <path>`: use a custom CPU policy path (useful for testing or non-standard systems)

Examples:
```bash
# Dry run (no writes), enable debug logging
sudo ./target/release/uconsole-sleep --dry-run --debug

# Use a fake policy path for testing
sudo ./target/release/uconsole-sleep --policy-path /tmp/fake_policy
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
