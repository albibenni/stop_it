#!/bin/bash

# Installation script for Stop It native messaging host
# This allows the browser extension to communicate with the Rust application

set -e

echo "🍅 Stop It - Native Messaging Installation"
echo "=========================================="

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust first."
    exit 1
fi

# Build the Rust application
echo "📦 Building Rust application..."
cargo build --release

# Get the absolute path to the binary
BINARY_PATH="$(pwd)/target/release/stop_it"

if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Error: Binary not found at $BINARY_PATH"
    exit 1
fi

echo "✅ Binary built at: $BINARY_PATH"

# Create the native messaging manifest
MANIFEST_NAME="com.stopit.tracker.json"
MANIFEST_CONTENT="{
  \"name\": \"com.stopit.tracker\",
  \"description\": \"Stop It Activity Tracker Native Messaging Host\",
  \"path\": \"$BINARY_PATH\",
  \"type\": \"stdio\",
  \"allowed_origins\": [
    \"chrome-extension://EXTENSION_ID_PLACEHOLDER/\"
  ]
}"

# Determine browser-specific manifest directories
CHROMIUM_DIR="$HOME/.config/chromium/NativeMessagingHosts"
CHROME_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"
BRAVE_DIR="$HOME/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts"
EDGE_DIR="$HOME/.config/microsoft-edge/NativeMessagingHosts"

# Function to install manifest
install_manifest() {
    local dir=$1
    local browser_name=$2

    if [ -d "$(dirname "$dir")" ] || [ "$browser_name" == "Brave" ]; then
        mkdir -p "$dir"
        echo "$MANIFEST_CONTENT" > "$dir/$MANIFEST_NAME"
        echo "✅ Installed for $browser_name: $dir/$MANIFEST_NAME"
        return 0
    else
        echo "⏭️  Skipping $browser_name (not installed)"
        return 1
    fi
}

# Install for detected browsers
echo ""
echo "📝 Installing native messaging manifests..."
INSTALLED=false

if install_manifest "$BRAVE_DIR" "Brave"; then
    INSTALLED=true
fi

if install_manifest "$CHROMIUM_DIR" "Chromium"; then
    INSTALLED=true
fi

if install_manifest "$CHROME_DIR" "Chrome"; then
    INSTALLED=true
fi

if install_manifest "$EDGE_DIR" "Edge"; then
    INSTALLED=true
fi

if [ "$INSTALLED" = false ]; then
    echo "❌ No supported browsers found!"
    exit 1
fi

echo ""
echo "⚠️  IMPORTANT: After loading the extension, you need to:"
echo "   1. Find your extension ID in brave://extensions (or chrome://extensions)"
echo "   2. Update the manifest files above, replacing EXTENSION_ID_PLACEHOLDER with your actual extension ID"
echo "   3. Or run: ./update_extension_id.sh <YOUR_EXTENSION_ID>"
echo ""
echo "✅ Installation complete!"
echo ""
echo "Next steps:"
echo "1. Build the browser extension:"
echo "   cd browser-extension && npm install && npm run build"
echo "2. Load the extension in your browser:"
echo "   - Open brave://extensions (enable Developer mode)"
echo "   - Click 'Load unpacked'"
echo "   - Select the 'browser-extension/dist' folder"
echo "3. Get the extension ID and update the manifests"
echo "4. Test by browsing - check logs at ~/.local/share/stop_it/native_messaging.log"
