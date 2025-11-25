#!/bin/bash

# Script to update extension ID in native messaging manifests

if [ -z "$1" ]; then
    echo "Usage: $0 <EXTENSION_ID>"
    echo ""
    echo "Example: $0 abcdefghijklmnopqrstuvwxyz123456"
    echo ""
    echo "To find your extension ID:"
    echo "1. Open brave://extensions (or chrome://extensions)"
    echo "2. Enable 'Developer mode'"
    echo "3. Find the 'ID' field under your extension"
    exit 1
fi

EXTENSION_ID="$1"
MANIFEST_NAME="com.stopit.tracker.json"

# Directories where manifests are installed
DIRS=(
    "$HOME/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts"
    "$HOME/.config/chromium/NativeMessagingHosts"
    "$HOME/.config/google-chrome/NativeMessagingHosts"
    "$HOME/.config/microsoft-edge/NativeMessagingHosts"
)

echo "🔧 Updating extension ID to: $EXTENSION_ID"
echo ""

UPDATED=false

for DIR in "${DIRS[@]}"; do
    MANIFEST_PATH="$DIR/$MANIFEST_NAME"
    if [ -f "$MANIFEST_PATH" ]; then
        # Update the extension ID in the manifest
        sed -i "s|chrome-extension://EXTENSION_ID_PLACEHOLDER/|chrome-extension://$EXTENSION_ID/|g" "$MANIFEST_PATH"
        echo "✅ Updated: $MANIFEST_PATH"
        UPDATED=true
    fi
done

if [ "$UPDATED" = false ]; then
    echo "❌ No manifest files found. Run ./install_native_messaging.sh first."
    exit 1
fi

echo ""
echo "✅ Extension ID updated successfully!"
echo "You can now test the extension by browsing websites."
echo "Check logs at: ~/.local/share/stop_it/native_messaging.log"
