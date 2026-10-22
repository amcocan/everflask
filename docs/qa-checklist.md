# QA Checklist (MacOS)

## Release build (automated in `build-release.sh`, verified 2026-10-21)
- [x] Universal binary: `arm64` and `x86_64`, minimum MacOS 11.0 on both
- [x] Hardened runtime on; signature verifies (`codesign --verify --strict`)
- [x] DMG contains EverFlask.app and an Applications shortcut; checksum verifies
- [x] Gatekeeper rejects it (expected: unsigned by decision; README → Installing)
- [x] Release copy: launches empty, no assertions; single instance; DYLD injection blocked; no network symbols
- [ ] On a second Mac or user account: download from GitHub, Open Anyway flow works as the README describes

- [ ] Toggle on: `pmset -g assertions` shows the EverFlask assertion
- [ ] Toggle off: assertion gone
- [ ] Icon matches state in light and dark menu bar
- [ ] Double-click toggles; menu works
- [ ] Single instance enforced
- [ ] Quit releases the assertion

## Keeping chat apps Active (merged into the toggle)
- [ ] First fill shows the MacOS Accessibility prompt; "Don't Allow" leaves keep-awake working, and the tooltip says to allow Accessibility
- [ ] After granting, the tooltip changes to "Keeping awake and active" within 60 s, without relaunching
- [ ] If MacOS doesn't prompt, **Allow Accessibility** opens Settings → Privacy & Security → Accessibility with EverFlask listed; it disappears from the menu once access is active and comes back if access is revoked
- [ ] Granted + full + idle: `ioreg -c IOHIDSystem | grep HIDIdleTime` never passes ~105 s
- [ ] Granted + full + actively typing: no nudges are posted
- [ ] Pointer never moves
- [ ] Teams stays Available past its 5 min idle timeout
- [ ] Emptying the flask or quitting stops nudges (idle time climbs past 105 s)
- [ ] Revoking Accessibility while full switches the tooltip to keep-awake-only within 60 s
- [ ] Manual test passes: `cargo test -p everflask-platform --test macos_activity -- --ignored`

## Single instance
- [ ] Launching EverFlask again while it runs doesn't add a second flask

## Uninstall
- [ ] Menu → Uninstall EverFlask → Cancel changes nothing
- [ ] Confirm: app is in the Trash; EverFlask is gone from Settings → Accessibility
- [ ] `~/Library/Preferences/io.github.amcocan.everflask.plist` doesn't exist
- [ ] `defaults read com.apple.universalaccessAuthWarning` has no EverFlask keys and still has every other key
- [ ] `pmset -g assertions` shows no EverFlask assertions
- [ ] `packaging/macos/uninstall.sh` on an already-deleted install removes the same items and prints guidance for the permission
