#!/bin/bash

# Signal You Messenger launcher script for Debian package

# Set up environment
export ELECTRON_IS_DEV=0
export NODE_ENV=production

# Application data directory
APP_DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/signal-you-messenger"
mkdir -p "$APP_DATA_DIR/data" "$APP_DATA_DIR/uploads"

# Export paths for the application
export DATA_DIR="$APP_DATA_DIR/data"
export UPLOAD_DIR="$APP_DATA_DIR/uploads"

# Find the installed Electron binary
ELECTRON_PATH="/opt/signal-you-messenger/node_modules/electron/dist/electron"

if [ ! -x "$ELECTRON_PATH" ]; then
    echo "Error: Electron binary not found at $ELECTRON_PATH"
    exit 1
fi

# Launch the application
cd /opt/signal-you-messenger
exec "$ELECTRON_PATH" electron/main.js "$@"
