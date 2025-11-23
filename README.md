# Stop It - Browser Activity Monitor & Pomodoro Timer

A Rust-based CLI tool that monitors your browser activity and helps you stay focused using the Pomodoro technique.

## Features

- **Active Window Monitoring**: Tracks which browser tab/window is currently active
- **Domain Extraction**: Intelligently extracts domain names from window titles
- **Time Tracking**: Records time spent on each domain during your session
- **Pomodoro Timer**: Notifies you after 25 minutes of work to take a 5-minute break
- **Desktop Notifications**: Sends native notifications when it's time for a break
- **Session Statistics**: View detailed stats on Ctrl+C

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

Run the monitor:

```bash
cargo run
# or after building:
./target/release/stop_it
```

The app will:
1. Start monitoring your active window
2. Log domain switches in real-time
3. Track time spent on each domain
4. Send notifications after 25 minutes of work
5. Display session statistics when you press Ctrl+C

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
Monitoring active window... Press Ctrl+C to stop and see stats

[14:23:45] Switched to: github.com
[14:25:12] Switched to: stackoverflow.com
[14:48:00] 🔔 You've been working for 25 minutes! Time for a 5-minute break.

--- Session Statistics ---
Session duration: 25 minutes

Time spent per domain:
  github.com - 15m 23s
  stackoverflow.com - 8m 12s
  docs.rs - 1m 25s
------------------------
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
