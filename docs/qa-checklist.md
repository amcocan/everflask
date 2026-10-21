# QA Checklist (macOS)

- [ ] Toggle on: `pmset -g assertions` shows the Everflask assertion
- [ ] Toggle off: assertion gone
- [ ] Icon matches state in light and dark menu bar
- [ ] Double-click toggles; menu works
- [ ] Single instance enforced
- [ ] Quit releases the assertion

## Simulate activity
- [ ] Off by default after launch
- [ ] Turning it on without permission shows the macOS prompt and the item stays unchecked
- [ ] After granting, the item still stays unchecked until Everflask is relaunched (macOS applies the grant at next launch)
- [ ] With permission: `ioreg -c IOHIDSystem | grep HIDIdleTime` drops to ~0 at most every 60 s while idle
- [ ] Pointer never moves
- [ ] Teams stays Available past its 5 min idle timeout
- [ ] Emptying the flask, unchecking the item or quitting stops nudges (idle time climbs again)
- [ ] Revoking Accessibility while on switches the item off within 60 s
- [ ] Manual test passes: `cargo test -p everflask-platform --test macos_activity -- --ignored`
