#!/bin/bash

# Installation script for Stop It daemon
# This sets up the daemon to run automatically on system startup

set -e

echo "🍅 Stop It - Daemon Installation"
echo "================================="

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

# Create systemd user service directory
SYSTEMD_DIR="$HOME/.config/systemd/user"
mkdir -p "$SYSTEMD_DIR"

# Copy service file and update the binary path
SERVICE_FILE="$SYSTEMD_DIR/stop-it.service"
cat > "$SERVICE_FILE" << EOF
[Unit]
Description=Stop It - Activity Tracker & Pomodoro Timer Daemon
After=network.target

[Service]
Type=simple
ExecStart=$BINARY_PATH --daemon
Restart=on-failure
RestartSec=5
Environment="DISPLAY=:0"
Environment="WAYLAND_DISPLAY=wayland-1"

# Logging
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
EOF

echo "✅ Service file created at: $SERVICE_FILE"

# Reload systemd user daemon
echo "🔄 Reloading systemd user daemon..."
systemctl --user daemon-reload

# Enable the service
echo "🔧 Enabling service to start on boot..."
systemctl --user enable stop-it.service

# Start the service
echo "▶️  Starting the daemon..."
systemctl --user start stop-it.service

# Check status
echo ""
echo "📊 Service status:"
systemctl --user status stop-it.service --no-pager

echo ""
echo "✅ Installation complete!"
echo ""
echo "Useful commands:"
echo "  systemctl --user status stop-it    # Check daemon status"
echo "  systemctl --user stop stop-it      # Stop the daemon"
echo "  systemctl --user start stop-it     # Start the daemon"
echo "  systemctl --user restart stop-it   # Restart the daemon"
echo "  journalctl --user -u stop-it -f    # View daemon logs (follow mode)"
echo ""
echo "Next steps:"
echo "1. Build the browser extension:"
echo "   cd browser-extension && npm install && npm run build"
echo "2. Load the extension in your browser:"
echo "   - Open brave://extensions (enable Developer mode)"
echo "   - Click 'Load unpacked'"
echo "   - Select the 'browser-extension/dist' folder"
echo "3. The extension will automatically connect to the daemon at ws://127.0.0.1:8765"
