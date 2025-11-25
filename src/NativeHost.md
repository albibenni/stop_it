# Stop it - Native Host

## Installation and Usage

From the project root:

```bash
# Build the Rust app and install manifests
./install_native_messaging.sh

# Update manifests with your extension ID
./update_extension_id.sh <YOUR_EXTENSION_ID>
```

### What the script do `./install_native_messaging.sh`

1. Builds your Rust app (cargo build --release)
2. Creates a JSON manifest with:
    - Name: com.stopit.tracker
    - Path to your compiled binary: /path/to/stop_it/target/release/stop_it
    - Allowed extension IDs (initially a placeholder)
    - Communication type: stdio (standard input/output)
3. Installs the manifest in browser-specific directories:
    - Brave: ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/
    - Chrome: ~/.config/google-chrome/NativeMessagingHosts/
    - Chromium: ~/.config/chromium/NativeMessagingHosts/
    - Edge: ~/.config/microsoft-edge/NativeMessagingHosts/

#### Why is needed

Without this manifest file in the correct location, the browser won't know:

- That your native messaging host exists
- Where to find the executable
- Whether the extension is authorized to use it

### What the script do `./update_extension_id.sh`

Updates the manifest files with your actual extension ID after you load the extension.

- Takes your extension ID as a parameter
- Finds all installed manifest files
- Replaces `EXTENSION_ID_PLACEHOLDER` with your actual extension ID using `sed`

You can't know your extension ID until after you've loaded the unpacked extension in your browser. Browser extensions get assigned a unique ID, and this ID must be in the allowed_origins field of the manifest.

### Workflow Summary

1. Run `install_native_messaging.sh` → Creates manifest with placeholder
2. Load extension in browser → Get assigned an extension ID
3. Run `update_extension_id.sh <ID>` → Updates manifest with real ID
4. Extension can now communicate with Rust app via native messaging

Without these scripts, you'd have to manually create and place the manifest files in the correct
browser directories and manage the extension ID yourself - these scripts automate that process.
