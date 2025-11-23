#!/bin/bash
#
# Generate PNG icons from SVG source
# Requires: inkscape or imagemagick (convert)
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ICONS_DIR="$PROJECT_DIR/build/icons"
SVG_SOURCE="$ICONS_DIR/icon.svg"

echo "=== Generating Icons ==="
echo "Source: $SVG_SOURCE"
echo "Output: $ICONS_DIR"
echo ""

# Check for source SVG
if [ ! -f "$SVG_SOURCE" ]; then
    echo "Error: Source SVG not found at $SVG_SOURCE"
    exit 1
fi

# Ensure output directory exists
mkdir -p "$ICONS_DIR"

# Function to generate PNG using available tool
generate_png() {
    local size=$1
    local output="$ICONS_DIR/icon-${size}.png"

    if command -v inkscape >/dev/null 2>&1; then
        inkscape "$SVG_SOURCE" --export-filename="$output" -w "$size" -h "$size" 2>/dev/null
        echo "Generated: icon-${size}.png (using inkscape)"
    elif command -v convert >/dev/null 2>&1; then
        convert -background none "$SVG_SOURCE" -resize "${size}x${size}" "$output" 2>/dev/null
        echo "Generated: icon-${size}.png (using imagemagick)"
    elif command -v rsvg-convert >/dev/null 2>&1; then
        rsvg-convert -w "$size" -h "$size" "$SVG_SOURCE" -o "$output" 2>/dev/null
        echo "Generated: icon-${size}.png (using rsvg-convert)"
    else
        echo "Warning: No image conversion tool found. Install inkscape, imagemagick, or librsvg."
        return 1
    fi
}

# Generate all required sizes
sizes=(16 24 32 48 64 96 128 192 256 512 1024)

for size in "${sizes[@]}"; do
    generate_png "$size" || true
done

# Create symlinks for electron-builder expected names
echo ""
echo "Creating symlinks for electron-builder..."
cd "$ICONS_DIR"

# Linux icon (electron-builder expects icon.png)
if [ -f "icon-512.png" ]; then
    ln -sf icon-512.png icon.png
    echo "Created: icon.png -> icon-512.png"
fi

# Tray icon (smaller)
if [ -f "icon-32.png" ]; then
    ln -sf icon-32.png tray-icon.png
    echo "Created: tray-icon.png -> icon-32.png"
fi

# Generate ICO file for Windows (if icotool is available)
if command -v icotool >/dev/null 2>&1; then
    echo ""
    echo "Generating Windows ICO file..."
    icotool -c -o "$ICONS_DIR/icon.ico" \
        "$ICONS_DIR/icon-16.png" \
        "$ICONS_DIR/icon-24.png" \
        "$ICONS_DIR/icon-32.png" \
        "$ICONS_DIR/icon-48.png" \
        "$ICONS_DIR/icon-64.png" \
        "$ICONS_DIR/icon-128.png" \
        "$ICONS_DIR/icon-256.png" 2>/dev/null || echo "Warning: ICO generation failed"
fi

# Generate ICNS file for macOS (if png2icns or iconutil is available)
if command -v png2icns >/dev/null 2>&1; then
    echo ""
    echo "Generating macOS ICNS file..."
    png2icns "$ICONS_DIR/icon.icns" \
        "$ICONS_DIR/icon-16.png" \
        "$ICONS_DIR/icon-32.png" \
        "$ICONS_DIR/icon-128.png" \
        "$ICONS_DIR/icon-256.png" \
        "$ICONS_DIR/icon-512.png" 2>/dev/null || echo "Warning: ICNS generation failed"
elif command -v iconutil >/dev/null 2>&1; then
    echo ""
    echo "Generating macOS ICNS file using iconutil..."
    ICONSET_DIR="$ICONS_DIR/icon.iconset"
    mkdir -p "$ICONSET_DIR"
    cp "$ICONS_DIR/icon-16.png" "$ICONSET_DIR/icon_16x16.png" 2>/dev/null || true
    cp "$ICONS_DIR/icon-32.png" "$ICONSET_DIR/icon_16x16@2x.png" 2>/dev/null || true
    cp "$ICONS_DIR/icon-32.png" "$ICONSET_DIR/icon_32x32.png" 2>/dev/null || true
    cp "$ICONS_DIR/icon-64.png" "$ICONSET_DIR/icon_32x32@2x.png" 2>/dev/null || true
    cp "$ICONS_DIR/icon-128.png" "$ICONSET_DIR/icon_128x128.png" 2>/dev/null || true
    cp "$ICONS_DIR/icon-256.png" "$ICONSET_DIR/icon_128x128@2x.png" 2>/dev/null || true
    cp "$ICONS_DIR/icon-256.png" "$ICONSET_DIR/icon_256x256.png" 2>/dev/null || true
    cp "$ICONS_DIR/icon-512.png" "$ICONSET_DIR/icon_256x256@2x.png" 2>/dev/null || true
    cp "$ICONS_DIR/icon-512.png" "$ICONSET_DIR/icon_512x512.png" 2>/dev/null || true
    cp "$ICONS_DIR/icon-1024.png" "$ICONSET_DIR/icon_512x512@2x.png" 2>/dev/null || true
    iconutil -c icns "$ICONSET_DIR" -o "$ICONS_DIR/icon.icns" 2>/dev/null || echo "Warning: ICNS generation failed"
    rm -rf "$ICONSET_DIR"
fi

echo ""
echo "=== Icon Generation Complete ==="
ls -la "$ICONS_DIR"
