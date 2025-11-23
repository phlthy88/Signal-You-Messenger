#!/bin/bash
# Post-installation script for Signal You Messenger

set -e

# Update desktop database
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications || true
fi

# Update icon cache
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi

# Create symbolic link in PATH if needed
if [ ! -L /usr/local/bin/signal-you-messenger ]; then
    ln -sf /usr/bin/signal-you-messenger /usr/local/bin/signal-you-messenger 2>/dev/null || true
fi

echo "Signal You Messenger has been installed successfully!"
echo "You can launch it from your application menu or by running 'signal-you-messenger'"

exit 0
