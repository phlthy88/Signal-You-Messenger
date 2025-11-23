<div align="center">
<img width="1200" height="475" alt="Signal You Messenger Banner" src="https://github.com/user-attachments/assets/0aa67016-6eaf-458a-adb2-6e31a0763ed6" />

# Signal You Messenger

**A GTK4/libadwaita Material You fork of Signal Messenger for Linux**

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![GTK4](https://img.shields.io/badge/GTK-4.0-green.svg)](https://gtk.org/)
[![libadwaita](https://img.shields.io/badge/libadwaita-1.4-purple.svg)](https://gnome.pages.gitlab.gnome.org/libadwaita/)
[![Signal Protocol](https://img.shields.io/badge/Signal-Protocol-blue.svg)](https://signal.org/)

[Features](#features) | [Installation](#installation) | [Building](#building) | [Contributing](#contributing)

</div>

---

## Overview

Signal You Messenger is a native GTK4/libadwaita Signal client for Linux, designed with Material You (Material Design 3) aesthetics and deep GNOME integration. This project aims to provide a modern, adaptive Signal experience that feels native on the Linux desktop while maintaining full compatibility with the Signal Protocol.

### Why Signal You?

- **Native GTK4 Experience**: Built with GTK4 and libadwaita for seamless GNOME integration
- **Material You Theming**: Adaptive color schemes that follow your system accent colors
- **Signal Protocol**: Full end-to-end encryption using the official libsignal library
- **Flatpak First**: Primary distribution through Flathub for easy installation and updates
- **Lightweight**: Native performance without Electron overhead

## Features

### Signal Protocol Integration
- **End-to-end encryption** - Full Signal Protocol implementation via libsignal
- **Sealed sender** - Enhanced metadata protection
- **Perfect forward secrecy** - Compromised keys don't affect past messages
- **Message verification** - Safety number verification for contacts

### Core Messaging
- **Private chats** - One-on-one encrypted conversations
- **Group messaging** - Secure group conversations with Signal groups
- **Media sharing** - Send photos, videos, files, and voice messages
- **Disappearing messages** - Auto-delete messages after a set time
- **Read receipts** - Optional delivery and read confirmations
- **Typing indicators** - See when contacts are typing

### GTK4/libadwaita Design
- **Adaptive layouts** - Responsive design for any screen size
- **Material You colors** - Dynamic theming based on system accent color
- **Dark/Light modes** - Automatic theme switching with system preference
- **GNOME integration** - Native notifications, app indicators, and portal support
- **Touch-friendly** - Optimized for touchscreen devices and convertibles

### Desktop Integration
- **System notifications** - Native GNOME notification support
- **Quick reply** - Reply directly from notifications
- **Do Not Disturb** - Respect GNOME's notification settings
- **Background operation** - Continue receiving messages when minimized
- **File portal** - Secure file access through XDG portals

## Technology Stack

| Component | Technology |
|-----------|------------|
| UI Framework | GTK4, libadwaita 1.4 |
| Language | Rust |
| Signal Protocol | libsignal-client |
| Database | SQLCipher (encrypted SQLite) |
| Async Runtime | Tokio |
| Build System | Meson, Cargo |
| Packaging | Flatpak (primary), Deb, RPM |

## Installation

### Flatpak (Recommended)

```bash
# Add Flathub if not already configured
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# Install Signal You Messenger
flatpak install flathub com.signalyou.Messenger

# Run
flatpak run com.signalyou.Messenger
```

### Debian/Ubuntu

```bash
# Download the latest .deb package from releases
sudo dpkg -i signal-you-messenger_*.deb
sudo apt-get install -f  # Install dependencies
```

### Fedora/RHEL

```bash
# Download the latest .rpm package from releases
sudo dnf install signal-you-messenger-*.rpm
```

### Arch Linux (AUR)

```bash
# Using yay
yay -S signal-you-messenger

# Or manually
git clone https://aur.archlinux.org/signal-you-messenger.git
cd signal-you-messenger
makepkg -si
```

## Building from Source

### Prerequisites

#### Fedora/RHEL
```bash
sudo dnf install gtk4-devel libadwaita-devel rust cargo meson ninja-build \
    openssl-devel protobuf-compiler clang-devel
```

#### Debian/Ubuntu
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev rustc cargo meson ninja-build \
    libssl-dev protobuf-compiler libclang-dev build-essential
```

#### Arch Linux
```bash
sudo pacman -S gtk4 libadwaita rust meson ninja openssl protobuf clang
```

### Build Steps

```bash
# Clone the repository
git clone https://github.com/phlthy88/Signal-You-Messenger.git
cd Signal-You-Messenger

# Configure with Meson
meson setup build --prefix=/usr

# Build
meson compile -C build

# Install (optional)
sudo meson install -C build

# Or run directly
./build/src/signal-you-messenger
```

### Flatpak Build

```bash
# Install Flatpak SDK
flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46

# Build and install locally
flatpak-builder --user --install --force-clean build-dir flatpak/com.signalyou.Messenger.yml

# Run
flatpak run com.signalyou.Messenger
```

## Project Structure

```
Signal-You-Messenger/
├── src/                        # Rust source code
│   ├── main.rs                 # Application entry point
│   ├── application.rs          # GtkApplication implementation
│   ├── window.rs               # Main window
│   ├── ui/                     # UI components
│   │   ├── chat_list.rs        # Conversation list
│   │   ├── chat_view.rs        # Message view
│   │   ├── message_row.rs      # Individual message widget
│   │   ├── compose_bar.rs      # Message composition
│   │   └── contact_row.rs      # Contact list item
│   ├── signal/                 # Signal protocol integration
│   │   ├── client.rs           # Signal client wrapper
│   │   ├── protocol.rs         # Protocol implementation
│   │   ├── store.rs            # Encrypted storage
│   │   └── types.rs            # Signal data types
│   └── services/               # Background services
│       ├── notifications.rs    # Notification handling
│       ├── sync.rs             # Message sync
│       └── websocket.rs        # Signal WebSocket
├── data/                       # Application data files
│   ├── com.signalyou.Messenger.desktop.in
│   ├── com.signalyou.Messenger.metainfo.xml.in
│   ├── com.signalyou.Messenger.gschema.xml
│   └── icons/                  # Application icons
├── po/                         # Translations
├── flatpak/                    # Flatpak packaging
│   └── com.signalyou.Messenger.yml
├── debian/                     # Debian packaging
├── Cargo.toml                  # Rust dependencies
├── meson.build                 # Meson build configuration
└── meson_options.txt           # Build options
```

## Configuration

Signal You Messenger stores its configuration and data in standard XDG directories:

- **Config**: `~/.config/signal-you-messenger/`
- **Data**: `~/.local/share/signal-you-messenger/`
- **Cache**: `~/.cache/signal-you-messenger/`

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SIGNAL_YOU_DEBUG` | Enable debug logging | `0` |
| `SIGNAL_YOU_PROFILE` | Configuration profile name | `default` |

## Linking to Signal

Signal You Messenger requires linking to an existing Signal account:

1. Open Signal on your primary device (phone)
2. Go to Settings → Linked Devices
3. Scan the QR code displayed in Signal You Messenger
4. Your message history will sync to the desktop client

**Note**: Signal You Messenger acts as a linked device. You need the Signal mobile app to register a new account.

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting pull requests.

### Development Setup

```bash
# Clone with development dependencies
git clone https://github.com/phlthy88/Signal-You-Messenger.git
cd Signal-You-Messenger

# Set up development environment
meson setup build -Dprofile=development

# Run with debug output
SIGNAL_YOU_DEBUG=1 ./build/src/signal-you-messenger
```

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use meaningful variable and function names
- Document public APIs with rustdoc
- Write tests for new functionality

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

Signal You Messenger uses the Signal Protocol, which is licensed under the GPLv3.

## Acknowledgments

- [Signal](https://signal.org/) - For the Signal Protocol and inspiration
- [GNOME](https://gnome.org/) - For GTK4 and libadwaita
- [libsignal](https://github.com/signalapp/libsignal) - Signal Protocol implementation
- [Flare](https://gitlab.com/schmiddi-on-mobile/flare) - Inspiration for GTK Signal clients

---

<div align="center">

**Built with GTK4, libadwaita, and the Signal Protocol**

A native Linux Signal experience with Material You design

</div>
