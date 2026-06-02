# KartoffelPure

Privacy-focused browser (Rust + GTK4 + WebKitGTK).

## Requirements (Ubuntu)

```bash
sudo apt install build-essential pkg-config libgtk-4-dev libwebkitgtk-6.0-dev
```

You also need a recent Rust toolchain (`rustup` recommended).

## Build and run

```bash
cargo fmt
cargo clippy -- -D warnings
cargo build
cargo run
```

Release build:

```bash
cargo build --release
./target/release/kartoffelpure
```

## v0.2.2 scope

- Everything in v0.2.1 (tabs, overflow, layout modes)
- **Persistent browsing history** (SQLite at `~/.local/share/kartoffelpure/kartoffelpure.db`)
- History dialog (toolbar **History** button): search, open in active tab, delete entry, clear all
- History recorded on successful page load completion only
- No bookmarks, downloads, settings UI, sync, or extensions

### Tab layout API

```rust
use kartoffelpure::ui::{BrowserWindow, TabLayoutMode, TabStripConfig};

// Vertical sidebar tabs at window creation:
let config = TabStripConfig::with_mode(TabLayoutMode::Vertical);
let window = BrowserWindow::with_tab_config(app, config);

// Or switch at runtime:
window.set_tab_layout_mode(TabLayoutMode::Horizontal);
```

See [ROADMAP.md](ROADMAP.md) for later milestones.
