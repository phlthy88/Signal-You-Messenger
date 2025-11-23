#!/bin/bash
#
# Build script for Signal You Messenger Flatpak package
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/flatpak-build"
REPO_DIR="$PROJECT_DIR/flatpak-repo"

echo "=== Signal You Messenger Flatpak Build ==="
echo "Project directory: $PROJECT_DIR"
echo ""

# Check for required tools
command -v flatpak-builder >/dev/null 2>&1 || {
    echo "Error: flatpak-builder is not installed."
    echo "Install it with: sudo apt install flatpak-builder"
    exit 1
}

# Check for required Flatpak runtimes
if ! flatpak info org.freedesktop.Platform//23.08 >/dev/null 2>&1; then
    echo "Installing Freedesktop Platform runtime..."
    flatpak install -y flathub org.freedesktop.Platform//23.08
fi

if ! flatpak info org.freedesktop.Sdk//23.08 >/dev/null 2>&1; then
    echo "Installing Freedesktop SDK..."
    flatpak install -y flathub org.freedesktop.Sdk//23.08
fi

if ! flatpak info org.electronjs.Electron2.BaseApp//23.08 >/dev/null 2>&1; then
    echo "Installing Electron BaseApp..."
    flatpak install -y flathub org.electronjs.Electron2.BaseApp//23.08
fi

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf "$BUILD_DIR" "$REPO_DIR"

# Generate npm sources for offline build (optional, requires flatpak-node-generator)
if command -v flatpak-node-generator >/dev/null 2>&1; then
    echo "Generating npm sources for offline build..."
    cd "$PROJECT_DIR"
    flatpak-node-generator npm package-lock.json -o flatpak/flatpak-npm-sources.json || true
    flatpak-node-generator npm server/package-lock.json -o flatpak/flatpak-server-npm-sources.json || true
fi

# Build the Flatpak
echo "Building Flatpak..."
cd "$PROJECT_DIR"

flatpak-builder \
    --force-clean \
    --install-deps-from=flathub \
    --ccache \
    --repo="$REPO_DIR" \
    "$BUILD_DIR" \
    flatpak/com.signalyou.Messenger.yml

# Create a Flatpak bundle
echo "Creating Flatpak bundle..."
flatpak build-bundle \
    "$REPO_DIR" \
    "$PROJECT_DIR/release/signal-you-messenger.flatpak" \
    com.signalyou.Messenger

echo ""
echo "=== Build Complete ==="
echo "Flatpak bundle: $PROJECT_DIR/release/signal-you-messenger.flatpak"
echo ""
echo "To install locally:"
echo "  flatpak install --user signal-you-messenger.flatpak"
echo ""
echo "To run:"
echo "  flatpak run com.signalyou.Messenger"
