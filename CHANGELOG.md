# Changelog

## Unreleased

### Known issues (to fix next)
- **Dev builds lose the Accessibility grant on every rebuild.** Ad-hoc signing identifies the app by code hash, so each rebuild (including `scripts/security-check.sh`) looks like a new app. The stale entry also stops MacOS from prompting again. Workaround: see "Developing" in the README. Fix: sign dev builds with a stable identity (Apple Development certificate or a self-signed one).
- **Uninstall is untested end to end.** Testing is on hold; see the Uninstall section of `docs/qa-checklist.md`.
- **`Info.plist` still uses the display name "Everflask"** while the docs say "EverFlask". Changing it means granting Accessibility again.

### Fixed
- Keeping chat apps Active appeared broken after being merged into the toggle. Cause: the stale Accessibility grant above, not the merged logic. Verified with a valid grant: nudges fire once idle passes 45 s, system idle time peaked at 59 s, and Teams stayed Available.

### Added
- `EVERFLASK_DEBUG` diagnostic log (permission status and every nudge decision), off by default.
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
