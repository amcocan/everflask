# Changelog

## Unreleased

### Known issues (to fix next)
- **Dev builds lose the Accessibility grant on every rebuild.** Ad-hoc signing identifies the app by code hash, so each rebuild (including `scripts/security-check.sh`) looks like a new app. The stale entry also stops MacOS from prompting again. Workaround: see "Developing" in the README. Releases are unsigned by decision, so this stays; `dev-bundle.sh` clears the stale grant on each build.

### Fixed
- Uninstall tested end to end: the in-app uninstall removed the app (to Trash), preferences, permission, exactly EverFlask's 3 warning records (other 18 untouched) and the power assertions. The script route removed the preferences and guided the manual permission step.
- Granting Accessibility no longer needs a relaunch. EverFlask also checks `AXIsProcessTrusted`, which updates live; MacOS's cached post-event check kept saying no until restart even though nudges already worked (verified by idle-timer resets before any relaunch).
- MacOS shows the permission prompt only once per app. Added **Allow Accessibility** to the menu, which opens the right Settings pane.
- Display name is now "EverFlask" everywhere users see it: Finder, System Settings, the dev bundle (`EverFlask.app`) and the power assertion shown by `pmset`. The bundle ID is unchanged.
- Keeping chat apps Active appeared broken after being merged into the toggle. Cause: the stale Accessibility grant above, not the merged logic. Verified with a valid grant: nudges fire once idle passes 45 s, system idle time peaked at 59 s, and Teams stayed Available.

### Changed
- Releases will be unsigned (no Apple Developer ID); README "Installing" explains Open Anyway, checksums and building from source.
- `uninstall.sh` prints the manual Accessibility step once and opens the right Settings pane when MacOS no longer recognises the deleted app.

### Added
- `packaging/macos/build-release.sh`: universal DMG with SHA-256 checksum, self-verifying (architectures, minimum MacOS, hardened runtime, signature).
- Release workflow: a version tag builds the DMG and creates a draft GitHub release; manual runs build a dry-run artifact. Build attestations once the repo is public.
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
