# Everflask

Rust menu-bar/tray app that keeps the machine awake. Flask icon: full = active, empty = inactive. Double-click toggles.

## Conventions
- Cargo workspace: `core` (pure logic), `platform` (OS backends behind the `KeepAwake` trait), `app` (tray/UI).
- No OS-specific code in `core`. Every `unsafe` block needs a `// SAFETY:` comment.
- No network access. Config is user-only readable.
- Before commit: `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`.
- Release gate: `cargo audit` and `cargo deny check`.
- Target order: macOS first, then Windows, Linux.
