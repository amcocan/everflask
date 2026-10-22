# Changelog

## Unreleased

### Known issues (to fix next)
- **Dev builds lose the Accessibility grant on every rebuild.** Ad-hoc signing identifies the app by code hash, so each rebuild (including `scripts/security-check.sh`) looks like a new app. The stale entry also stops MacOS from prompting again. Workaround: see "Developing" in the README. Fix: sign dev builds with a stable identity (Apple Development certificate or a self-signed one).
- **Uninstall is untested end to end.** Testing is on hold; see the Uninstall section of `docs/qa-checklist.md`.

### Fixed
- Granting Accessibility no longer needs a relaunch. EverFlask also checks `AXIsProcessTrusted`, which updates live; MacOS's cached post-event check kept saying no until restart even though nudges already worked (verified by idle-timer resets before any relaunch).
- MacOS shows the permission prompt only once per app. Added **Allow Accessibility** to the menu, which opens the right Settings pane.
- Display name is now "EverFlask" everywhere users see it: Finder, System Settings, the dev bundle (`EverFlask.app`) and the power assertion shown by `pmset`. The bundle ID is unchanged.
- Keeping chat apps Active appeared broken after being merged into the toggle. Cause: the stale Accessibility grant above, not the merged logic. Verified with a valid grant: nudges fire once idle passes 45 s, system idle time peaked at 59 s, and Teams stayed Available.

### Added
- App icon (red flask on a black rounded square) for Finder, Applications, Spotlight and System Settings, generated from `assets/app-icon/AppIcon.svg` by `scripts/make-app-icon.sh`.
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
