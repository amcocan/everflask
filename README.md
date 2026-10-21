# Everflask

A tiny menu-bar app that keeps your computer awake. A potion flask next to the clock shows the state:

- **Full (red liquid)**: your Mac won't sleep and the screen stays on.
- **Off (grey liquid)**: normal sleep behaviour.

Double-click the flask to switch. Right-click it for the menu.

Status: early development. macOS first; Windows and Linux later.

> [!WARNING]
> **Simulate activity is off by default. Read this before you turn it on.**
>
> The optional **Simulate activity (keeps chat apps Active)** mode makes your Mac think someone is using it, so apps like Microsoft Teams and Slack don't switch you to "Away". To do that it needs the macOS **Accessibility** permission, and it has side effects:
>
> - **Your screen won't lock on its own.** Automatic lock ("require password after screen saver or display is turned off") never triggers while the mode runs. That includes lock policies set by your employer's IT. Lock manually with **Control-Command-Q** when you step away.
> - **Accessibility is a powerful permission.** It lets an app send input to other apps. Everflask only ever sends one fixed input (see below), but only grant it to a copy you trust, such as an official signed release from this repository.
> - **It may break workplace rules.** Faking presence can breach your organisation's acceptable-use or monitoring policies. Check before you use it on a work device.

## Simulate activity: what it actually does

While the flask is full **and** the mode is turned on, Everflask posts one mouse-move event to the pointer's current position **every 60 seconds**. The pointer doesn't move. macOS counts it as input, which resets the idle timer that chat apps watch.

That event is the only input Everflask ever sends. It never types, clicks, or sends anything configurable, and nothing outside the app can trigger it. The nudges stop the moment you empty the flask, turn the mode off, or quit. If you remove the Accessibility permission, the mode switches itself off.

### Why it needs the Accessibility permission

macOS doesn't let apps create mouse or keyboard input unless you've approved them. That's what stops malware from quietly typing or clicking for you. Even a mouse move that goes nowhere counts as input, so Everflask has to ask.

Apps can't reset the idle timer any other way. Everflask's normal keep-awake mode, which needs no permission, stops your Mac sleeping, but macOS doesn't count it as activity. Apple's "user is active" API (`IOPMAssertionDeclareUserActivity`) doesn't reset the idle timers chat apps read either. We tested both. Only real input events do, so the Accessibility permission is the minimum this feature needs.

Everflask uses the permission for that one mouse-move event and nothing else. It doesn't read your screen, other windows or your keystrokes.

### Turning it on

1. Right-click the flask and choose **Simulate activity (keeps chat apps Active)**.
2. When macOS asks, open **System Settings → Privacy & Security → Accessibility** and switch **Everflask** on.
3. **Quit and relaunch Everflask.** macOS only applies the new permission when the app next starts, so until then the item stays unchecked.
4. Right-click the flask and choose **Simulate activity** again. A checkmark shows it's on.

To turn the permission off later, switch Everflask off in the same Settings pane. The mode switches itself off within 60 seconds.

## Disclaimer

Everflask is provided "as is", without warranty of any kind (see [LICENSE](LICENSE)). You're responsible for how you use it. In particular, Simulate activity changes how your computer and other apps judge whether you're present. The authors aren't responsible for any consequences of using it, including breaches of employer, school or other policies, missed security protections such as automatic screen lock, or decisions others make based on your displayed status. Don't use it to misrepresent your availability where that would be dishonest or against the rules that apply to you.

## Building
    cargo build --release
    cargo test

The simulate-activity check is a manual test because it needs the Accessibility permission and an idle machine:

    cargo test -p everflask-platform --test macos_activity -- --ignored

## Layout
- `crates/everflask-core`: state, double-click detection and nudge scheduling (no OS code)
- `crates/everflask-platform`: per-OS keep-awake and activity backends
- `crates/everflask-app`: menu-bar icon and event loop
- `assets/icons`: icon sources (SVG) and the design file
- `packaging`: per-OS bundle files
- `docs`: threat model and QA checklist

## Security
See [SECURITY.md](SECURITY.md) to report a vulnerability, and [docs/threat-model.md](docs/threat-model.md) for the design rationale.
