#!/bin/bash

# Signal You Messenger launcher script for Flatpak

# Set up environment
export ELECTRON_IS_DEV=0
export NODE_ENV=production

# Flatpak-specific paths
export XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
export XDG_CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}"

# Application data directory
APP_DATA_DIR="$XDG_DATA_HOME/signal-you-messenger"
mkdir -p "$APP_DATA_DIR/data" "$APP_DATA_DIR/uploads"

# Export paths for the application
export DATA_DIR="$APP_DATA_DIR/data"
export UPLOAD_DIR="$APP_DATA_DIR/uploads"

# Launch the application
cd /app/lib/signal-you-messenger
exec ./electron electron/main.js "$@"
