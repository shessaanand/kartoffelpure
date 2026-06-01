# KartoffelPure

Privacy-focused browser (Rust + GTK4 + WebKitGTK). **v0.1** is a single-window shell: toolbar, address bar, Back / Forward / Reload, and one web view.

## Requirements (Ubuntu)

```bash
sudo apt install build-essential pkg-config libgtk-4-dev libwebkitgtk-6.0-dev
```

You also need a recent Rust toolchain (`rustup` recommended).

## Build and run

```bash
cargo build
cargo run
```

Release build:

```bash
cargo build --release
./target/release/kartoffelpure
```

## v0.1 scope

- One window, one active web view (no tabs)
- Default page: https://example.com
- Address bar navigation (Enter), back, forward, reload
- No bookmarks, history database, downloads, settings UI, or extensions

See [ROADMAP.md](ROADMAP.md) for later milestones.
