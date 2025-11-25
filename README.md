# Stop It - Browser Activity Monitor & Pomodoro Timer

A Rust-based CLI tool that monitors your browser activity and helps you stay focused using the Pomodoro technique.

## Features

- **Active Window Monitoring**: Tracks which browser tab/window is currently active
- **Universal Domain Extraction**: Extracts domains from ANY website (github.com, google.com, docs.rs, etc.)
- **Time Tracking**: Records time spent on each domain during your session
- **Work/Break Mode**: Automatically switches between work (25min) and break (5min) modes
- **File Logging**: Logs all activity to `~/.local/share/stop_it/activity.log` by default
- **Desktop Notifications**: Sends native notifications when it's time to switch modes
- **Session Statistics**: View detailed stats when you press Ctrl+C
- **Verbose Mode**: Debug mode to see what titles are being detected

## Requirements

- Linux with Hyprland (Wayland compositor)
- `hyprctl` command available (included with Hyprland)
- Rust 1.70 or later

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/stop_it`

## Usage

### Basic Usage

```bash
cargo run
# or after building:
./target/release/stop_it
```

### Options

```bash
# Verbose mode (shows all window titles and domain extraction)
cargo run -- --verbose
cargo run -- -v

# Custom log file location
cargo run -- --log /path/to/activity.log
cargo run -- -l /path/to/activity.log

# Combine options
cargo run -- --verbose --log ./my-activity.log
```

### What it does

1. **Starts in WORK mode** (💼) - 25 minutes
2. **Logs domain switches** - Both to terminal and log file
3. **Tracks time per domain** - Accumulates seconds on each site
4. **Notifies on mode switch** - Desktop notification + terminal alert
5. **Switches to BREAK mode** (☕) - 5 minutes
6. **Repeats the cycle** - Automatic work/break rotation
7. **Shows stats on exit** - Press Ctrl+C to see session summary

## How It Works

1. **Window Monitoring**: Uses Hyprland IPC (`hyprctl`) to query the active window every second
2. **Domain Parsing**: Extracts domains from browser window titles using regex patterns
3. **Time Tracking**: Accumulates seconds spent on each domain
4. **Pomodoro Logic**: Tracks session duration and notifies at 25-minute intervals
5. **Statistics**: Displays sorted list of domains by time spent

## Configuration

You can modify these constants in `src/main.rs`:

```rust
const POLL_INTERVAL_MS: u64 = 1000;        // Window check interval
const POMODORO_WORK_MINUTES: i64 = 25;     // Work period
const POMODORO_BREAK_MINUTES: i64 = 5;     // Break period
```

## Output Example

```
🍅 Stop It - Browser Activity Monitor & Pomodoro Timer
======================================================
Pomodoro settings: 25min work / 5min break
Running on Hyprland (Wayland)
Logging to: /home/user/.local/share/stop_it/activity.log
Monitoring active window... Press Ctrl+C to stop and see stats

💼 Starting in WORK mode

[14:23:45] Switched to: github.com
[14:25:12] Switched to: stackoverflow.com
[14:30:08] Switched to: docs.rs
[14:48:00] 🔔 Work session complete! Time for a 5-minute break.

--- Session Statistics ---
Session duration: 25 minutes

Time spent per domain:
  github.com - 15m 23s
  stackoverflow.com - 8m 12s
  docs.rs - 1m 25s
------------------------

☕ [14:48:00] Switched to BREAK mode

[14:53:00] 🔔 Break is over! Starting 25-minute work session.

💼 [14:53:00] Switched to WORK mode
```

## Log File

Activity is logged to `~/.local/share/stop_it/activity.log` by default:

```
=== Session started at 2025-11-23 14:23:00 ===
[14:23:45] Switched to: github.com
[14:25:12] Switched to: stackoverflow.com
🔔 Work session complete! Time for a 5-minute break.
[14:48:00] Switched to BREAK mode
🔔 Break is over! Starting 25-minute work session.
[14:53:00] Switched to WORK mode
```

## Limitations

- **Hyprland Only**: Currently only works with Hyprland compositor (not other Wayland compositors or X11)
- **Title-based Detection**: Relies on browser window titles containing domain names
- **No HTTPS Inspection**: Cannot decrypt HTTPS traffic (this is intentional for privacy)

## Future Enhancements

- Configuration file support
- Activity logging to file
- Website blocking after thresholds
- Support for other Wayland compositors (Sway, GNOME, KDE)
- Support for X11
- Custom domain categorization (work vs. distraction)
- Web dashboard for analytics

## Privacy

This tool runs entirely locally and does NOT:
- Send any data to external servers
- Store browsing history permanently
- Decrypt HTTPS traffic
- Track anything beyond active window titles

## License

MIT
