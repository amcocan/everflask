# QA Checklist (MacOS)

- [ ] Toggle on: `pmset -g assertions` shows the EverFlask assertion
- [ ] Toggle off: assertion gone
- [ ] Icon matches state in light and dark menu bar
- [ ] Double-click toggles; menu works
- [ ] Single instance enforced
- [ ] Quit releases the assertion

## Keeping chat apps Active (merged into the toggle)
- [ ] First fill shows the MacOS Accessibility prompt; "Don't Allow" leaves keep-awake working, and the tooltip says to allow Accessibility
- [ ] After granting, the tooltip still asks for the permission until EverFlask is relaunched (MacOS applies the grant at next launch)
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
- [ ] Menu → Uninstall EverFlask… → Cancel changes nothing
- [ ] Confirm: app is in the Trash; EverFlask is gone from Settings → Accessibility
- [ ] `~/Library/Preferences/io.github.amcocan.everflask.plist` doesn't exist
- [ ] `defaults read com.apple.universalaccessAuthWarning` has no EverFlask keys and still has every other key
- [ ] `pmset -g assertions` shows no EverFlask assertions
- [ ] `packaging/macos/uninstall.sh` on an already-deleted install removes the same items and prints guidance for the permission
