# Stop It - Browser Extension

TypeScript browser extension that sends URL and domain information to the *Stop It activity tracker*.

## Building

```bash
npm install
npm run build
```

The built extension will be in the `dist/` folder.

## Loading in Browser

### Brave/Chrome/Chromium

1. Open `brave://extensions` (or `chrome://extensions`)
2. Enable **Developer mode** (toggle in top right)  - `manage extension`
3. Click **Load unpacked**
4. Select the `browser-extension/dist` folder
5. Note the **Extension ID** (you'll need this for native messaging setup)

### Getting the Extension ID

After loading the extension:

1. From the `manage extension` look for the extension
2. Click on `details`
3. Find the `ID` field (looks like: `abcdefghijklmnopqrstuvwxyz123456`)
4. Copy this ID - you'll need it for the native messaging setup

## Next Steps

After building and loading the extension, go back to the main project and:

1. Run `./install_native_messaging.sh`
2. Run `./update_extension_id.sh <YOUR_EXTENSION_ID>`
3. Test by browsing websites

## Development

Watch mode (automatically rebuild on changes):

```bash
npm run watch
```

Note: You'll need to click the reload button in `brave://extensions` after each rebuild.

## How It Works

- The extension monitors the active tab in your browser
- When you switch tabs or navigate to new URLs, it extracts:
  - Full URL
  - Page title
  - Domain name
- This data is sent to the native messaging host (Stop It Rust app)
- The Rust app logs this information for accurate activity tracking

## Troubleshooting

If the extension isn't working:

1. Check extension is loaded: `brave://extensions`
2. Check native messaging manifest is installed: `~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.stopit.tracker.json`
3. Check the manifest has correct extension ID (not EXTENSION_ID_PLACEHOLDER)
4. Check native messaging logs: `~/.local/share/stop_it/native_messaging.log`
5. Check browser extension console: Right-click extension → Inspect → Console tab
