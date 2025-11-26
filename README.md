# Stop It - Browser Activity Monitor & Pomodoro Timer

A Rust-based daemon that monitors your browser activity and helps you stay focused using the Pomodoro technique.

## Features

- **Daemon Mode**: Runs continuously in the background as a systemd service
- **WebSocket Server**: Real-time communication with browser extension (ws://127.0.0.1:8765)
- **Dual Tracking**: Monitors both browser extension messages AND Hyprland window titles
- **Universal Domain Extraction**: Extracts domains from ANY website (github.com, google.com, docs.rs, etc.)
- **Time Tracking**: Records time spent on each domain during your session
- **Pomodoro Timer**: Automatically switches between work (25min) and break (5min) modes
- **File Logging**: Logs all activity to `~/.local/share/stop_it/daemon.log`
- **Desktop Notifications**: Sends native notifications when it's time to switch modes
- **Session Statistics**: View detailed stats after each work session
- **Auto-Reconnect**: Extension automatically reconnects if daemon restarts

## Requirements

- Linux with Hyprland (Wayland compositor)
- `hyprctl` command available (included with Hyprland)
- Rust 1.70 or later
- systemd (for daemon autostart)

## Quick Start

### 1. Install the Daemon

```bash
./install_daemon.sh
```

This will:

- Build the Rust application
- Install and enable the systemd service
- Start the daemon automatically

### 2. Install the Browser Extension

```bash
cd browser-extension
npm install
npm run build
```

Then load the extension in your browser:

- Open `brave://extensions` (or `chrome://extensions`)
- Enable "Developer mode"
- Click "Load unpacked"
- Select the `browser-extension/dist` folder

### 3. Verify It's Working

Check the daemon logs:

```bash
journalctl --user -u stop-it -f
```

You should see:

```
🍅 Stop It - Daemon Mode
Running WebSocket server on ws://127.0.0.1:8765
New WebSocket connection from: 127.0.0.1:xxxxx
[16:08:38] Browser switched to: github.com
```

## Useful Commands

```bash
# Check daemon status
systemctl --user status stop-it

# Stop the daemon
systemctl --user stop stop-it

# Start the daemon
systemctl --user start stop-it

# Restart the daemon
systemctl --user restart stop-it

# View daemon logs (follow mode)
journalctl --user -u stop-it -f
```

## How It Works

### Architecture

```
┌─────────────────┐                    ┌──────────────────────────┐
│     Browser     │     WebSocket      │   Rust Daemon            │
│   Extension     │◄──────────────────►│  (always running)        │
│  (JavaScript)   │  ws://localhost    │                          │
│                 │     :8765          │  ├─ WebSocket Server     │
└─────────────────┘                    │  ├─ Hyprland Monitor     │
                                       │  ├─ Pomodoro Timer       │
                                       │  └─ Activity Tracker     │
                                       └──────────────────────────┘
```

### Message Flow

1. **Daemon starts**: Launches WebSocket server on `ws://127.0.0.1:8765`
2. **Extension connects**: Establishes persistent WebSocket connection
3. **Continuous monitoring**:
   - Extension sends URL/title updates when tabs change
   - Hyprland poller detects window title changes every second
   - Both sources feed into the same activity tracker
4. **Pomodoro timer**: Runs every second, tracks work/break cycles
5. **Real-time updates**: All activity logged and tracked continuously
6. **Auto-reconnect**: If daemon restarts, extension reconnects automatically

## Troubleshooting

**Daemon not starting:**

```bash
# Check daemon status
systemctl --user status stop-it

# Check logs for errors
journalctl --user -u stop-it -n 50
```

**Extension can't connect:**

1. Verify daemon is running: `systemctl --user status stop-it`
2. Check if WebSocket port is listening: `ss -tlnp | grep 8765`
3. Check extension console (Right-click extension → Inspect)
   - Should see: "Connected to Stop It daemon"
   - If seeing: "WebSocket not connected" - daemon may be down

**No activity being tracked:**

1. Check daemon logs: `journalctl --user -u stop-it -f`
2. Open a website and see if messages appear
3. Verify both extension and Hyprland monitoring are working

## Browser Extension Documentation

> 📘 **Documentation:** For detailed browser extension setup, see [Browser Extension Guide](./browser-extension/BrowserExtension.md)

## Privacy

This tool runs entirely locally and does NOT:

- Send any data to external servers
- Store browsing history permanently
- Decrypt HTTPS traffic
- Track anything beyond active window titles

## License

MIT
