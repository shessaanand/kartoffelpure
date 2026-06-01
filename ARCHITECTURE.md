# KartoffelPure Architecture

## Core Principles

- Privacy first
- Minimal UI
- Modular architecture
- Rust only
- Open source

## Layers

### UI Layer
GTK4 application window.

Responsibilities:
- Tabs
- Toolbar
- Address bar
- Settings UI

### Browser Layer

Responsibilities:
- Navigation
- Tab management
- History
- Bookmarks

### Rendering Layer

Technology:
- WebKitGTK

Responsibilities:
- Render websites
- Handle navigation events

### Database Layer

Technology:
- SQLite

Responsibilities:
- History
- Bookmarks
- Settings

### Privacy Layer

Responsibilities:
- Tracker blocking
- Cookie controls
- Permission management
