# Changelog

## Unreleased

### Known issues (to fix next)
- **Keeping chat apps Active doesn't work since it was merged into the toggle.** Filling the flask keeps the Mac awake, but Teams still goes Away. Not yet investigated. Things to check first:
  - Whether the Accessibility grant still matches the current build. Every dev-bundle rebuild (including `scripts/security-check.sh`) changes the ad-hoc signature, which can silently invalidate the grant while Settings still shows it switched on.
  - What the tooltip says while full ("keeping awake and active" or the "allow Accessibility" message).
  - Whether the new 45-second idle check ever lets a nudge through.
  - Run the manual test: `cargo test -p everflask-platform --test macos_activity -- --ignored`.
- **Uninstall is untested end to end.** Testing is on hold; see the Uninstall section of `docs/qa-checklist.md`.
- **`Info.plist` still uses the display name "Everflask"** while the docs say "EverFlask". Changing it means granting Accessibility again.

### Added
- Menu-bar flask that keeps the Mac awake (IOKit power assertions); double-click to toggle.
- Designed flask artwork with instant light/dark menu-bar switching.
- Keeping chat apps Active while full (zero-distance mouse move after 45 s idle; needs Accessibility).
- Uninstall EverFlask… menu item and `packaging/macos/uninstall.sh`.
- Single-instance enforcement.
- `scripts/security-check.sh`, threat model, QA checklist.

### Security
- Hardened runtime in the dev bundle (blocks DYLD injection).
- Fixed: synthetic event could have jumped the pointer to (0, 0).
- CI: `cargo audit`, `cargo deny`, actions pinned to SHAs, read-only token.
