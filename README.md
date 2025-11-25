# Stop It - Browser Activity Monitor & Pomodoro Timer

A Rust-based CLI tool that monitors your browser activity and helps you stay focused using the Pomodoro technique.

## Features

- **Active Window Monitoring**: Tracks which browser tab/window is currently active
- **Browser Extension**: Accurate URL tracking via native messaging (optional)
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

## Browser Extension

> 📘 **Documentation:** [Read the full Browser Extension Guide](./browser-extension/BrowserExtension.md)

## Native Host

> 📘 **Documentation:** [Read the full Native Host Guide](./src/NativeHost.md)

## Test It

1. After following both doc, load the extension and run the `NativeHost`
2. Open your browser and navigate to any website (e.g., github.com)
3. Check the native messaging log:

   ```bash
   tail -f ~/.local/share/stop_it/native_messaging.log
   ```

4. Check the browser activity log:

   ```bash
   tail -f ~/.local/share/stop_it/browser_activity.log
   ```

You should see entries like:

```log
[14:23:45] Received: url=https://github.com/..., title=..., domain=Some("github.com")
```

## How It Works

### Architecture

```
┌─────────────────┐      Native        ┌──────────────────┐
│     Browser     │     Messaging      │   Rust Binary    │
│   Extension     │◄──────────────────►│  (stop_it)       │
│  (JavaScript)   │   JSON via stdin   │  --native-       │
│                 │   /stdout          │   messaging      │
└─────────────────┘                    └──────────────────┘
```

### Message Flow

1. Extension monitors active tab changes
2. When URL/title changes, extension extracts:
   - Full URL
   - Page title
   - Domain (hostname without www)
3. Sends JSON message to native host via Chrome's native messaging API
4. Rust app receives message on stdin, logs to files
5. Rust app sends acknowledgment response

## Troubleshooting

### Extension shows errors in console

1. Right-click extension icon → Inspect
2. Check Console tab for errors
3. Common error: `"Specified native messaging host not found"`
   - Solution: Run `./install_native_messaging.sh`

### Native host not receiving messages

1. Check manifest files exist:

   ```bash
   ls ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/
   ```

2. Check manifest has correct extension ID:

   ```bash
   cat ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.stopit.tracker.json
   ```

   Should contain `chrome-extension://YOUR_ACTUAL_ID/`, not `EXTENSION_ID_PLACEHOLDER`

3. Check binary path in manifest is correct:

   ```bash
   # The path should point to your actual binary
   jq .path ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.stopit.tracker.json
   ```

### No logs appearing

1. Check log directory exists:

   ```bash
   ls -la ~/.local/share/stop_it/
   ```

2. Test with manual invocation:

   ```bash
   echo '{"type":"tab_update","url":"https://test.com","title":"Test","domain":"test.com","timestamp":123}' | \
     ./target/release/stop_it --native-messaging
   ```

3. Check file permissions on binary:

   ```bash
   ls -l ./target/release/stop_it
   ```

## Privacy

This tool runs entirely locally and does NOT:

- Send any data to external servers
- Store browsing history permanently
- Decrypt HTTPS traffic
- Track anything beyond active window titles

## License

MIT
