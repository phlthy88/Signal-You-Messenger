<div align="center">
<img width="1200" height="475" alt="Signal You Messenger Banner" src="https://github.com/user-attachments/assets/0aa67016-6eaf-458a-adb2-6e31a0763ed6" />

# Signal You Messenger

**Secure messaging application with AI-powered features**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Node.js](https://img.shields.io/badge/Node.js-18%2B-green.svg)](https://nodejs.org/)
[![Electron](https://img.shields.io/badge/Electron-33-blue.svg)](https://www.electronjs.org/)
[![React](https://img.shields.io/badge/React-19-61dafb.svg)](https://react.dev/)

[Features](#features) | [Installation](#installation) | [Development](#development) | [Building](#building) | [Documentation](#documentation)

</div>

---

## Overview

Signal You Messenger is a modern, cross-platform desktop messaging application that combines secure communication with AI-powered features. Built with Electron, React, and Node.js, it delivers a native experience on Linux, macOS, and Windows.

## Features

### Core Messaging
- **Real-time messaging** - Instant message delivery using WebSocket connections
- **Chat management** - Create, organize, and manage conversations
- **Contact list** - Add and manage contacts with ease
- **Message history** - Full chat history with local storage

### AI Integration
- **Gemini AI** - Powered by Google's Gemini API for intelligent responses
- **Smart suggestions** - AI-assisted message composition
- **Context-aware responses** - Intelligent conversation understanding

### Desktop Experience
- **Native notifications** - System-level notification support
- **System tray** - Quick access from the system tray with context menu
- **Keyboard shortcuts** - Efficient navigation with hotkeys
- **Dark/Light themes** - Material Design 3 theming with theme switching
- **Emoji picker** - Rich emoji support for expressive messaging

### Security & Authentication
- **JWT authentication** - Secure token-based authentication
- **Password hashing** - bcrypt password encryption
- **Session management** - Persistent login sessions

### File Sharing
- **File uploads** - Share files and media in chats
- **Image support** - Send and receive images
- **Configurable limits** - Adjustable file size restrictions

## Technology Stack

| Component | Technology |
|-----------|------------|
| Frontend | React 19, TypeScript, Vite, Zustand |
| Backend | Node.js, Express, SQLite (better-sqlite3) |
| Desktop | Electron 33 |
| Real-time | WebSocket (ws) |
| AI | Google Gemini API |
| Styling | Tailwind CSS, Material Design 3 |
| Testing | Vitest, React Testing Library |

## Installation

### Prerequisites
- Node.js 18 or higher
- npm 9 or higher

### Quick Start

```bash
# Clone the repository
git clone https://github.com/phlthy88/Signal-You-Messenger.git
cd Signal-You-Messenger

# Install dependencies (includes server dependencies)
npm install

# Set up environment variables
cp .env.example .env.local
# Edit .env.local with your API keys

# Start development server
npm run dev
```

### Pre-built Packages

Download the latest release for your platform:

| Platform | Formats |
|----------|---------|
| **Linux** | `.deb`, `.rpm`, `.AppImage`, `.flatpak` |
| **macOS** | `.dmg`, `.zip` |
| **Windows** | `.exe` (NSIS installer), portable |

## Development

### Available Scripts

```bash
# Web development
npm run dev              # Start Vite dev server

# Electron development
npm run dev:electron     # Start Electron with hot reload

# Backend server
npm run server           # Start backend in development mode
npm run server:start     # Start backend in production mode

# Testing
npm run test             # Run tests
npm run test:ui          # Run tests with UI
npm run test:coverage    # Run tests with coverage

# Linting
npm run lint             # Lint source code
```

### Project Structure

```
Signal-You-Messenger/
├── src/                    # Frontend source code
│   ├── components/         # React components
│   │   ├── AuthForm.tsx    # Authentication UI
│   │   ├── ChatList.tsx    # Chat list sidebar
│   │   ├── ChatWindow.tsx  # Main chat interface
│   │   ├── ContactList.tsx # Contact management
│   │   ├── EmojiPicker.tsx # Emoji selection
│   │   ├── Settings.tsx    # App settings
│   │   └── Sidebar.tsx     # Navigation sidebar
│   ├── contexts/           # React contexts
│   ├── services/           # API and WebSocket services
│   ├── store/              # Zustand state management
│   └── test/               # Test files
├── server/                 # Backend server
│   ├── routes/             # API endpoints
│   │   ├── auth.js         # Authentication
│   │   ├── chats.js        # Chat operations
│   │   ├── contacts.js     # Contact management
│   │   ├── ai.js           # AI integration
│   │   ├── settings.js     # User settings
│   │   └── upload.js       # File uploads
│   ├── services/           # Backend services
│   ├── middleware/         # Express middleware
│   ├── models/             # Database models
│   └── config/             # Configuration
├── electron/               # Electron main process
│   ├── main.js             # Main entry point
│   └── preload.js          # Preload script
├── build/                  # Build resources & icons
├── debian/                 # Debian packaging
├── flatpak/                # Flatpak packaging
└── scripts/                # Build scripts
```

## Building

### Build for Current Platform

```bash
npm run electron:build
```

### Platform-Specific Builds

```bash
# Linux
npm run electron:build:linux      # All Linux formats
npm run electron:build:deb        # Debian package
npm run electron:build:appimage   # AppImage
npm run electron:build:flatpak    # Flatpak
npm run electron:build:rpm        # RPM package

# macOS
npm run electron:build:mac

# Windows
npm run electron:build:win

# All platforms
npm run electron:build:all
```

### Manual Package Building

For more control over the build process:

```bash
# Debian package
./scripts/build-deb.sh

# Flatpak bundle
./scripts/build-flatpak.sh

# Generate icons (first time)
./scripts/generate-icons.sh
```

See [BUILDING.md](./BUILDING.md) for detailed build instructions and troubleshooting.

## Configuration

### Environment Variables

Create a `.env.local` file in the root directory:

```env
GEMINI_API_KEY=your_gemini_api_key_here
```

### Server Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Backend server port | `3001` |
| `JWT_SECRET` | JWT signing secret | Auto-generated |
| `GEMINI_API_KEY` | Google Gemini API key | Optional |
| `CORS_ORIGIN` | Allowed CORS origin | `http://localhost:3000` |
| `MAX_FILE_SIZE` | Max upload size (bytes) | `10485760` (10MB) |

## Documentation

- [Building Guide](./BUILDING.md) - Detailed build instructions for all platforms
- [API Documentation](./server/README.md) - Backend API reference

## Architecture

Signal You Messenger uses a modern architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                     Electron Shell                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────┐    ┌─────────────────────────────┐│
│  │    React Frontend   │◄──►│     Express Backend         ││
│  │                     │    │                             ││
│  │  • Components       │    │  • REST API                 ││
│  │  • Zustand Store    │    │  • WebSocket Server         ││
│  │  • Theme Context    │    │  • SQLite Database          ││
│  │  • WebSocket Client │    │  • Gemini AI Service        ││
│  └─────────────────────┘    └─────────────────────────────┘│
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**Built with Electron, React, and Node.js**

Made with care for secure, intelligent messaging

</div>
