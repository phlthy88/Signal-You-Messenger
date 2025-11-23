# Building Signal You Messenger

This document explains how to build Signal You Messenger, a native GTK4/libadwaita Signal client for Linux.

## Prerequisites

### Runtime Dependencies

- GTK 4.12 or later
- libadwaita 1.4 or later
- SQLite 3 with SQLCipher
- OpenSSL 3.0 or later

### Build Dependencies

#### Fedora/RHEL

```bash
sudo dnf install gtk4-devel libadwaita-devel rust cargo meson ninja-build \
    openssl-devel protobuf-compiler clang-devel sqlite-devel \
    blueprint-compiler gettext-devel desktop-file-utils appstream
```

#### Debian/Ubuntu

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev rustc cargo meson ninja-build \
    libssl-dev protobuf-compiler libclang-dev build-essential \
    libsqlite3-dev blueprint-compiler gettext desktop-file-utils appstream
```

#### Arch Linux

```bash
sudo pacman -S gtk4 libadwaita rust meson ninja openssl protobuf clang \
    sqlite blueprint-compiler gettext desktop-file-utils appstream
```

### Flatpak Build Dependencies

```bash
# Install Flatpak and Builder
sudo dnf install flatpak flatpak-builder  # Fedora
sudo apt install flatpak flatpak-builder   # Debian/Ubuntu

# Add Flathub repository
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# Install GNOME SDK
flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46 \
    org.freedesktop.Sdk.Extension.rust-stable//23.08
```

## Building from Source

### Standard Build

```bash
# Clone the repository
git clone https://github.com/phlthy88/Signal-You-Messenger.git
cd Signal-You-Messenger

# Configure with Meson
meson setup build --prefix=/usr

# Build
meson compile -C build

# Run tests
meson test -C build

# Install (optional)
sudo meson install -C build
```

### Development Build

```bash
# Configure for development
meson setup build -Dprofile=development

# Build
meson compile -C build

# Run directly
./build/src/signal-you-messenger
```

### Debug Build

```bash
# Build with debug symbols
meson setup build --buildtype=debug

# Run with debug logging
SIGNAL_YOU_DEBUG=1 ./build/src/signal-you-messenger
```

## Flatpak Build (Recommended)

The recommended way to build and distribute Signal You Messenger is via Flatpak:

```bash
# Build and install locally
flatpak-builder --user --install --force-clean build-dir flatpak/com.signalyou.Messenger.yml

# Run
flatpak run com.signalyou.Messenger

# Build a distributable bundle
flatpak-builder --repo=repo --force-clean build-dir flatpak/com.signalyou.Messenger.yml
flatpak build-bundle repo signal-you-messenger.flatpak com.signalyou.Messenger
```

### Flatpak Development

For faster iteration during development:

```bash
# Build without installing
flatpak-builder --force-clean build-dir flatpak/com.signalyou.Messenger.yml

# Run from build directory
flatpak-builder --run build-dir flatpak/com.signalyou.Messenger.yml signal-you-messenger
```

## Debian Package Build

```bash
# Install build dependencies
sudo apt install debhelper devscripts

# Build the package
dpkg-buildpackage -us -uc -b

# Install
sudo dpkg -i ../signal-you-messenger_*.deb
```

## RPM Package Build

```bash
# Install build dependencies
sudo dnf install rpm-build rpmdevtools

# Setup RPM build tree
rpmdev-setuptree

# Build the package
rpmbuild -bb signal-you-messenger.spec
```

## Project Structure

```
Signal-You-Messenger/
├── src/                        # Rust source code
│   ├── main.rs                 # Entry point
│   ├── application.rs          # GtkApplication
│   ├── window.rs               # Main window
│   ├── config.rs.in            # Generated config
│   ├── style.css               # Application styles
│   ├── ui/                     # UI components
│   │   ├── *.rs                # Rust widget code
│   │   └── *.blp               # Blueprint UI templates
│   ├── signal/                 # Signal protocol
│   │   ├── client.rs           # High-level client
│   │   ├── protocol.rs         # Protocol wrapper
│   │   ├── store.rs            # Encrypted storage
│   │   └── types.rs            # Data types
│   └── services/               # Background services
│       ├── notifications.rs
│       ├── sync.rs
│       └── websocket.rs
├── data/                       # Application data
│   ├── com.signalyou.Messenger.desktop.in
│   ├── com.signalyou.Messenger.metainfo.xml.in
│   ├── com.signalyou.Messenger.gschema.xml
│   └── icons/
├── po/                         # Translations
├── flatpak/                    # Flatpak packaging
│   └── com.signalyou.Messenger.yml
├── debian/                     # Debian packaging
├── Cargo.toml                  # Rust dependencies
├── meson.build                 # Build configuration
└── meson_options.txt           # Build options
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SIGNAL_YOU_DEBUG` | Enable debug logging (1 = enabled) | `0` |
| `SIGNAL_YOU_PROFILE` | Configuration profile name | `default` |
| `G_MESSAGES_DEBUG` | GTK debug messages | - |
| `GTK_DEBUG` | GTK debugging options | - |

## Troubleshooting

### Blueprint Compiler Not Found

Install blueprint-compiler:

```bash
# Fedora
sudo dnf install blueprint-compiler

# Ubuntu/Debian (if not in repos, install from pip)
pip install blueprint-compiler

# Arch
sudo pacman -S blueprint-compiler
```

### Rust Version Too Old

The project requires Rust 1.75 or later:

```bash
# Update Rust
rustup update stable
```

### Missing GTK4 or libadwaita

Ensure you have the development packages:

```bash
# Check versions
pkg-config --modversion gtk4
pkg-config --modversion libadwaita-1
```

### Flatpak Build Fails

Ensure you have the correct SDK version:

```bash
flatpak list --runtime | grep org.gnome
# Should show org.gnome.Platform and org.gnome.Sdk version 46
```

### Database Errors

The application uses SQLCipher for encrypted storage. If you see database errors:

```bash
# Clear application data (will require re-linking)
rm -rf ~/.local/share/signal-you-messenger/
```

## Contributing

See the project README for contribution guidelines.

## License

GPL-3.0-or-later - See [LICENSE](LICENSE) for details.
