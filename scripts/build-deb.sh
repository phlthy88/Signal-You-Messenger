#!/bin/bash
#
# Build script for Signal You Messenger Debian package
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/deb-build"
VERSION="1.0.0"
PACKAGE_NAME="signal-you-messenger"
ARCH="amd64"

echo "=== Signal You Messenger Debian Package Build ==="
echo "Project directory: $PROJECT_DIR"
echo "Version: $VERSION"
echo ""

# Check for required tools
command -v dpkg-deb >/dev/null 2>&1 || {
    echo "Error: dpkg-deb is not installed."
    echo "Install it with: sudo apt install dpkg-dev"
    exit 1
}

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf "$BUILD_DIR"

# Create directory structure
echo "Creating package structure..."
PACKAGE_DIR="$BUILD_DIR/${PACKAGE_NAME}_${VERSION}_${ARCH}"
mkdir -p "$PACKAGE_DIR/DEBIAN"
mkdir -p "$PACKAGE_DIR/opt/signal-you-messenger"
mkdir -p "$PACKAGE_DIR/usr/bin"
mkdir -p "$PACKAGE_DIR/usr/share/applications"
mkdir -p "$PACKAGE_DIR/usr/share/icons/hicolor/16x16/apps"
mkdir -p "$PACKAGE_DIR/usr/share/icons/hicolor/32x32/apps"
mkdir -p "$PACKAGE_DIR/usr/share/icons/hicolor/48x48/apps"
mkdir -p "$PACKAGE_DIR/usr/share/icons/hicolor/64x64/apps"
mkdir -p "$PACKAGE_DIR/usr/share/icons/hicolor/128x128/apps"
mkdir -p "$PACKAGE_DIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$PACKAGE_DIR/usr/share/icons/hicolor/512x512/apps"
mkdir -p "$PACKAGE_DIR/usr/share/icons/hicolor/scalable/apps"
mkdir -p "$PACKAGE_DIR/usr/share/doc/${PACKAGE_NAME}"

# Build the application
echo "Building application..."
cd "$PROJECT_DIR"
npm ci || npm install
cd server && npm ci || npm install
cd "$PROJECT_DIR"
ELECTRON=true npm run build

# Copy application files
echo "Copying application files..."
cp -r dist "$PACKAGE_DIR/opt/signal-you-messenger/"
cp -r server "$PACKAGE_DIR/opt/signal-you-messenger/"
cp -r electron "$PACKAGE_DIR/opt/signal-you-messenger/"
cp -r node_modules "$PACKAGE_DIR/opt/signal-you-messenger/"
cp package.json "$PACKAGE_DIR/opt/signal-you-messenger/"

# Remove unnecessary files
rm -rf "$PACKAGE_DIR/opt/signal-you-messenger/server/data"
rm -rf "$PACKAGE_DIR/opt/signal-you-messenger/server/uploads"
rm -rf "$PACKAGE_DIR/opt/signal-you-messenger/server/node_modules/.cache"
rm -rf "$PACKAGE_DIR/opt/signal-you-messenger/node_modules/.cache"

# Install launcher script
echo "Installing launcher script..."
cp "$PROJECT_DIR/debian/signal-you-messenger.sh" "$PACKAGE_DIR/usr/bin/signal-you-messenger"
chmod 755 "$PACKAGE_DIR/usr/bin/signal-you-messenger"

# Install desktop file
echo "Installing desktop file..."
cp "$PROJECT_DIR/debian/signal-you-messenger.desktop" "$PACKAGE_DIR/usr/share/applications/"

# Install icons
echo "Installing icons..."
for size in 16 32 48 64 128 256 512; do
    if [ -f "$PROJECT_DIR/build/icons/icon-${size}.png" ]; then
        cp "$PROJECT_DIR/build/icons/icon-${size}.png" \
           "$PACKAGE_DIR/usr/share/icons/hicolor/${size}x${size}/apps/signal-you-messenger.png"
    fi
done
if [ -f "$PROJECT_DIR/build/icons/icon.svg" ]; then
    cp "$PROJECT_DIR/build/icons/icon.svg" \
       "$PACKAGE_DIR/usr/share/icons/hicolor/scalable/apps/signal-you-messenger.svg"
fi

# Create DEBIAN control file
echo "Creating control file..."
cat > "$PACKAGE_DIR/DEBIAN/control" << EOF
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: net
Priority: optional
Architecture: ${ARCH}
Depends: libgtk-3-0, libnotify4, libnss3, libxss1, libxtst6, libatspi2.0-0, libuuid1, libsecret-1-0, xdg-utils
Recommends: libappindicator3-1
Maintainer: Signal You Team <team@signalyou.com>
Description: Secure messaging application with AI features
 Signal You Messenger is a modern, secure messaging application that
 combines end-to-end encryption with AI-powered features for an enhanced
 communication experience.
 .
 Features include:
  * Secure messaging with real-time WebSocket communication
  * AI-powered smart replies using Google Gemini
  * Multiple themes including dark mode
  * Contact management and user discovery
  * File and image attachments
  * Cross-platform support
Homepage: https://github.com/phlthy88/Signal-You-Messenger
EOF

# Create post-installation script
echo "Creating post-installation script..."
cat > "$PACKAGE_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e
update-desktop-database -q /usr/share/applications || true
gtk-update-icon-cache -q /usr/share/icons/hicolor || true
exit 0
EOF
chmod 755 "$PACKAGE_DIR/DEBIAN/postinst"

# Create post-removal script
echo "Creating post-removal script..."
cat > "$PACKAGE_DIR/DEBIAN/postrm" << 'EOF'
#!/bin/bash
set -e
update-desktop-database -q /usr/share/applications || true
gtk-update-icon-cache -q /usr/share/icons/hicolor || true
exit 0
EOF
chmod 755 "$PACKAGE_DIR/DEBIAN/postrm"

# Create copyright file
echo "Creating copyright file..."
cp "$PROJECT_DIR/debian/copyright" "$PACKAGE_DIR/usr/share/doc/${PACKAGE_NAME}/"

# Build the package
echo "Building Debian package..."
mkdir -p "$PROJECT_DIR/release"
dpkg-deb --build --root-owner-group "$PACKAGE_DIR" "$PROJECT_DIR/release/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"

echo ""
echo "=== Build Complete ==="
echo "Debian package: $PROJECT_DIR/release/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
echo ""
echo "To install:"
echo "  sudo dpkg -i release/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
echo "  sudo apt-get install -f  # Fix any dependency issues"
echo ""
echo "To uninstall:"
echo "  sudo apt remove ${PACKAGE_NAME}"
