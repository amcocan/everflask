# QA Checklist (MacOS)

## Release build (automated in `build-release.sh`, verified 2026-10-21)
- [x] Universal binary: `arm64` and `x86_64`, minimum MacOS 11.0 on both
- [x] Hardened runtime on; signature verifies (`codesign --verify --strict`)
- [x] DMG contains EverFlask.app and an Applications shortcut; checksum verifies
- [x] Gatekeeper rejects it (expected: unsigned by decision; README → Installing)
- [x] Release copy: launches empty, no assertions; single instance; DYLD injection blocked; no network symbols
- [x] On a second Mac or user account: download from GitHub, Open Anyway flow works as the README describes

- [x] Toggle on: `pmset -g assertions` shows the EverFlask assertion
- [x] Toggle off: assertion gone
- [x] Icon matches state in light and dark menu bar
- [x] Double-click toggles; menu works
- [x] Single instance enforced
- [x] Quit releases the assertion

## Keeping chat apps Active (merged into the toggle)
- [x] First fill shows the MacOS Accessibility prompt; "Don't Allow" leaves keep-awake working, and the tooltip says to allow Accessibility
- [x] After granting, the tooltip changes to "Keeping awake and active" within 60 s, without relaunching
- [x] If MacOS doesn't prompt, **Allow Accessibility** opens Settings → Privacy & Security → Accessibility with EverFlask listed; it disappears from the menu once access is active and comes back if access is revoked
- [x] Granted + full + idle: `ioreg -c IOHIDSystem | grep HIDIdleTime` never passes ~105 s
- [x] Granted + full + actively typing: no nudges are posted
- [x] Pointer never moves
- [x] Teams stays Available past its 5 min idle timeout
- [x] Emptying the flask or quitting stops nudges (idle time climbs past 105 s)
- [x] Revoking Accessibility while full switches the tooltip to keep-awake-only within 60 s
- [x] Manual test passes: `cargo test -p everflask-platform --test macos_activity -- --ignored`

## Single instance
- [x] Launching EverFlask again while it runs doesn't add a second flask

## Uninstall
- [x] Menu → Uninstall EverFlask → Cancel changes nothing
- [x] Confirm: app is in the Trash; EverFlask is gone from Settings → Accessibility
- [x] `~/Library/Preferences/io.github.amcocan.everflask.plist` doesn't exist
- [x] `defaults read com.apple.universalaccessAuthWarning` has no EverFlask keys and still has every other key
- [x] `pmset -g assertions` shows no EverFlask assertions
- [x] `packaging/macos/uninstall.sh` on an already-deleted install removes the same items and prints guidance for the permission
