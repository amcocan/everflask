# Everflask

A tiny menu-bar / system-tray app that keeps your computer awake. A potion flask icon shows the state: **full** = awake, **empty** = normal. Double-click the icon to toggle.

Status: scaffold only. macOS first; Windows and Linux later.

## Layout
- `crates/everflask-core` - state and traits (no OS code)
- `crates/everflask-platform` - per-OS keep-awake backends
- `crates/everflask-app` - tray icon and event loop
- `assets/icons` - icon sources and exports
- `packaging` - per-OS bundle files
- `docs` - threat model and QA checklist

## Build
    cargo build --release
    cargo test
