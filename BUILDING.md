# Building Signal You Messenger

This document explains how to build Signal You Messenger as a standalone desktop application.

## Prerequisites

### All Platforms
- Node.js 18 or higher
- npm 9 or higher

### Linux (Debian/Ubuntu)
```bash
# Install build dependencies
sudo apt update
sudo apt install -y build-essential dpkg-dev libgtk-3-dev libnotify-dev libnss3-dev libxss-dev libxtst-dev

# For icon generation (optional)
sudo apt install -y inkscape  # or imagemagick

# For Flatpak builds
sudo apt install -y flatpak flatpak-builder
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
```

### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# For icon generation
brew install librsvg
```

### Windows
- Visual Studio Build Tools 2019 or later
- Windows SDK

## Installation

```bash
# Clone the repository
git clone https://github.com/phlthy88/Signal-You-Messenger.git
cd Signal-You-Messenger

# Install dependencies
npm install
```

## Development

```bash
# Run in development mode (web)
npm run dev

# Run with Electron in development
npm run dev:electron

# Run backend server separately
npm run server
```

## Building

### Generate Icons (First Time)

Before building, generate the application icons:

```bash
./scripts/generate-icons.sh
```

This creates PNG icons in various sizes from the SVG source.

### Build for Current Platform

```bash
# Build for current platform
npm run electron:build

# Build only the frontend
npm run build
```

### Build for Linux

```bash
# Build all Linux formats (deb, AppImage, rpm, flatpak)
npm run electron:build:linux

# Build specific format
npm run electron:build:deb
npm run electron:build:appimage
npm run electron:build:flatpak
npm run electron:build:rpm
```

### Build for macOS

```bash
npm run electron:build:mac
```

### Build for Windows

```bash
npm run electron:build:win
```

### Build for All Platforms

```bash
npm run electron:build:all
```

## Manual Package Building

### Debian Package (Alternative Method)

For more control over the Debian package:

```bash
./scripts/build-deb.sh
```

The package will be in `release/signal-you-messenger_1.0.0_amd64.deb`

### Flatpak Package (Alternative Method)

For more control over the Flatpak build:

```bash
./scripts/build-flatpak.sh
```

The bundle will be in `release/signal-you-messenger.flatpak`

## Installation

### Debian/Ubuntu

```bash
# Install the package
sudo dpkg -i release/signal-you-messenger_1.0.0_amd64.deb

# Fix any dependency issues
sudo apt-get install -f
```

### Flatpak

```bash
# Install from bundle
flatpak install --user release/signal-you-messenger.flatpak

# Run
flatpak run com.signalyou.Messenger
```

### AppImage

```bash
chmod +x release/Signal-You-Messenger-*.AppImage
./release/Signal-You-Messenger-*.AppImage
```

## Directory Structure

```
Signal-You-Messenger/
├── build/                  # Build resources
│   ├── icons/             # Application icons
│   └── entitlements.mac.plist  # macOS entitlements
├── debian/                 # Debian packaging files
├── electron/              # Electron main process
│   ├── main.js           # Main entry point
│   └── preload.js        # Preload script
├── flatpak/               # Flatpak packaging files
├── scripts/               # Build scripts
├── server/                # Backend server
├── src/                   # Frontend source
├── electron-builder.json  # Electron builder config
├── package.json           # Project config
└── vite.config.ts        # Vite config
```

## Troubleshooting

### Build Fails with Native Module Errors

```bash
# Rebuild native modules for Electron
npm run postinstall
npx electron-rebuild
```

### Icons Not Found

Run the icon generation script:
```bash
./scripts/generate-icons.sh
```

If no image conversion tools are available, install one:
```bash
# Ubuntu/Debian
sudo apt install inkscape

# macOS
brew install inkscape
```

### Flatpak Build Fails

Ensure you have the required runtimes:
```bash
flatpak install flathub org.freedesktop.Platform//23.08
flatpak install flathub org.freedesktop.Sdk//23.08
flatpak install flathub org.electronjs.Electron2.BaseApp//23.08
```

## Environment Variables

The following environment variables can be configured:

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Backend server port | `3001` |
| `JWT_SECRET` | JWT signing secret | Required |
| `GEMINI_API_KEY` | Google Gemini API key | Optional |
| `CORS_ORIGIN` | CORS allowed origin | `http://localhost:3000` |
| `MAX_FILE_SIZE` | Max upload file size | `10485760` (10MB) |

## License

MIT License - see LICENSE file for details.
