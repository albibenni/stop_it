#!/bin/bash
# Wrapper script for native messaging that logs all arguments
# This helps debug what Chrome/Brave is actually sending

LOG_FILE="$HOME/.local/share/stop_it/wrapper.log"
BINARY="/home/albibenni/benni-projects/stop_it/target/release/stop_it"

echo "=== Wrapper called at $(date) ===" >> "$LOG_FILE"
echo "Arguments: $@" >> "$LOG_FILE"
echo "PWD: $(pwd)" >> "$LOG_FILE"
echo "USER: $USER" >> "$LOG_FILE"

# Call the actual binary with --native-messaging flag
exec "$BINARY" --native-messaging
