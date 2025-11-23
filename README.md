<div align="center">
<img width="1200" height="475" alt="Signal You Messenger Banner" src="https://github.com/user-attachments/assets/0aa67016-6eaf-458a-adb2-6e31a0763ed6" />

# Signal You Messenger

**A GTK4/libadwaita Material You fork of Signal Messenger for Linux**

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![GTK4](https://img.shields.io/badge/GTK-4.12-green.svg)](https://gtk.org/)
[![libadwaita](https://img.shields.io/badge/libadwaita-1.4-purple.svg)](https://gnome.pages.gitlab.gnome.org/libadwaita/)
[![Signal Protocol](https://img.shields.io/badge/Signal-Protocol-blue.svg)](https://signal.org/)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)

[Features](#features) | [Installation](#installation) | [Dependencies](#dependencies) | [Building](#building-from-source) | [Contributing](#contributing)

</div>

---

## Overview

Signal You Messenger is a native GTK4/libadwaita Signal client for Linux, designed with Material You (Material Design 3) aesthetics and deep GNOME integration. This project provides a modern, adaptive Signal experience that feels native on the Linux desktop while maintaining full compatibility with the Signal Protocol.

### Why Signal You?

- **Native GTK4 Experience**: Built with GTK4 and libadwaita for seamless GNOME integration
- **Material You Theming**: Adaptive color schemes that follow your system accent colors
- **Full Signal Protocol**: Complete cryptographic implementation with X3DH and Double Ratchet
- **Flatpak First**: Primary distribution through Flathub for easy installation and updates
- **Lightweight**: Native performance without Electron overhead

## Features

### Signal Protocol Integration (Full Implementation)
- **X3DH Key Exchange**: Extended Triple Diffie-Hellman for secure initial key agreement
- **Double Ratchet Algorithm**: Forward secrecy and post-compromise security
- **End-to-end encryption**: Full Signal Protocol with AES-256-GCM encryption
- **Pre-key bundles**: One-time and signed pre-keys for asynchronous communication
- **Safety numbers**: Fingerprint verification for contact identity validation
- **Perfect forward secrecy**: Compromised keys don't affect past messages
- **Sealed sender**: Enhanced metadata protection (planned)

### Cryptographic Implementation
- **X25519**: Diffie-Hellman key exchange on Curve25519
- **Ed25519**: Digital signatures for identity keys
- **AES-256-GCM**: Authenticated encryption for messages
- **HKDF-SHA256**: Key derivation for session keys
- **HMAC-SHA256**: Message authentication codes
- **SQLCipher**: Encrypted local database storage

### Core Messaging
- **Private chats**: One-on-one encrypted conversations
- **Group messaging**: Secure group conversations with Signal groups
- **Media sharing**: Send photos, videos, files, and voice messages
- **Disappearing messages**: Auto-delete messages after a set time
- **Read receipts**: Optional delivery and read confirmations
- **Typing indicators**: See when contacts are typing
- **Device linking**: QR code-based account linking

### GTK4/libadwaita Design
- **Adaptive layouts**: Responsive design for any screen size
- **Material You colors**: Dynamic theming based on system accent color
- **Dark/Light modes**: Automatic theme switching with system preference
- **GNOME integration**: Native notifications, app indicators, and portal support
- **Touch-friendly**: Optimized for touchscreen devices and convertibles

### Desktop Integration
- **System notifications**: Native GNOME notification support
- **Quick reply**: Reply directly from notifications
- **Do Not Disturb**: Respect GNOME's notification settings
- **Background operation**: Continue receiving messages when minimized
- **File portal**: Secure file access through XDG portals

## Technology Stack

| Component | Technology |
|-----------|------------|
| UI Framework | GTK4 ≥4.12, libadwaita ≥1.4 |
| Language | Rust 1.75+ (2021 Edition) |
| Signal Protocol | Custom implementation (X3DH + Double Ratchet) |
| Cryptography | x25519-dalek, ed25519-dalek, aes-gcm, hkdf |
| Database | SQLCipher (encrypted SQLite via rusqlite) |
| Async Runtime | Tokio (full features) |
| HTTP Client | reqwest with rustls-tls |
| WebSocket | tokio-tungstenite |
| Build System | Meson ≥1.0, Cargo |
| Packaging | Flatpak (primary), Deb, RPM |

## Dependencies

### Complete Dependency List for Building

#### System Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| GTK4 | ≥4.12 | UI framework |
| libadwaita | ≥1.4 | GNOME design patterns |
| Rust | ≥1.75 | Programming language |
| Cargo | (with Rust) | Rust package manager |
| Meson | ≥1.0 | Build system |
| Ninja | any | Build backend |
| OpenSSL | any | TLS/cryptography |
| Protobuf Compiler | any | Protocol buffer compilation |
| Clang/LLVM | any | bindgen requirements |
| pkg-config | any | Library detection |
| GLib | ≥2.66 | Core GNOME library |
| gettext | any | Internationalization |

#### Rust Crate Dependencies (Cargo.toml)

**GTK/UI:**
| Crate | Version | Purpose |
|-------|---------|---------|
| gtk4 | 0.8 | GTK4 Rust bindings |
| libadwaita | 0.6 | libadwaita Rust bindings |

**Async Runtime:**
| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.35 | Async runtime (full features) |
| async-channel | 2.1 | Async channels |
| futures | 0.3 | Future utilities |

**Signal Protocol Cryptography:**
| Crate | Version | Purpose |
|-------|---------|---------|
| x25519-dalek | 2.0 | X25519 Diffie-Hellman |
| ed25519-dalek | 2.1 | Ed25519 signatures |
| curve25519-dalek | 4.1 | Low-level curve operations |
| aes-gcm | 0.10 | AES-256-GCM encryption |
| hkdf | 0.12 | HKDF key derivation |
| sha2 | 0.10 | SHA-256/SHA-512 hashing |
| hmac | 0.12 | HMAC authentication |
| ring | 0.17 | Additional cryptography |

**Networking:**
| Crate | Version | Purpose |
|-------|---------|---------|
| reqwest | 0.11 | HTTP client (rustls-tls) |
| tokio-tungstenite | 0.21 | WebSocket client |
| http | 1.1 | HTTP types |

**Serialization/Data:**
| Crate | Version | Purpose |
|-------|---------|---------|
| serde | 1.0 | Serialization framework |
| serde_json | 1.0 | JSON support |
| prost | 0.12 | Protocol Buffers |
| base64 | 0.21 | Base64 encoding |
| hex | 0.4 | Hex encoding |
| urlencoding | 2.1 | URL encoding |
| bytes | 1.5 | Byte buffer utilities |

**Database:**
| Crate | Version | Purpose |
|-------|---------|---------|
| rusqlite | 0.31 | SQLite (with SQLCipher) |

**Utilities:**
| Crate | Version | Purpose |
|-------|---------|---------|
| uuid | 1.6 | UUID generation |
| rand | 0.8 | Random number generation |
| chrono | 0.4 | Date/time handling |
| zeroize | 1.7 | Secure memory clearing |
| anyhow | 1.0 | Error handling |
| thiserror | 1.0 | Error types |
| directories | 5.0 | XDG directory paths |
| toml | 0.8 | TOML configuration |

**Logging:**
| Crate | Version | Purpose |
|-------|---------|---------|
| log | 0.4 | Logging facade |
| env_logger | 0.10 | Environment-based logging |
| tracing | 0.1 | Structured logging |
| tracing-subscriber | 0.3 | Tracing output |

**QR Code/Media:**
| Crate | Version | Purpose |
|-------|---------|---------|
| qrcode | 0.13 | QR code generation |
| image | 0.24 | Image processing |
| mime_guess | 2.0 | MIME type detection |

**i18n:**
| Crate | Version | Purpose |
|-------|---------|---------|
| gettext-rs | 0.7 | Internationalization |

**Build Dependencies:**
| Crate | Version | Purpose |
|-------|---------|---------|
| prost-build | 0.12 | Protobuf code generation |

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

### Prerequisites by Distribution

#### Fedora/RHEL/CentOS Stream

```bash
# Install all required dependencies
sudo dnf install \
    gtk4-devel \
    libadwaita-devel \
    rust \
    cargo \
    meson \
    ninja-build \
    openssl-devel \
    protobuf-compiler \
    protobuf-devel \
    clang-devel \
    llvm-devel \
    sqlite-devel \
    glib2-devel \
    gettext-devel \
    desktop-file-utils \
    pkgconf-pkg-config
```

#### Debian/Ubuntu

```bash
# Install all required dependencies
sudo apt install \
    libgtk-4-dev \
    libadwaita-1-dev \
    rustc \
    cargo \
    meson \
    ninja-build \
    libssl-dev \
    protobuf-compiler \
    libprotobuf-dev \
    libclang-dev \
    llvm-dev \
    libsqlite3-dev \
    libglib2.0-dev \
    gettext \
    desktop-file-utils \
    build-essential \
    pkg-config
```

#### Arch Linux

```bash
# Install all required dependencies
sudo pacman -S \
    gtk4 \
    libadwaita \
    rust \
    meson \
    ninja \
    openssl \
    protobuf \
    clang \
    llvm \
    sqlite \
    glib2 \
    gettext \
    desktop-file-utils \
    pkgconf \
    base-devel
```

#### openSUSE

```bash
# Install all required dependencies
sudo zypper install \
    gtk4-devel \
    libadwaita-devel \
    rust \
    cargo \
    meson \
    ninja \
    libopenssl-devel \
    protobuf-devel \
    clang-devel \
    llvm-devel \
    sqlite3-devel \
    glib2-devel \
    gettext-tools \
    desktop-file-utils \
    pkg-config
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
# Install Flatpak SDK and Rust extension
flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//23.08

# Build and install locally
flatpak-builder --user --install --force-clean build-dir flatpak/com.signalyou.Messenger.yml

# Run
flatpak run com.signalyou.Messenger
```

### Development Build

```bash
# Set up development environment with debug profile
meson setup build -Dprofile=development

# Build
meson compile -C build

# Run with debug output
SIGNAL_YOU_DEBUG=1 ./build/src/signal-you-messenger
```

## Project Structure

```
Signal-You-Messenger/
├── src/                        # Rust source code
│   ├── main.rs                 # Application entry point
│   ├── application.rs          # GtkApplication implementation
│   ├── window.rs               # Main window
│   ├── config.rs.in            # Build-time configuration template
│   ├── ui/                     # UI components
│   │   ├── mod.rs              # UI module exports
│   │   ├── chat_list.rs        # Conversation list
│   │   ├── chat_view.rs        # Message view
│   │   ├── message_row.rs      # Individual message widget
│   │   ├── compose_bar.rs      # Message composition
│   │   ├── contact_row.rs      # Contact list item
│   │   └── link_device_view.rs # Device linking QR view
│   ├── signal/                 # Signal Protocol implementation
│   │   ├── mod.rs              # Signal module exports
│   │   ├── crypto.rs           # Cryptographic primitives (X25519, Ed25519, AES-GCM)
│   │   ├── x3dh.rs             # X3DH key agreement protocol
│   │   ├── ratchet.rs          # Double Ratchet algorithm
│   │   ├── protocol.rs         # High-level protocol interface
│   │   ├── client.rs           # Signal service client
│   │   ├── store.rs            # SQLCipher encrypted storage
│   │   └── types.rs            # Signal data types
│   └── services/               # Background services
│       ├── mod.rs              # Services module exports
│       ├── notifications.rs    # Notification handling
│       ├── sync.rs             # Message synchronization
│       └── websocket.rs        # Signal WebSocket connection
├── data/                       # Application data files
│   ├── meson.build             # Data build configuration
│   ├── com.signalyou.Messenger.desktop.in
│   ├── com.signalyou.Messenger.metainfo.xml.in
│   ├── com.signalyou.Messenger.gschema.xml
│   └── icons/                  # Application icons
│       └── meson.build         # Icons build configuration
├── po/                         # Translations
│   └── meson.build             # i18n build configuration
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
| `RUST_BACKTRACE` | Show Rust backtraces | `0` |
| `RUST_LOG` | Logging level (trace, debug, info, warn, error) | `info` |

## Linking to Signal

Signal You Messenger requires linking to an existing Signal account:

1. Open Signal on your primary device (phone)
2. Go to Settings → Linked Devices
3. Scan the QR code displayed in Signal You Messenger
4. Your message history will sync to the desktop client

**Note**: Signal You Messenger acts as a linked device. You need the Signal mobile app to register a new account.

## Security

Signal You Messenger implements the complete Signal Protocol:

- **X3DH (Extended Triple Diffie-Hellman)**: Secure initial key exchange supporting asynchronous messaging
- **Double Ratchet**: Forward secrecy through continuous key rotation
- **AES-256-GCM**: Authenticated encryption for all messages
- **SQLCipher**: Encrypted local database for message and key storage
- **Zeroize**: Secure memory clearing for sensitive cryptographic material

All cryptographic operations use audited, well-tested Rust crates from the RustCrypto project.

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
SIGNAL_YOU_DEBUG=1 RUST_LOG=debug ./build/src/signal-you-messenger
```

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Run clippy for linting (`cargo clippy`)
- Use meaningful variable and function names
- Document public APIs with rustdoc
- Write tests for new functionality

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_session_establishment
```

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

Signal You Messenger uses the Signal Protocol, which is licensed under the GPLv3.

## Acknowledgments

- [Signal](https://signal.org/) - For the Signal Protocol and inspiration
- [GNOME](https://gnome.org/) - For GTK4 and libadwaita
- [RustCrypto](https://github.com/RustCrypto) - Cryptographic implementations
- [Dalek Cryptography](https://github.com/dalek-cryptography) - Curve25519/Ed25519 implementations
- [Flare](https://gitlab.com/schmiddi-on-mobile/flare) - Inspiration for GTK Signal clients

---

<div align="center">

**Built with GTK4, libadwaita, and the Signal Protocol**

A native Linux Signal experience with Material You design

**Version 1.0.0** | Rust 1.75+ | GTK4 4.12+ | libadwaita 1.4+

</div>
