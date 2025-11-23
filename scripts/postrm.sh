#!/bin/bash
# Post-removal script for Signal You Messenger

set -e

# Update desktop database
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications || true
fi

# Update icon cache
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi

# Remove symbolic link
rm -f /usr/local/bin/signal-you-messenger 2>/dev/null || true

# Note: User data in ~/.local/share/signal-you-messenger is preserved
# Users can manually remove it if desired

exit 0
