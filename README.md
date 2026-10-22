# EverFlask

A tiny menu-bar app that keeps your computer awake and keeps chat apps from showing you as Away. A potion flask next to the clock shows the state:

- **Full (red liquid)**: your Mac won't sleep, the screen stays on, and apps like Microsoft Teams and Slack keep showing you as Active.
- **Off (grey liquid)**: normal behaviour.

Double-click the flask to switch. Right-click it for the menu.

Status: early development. MacOS first; Windows and Linux later.

> [!WARNING]
> **Read this before you fill the flask.**
>
> To keep chat apps Active, EverFlask makes your Mac think someone is using it. That needs the MacOS **Accessibility** permission, which EverFlask asks for the first time you fill the flask. It has side effects:
>
> - **Your screen won't lock on its own.** While the flask is full, automatic lock ("require password after screen saver or display is turned off") never triggers. That includes lock policies set by your employer's IT. Lock manually with **Control-Command-Q** when you step away.
> - **Accessibility is a powerful permission.** It lets an app send input to other apps. EverFlask only ever sends one fixed input (see below), but only grant it to a copy you trust, such as an official signed release from this repository.
> - **It may break workplace rules.** Faking presence can breach your organisation's acceptable-use or monitoring policies. Check before you use it on a work device.
>
> If you'd rather not grant the permission, click **Don't Allow**. EverFlask still keeps your Mac awake, but chat apps may show you as Away.

## How it keeps chat apps Active

Chat apps decide you're Away when the Mac's idle timer (time since your last keyboard or mouse input) passes their limit, usually 5 minutes. While the flask is full, EverFlask checks the idle timer once a minute. **Only if you've been idle for 45 seconds or more**, it posts one mouse-move event to the pointer's current position. The pointer doesn't move. MacOS counts it as input, which resets the idle timer.

- **That event is the only input EverFlask ever sends.** It never types, clicks, or sends anything configurable, and nothing outside the app can trigger it.
- **While you're working, it sends nothing.** Your own input already keeps the timer low.
- **The idle timer never gets past about 105 seconds**, far below the usual 5-minute limit.
- **Nudges stop the moment you empty the flask or quit.**

### Why it needs the Accessibility permission

MacOS doesn't let apps create mouse or keyboard input unless you've approved them. That's what stops malware from quietly typing or clicking for you. Even a mouse move that goes nowhere counts as input, so EverFlask has to ask.

Apps can't reset the idle timer any other way. Preventing sleep, which needs no permission, keeps your Mac awake, but MacOS doesn't count it as activity. Apple's "user is active" API (`IOPMAssertionDeclareUserActivity`) doesn't reset the idle timers chat apps read either. We tested both. Only real input events do, so the Accessibility permission is the minimum this feature needs.

EverFlask uses the permission for that one mouse-move event and nothing else. It doesn't read your screen, other windows or your keystrokes.

### Granting the permission

1. Fill the flask. The first time, MacOS asks whether EverFlask may control your computer.
2. Open **System Settings → Privacy & Security → Accessibility** and switch **EverFlask** on.
3. **Quit and relaunch EverFlask.** MacOS only applies the new permission when the app next starts.
4. Fill the flask again. Hover over it to check: the tooltip says **keeping awake and active**. If it asks you to allow Accessibility instead, the permission isn't active yet.

To take the permission back, switch EverFlask off in the same Settings pane. EverFlask goes back to keep-awake only.

## Uninstalling

Dragging an app to the Trash leaves its settings and permissions behind, because MacOS runs no code when you delete an app. To remove EverFlask completely:

**Right-click the flask → Uninstall EverFlask…** and confirm. EverFlask then:

1. Removes its Accessibility permission.
2. Deletes its preferences and its entries in MacOS's permission-warning records.
3. Deletes any caches or saved state.
4. Moves itself to the Trash, so you can still recover it.

**Already deleted EverFlask?** Run the uninstall script. It removes the same items:

    sh packaging/macos/uninstall.sh

If EverFlask was deleted before the script ran, MacOS may no longer recognise it, so the script can't reset the permission. In that case, open **System Settings → Privacy & Security → Accessibility**, select EverFlask and click **−**.

What EverFlask can leave behind, for reference:

| Item | Location |
|---|---|
| Preferences (e.g. the icon's menu-bar position) | `~/Library/Preferences/io.github.amcocan.everflask.plist` |
| Accessibility permission | System Settings → Privacy & Security → Accessibility |
| "Permission warning shown" records | Keys mentioning EverFlask in the `com.apple.universalaccessAuthWarning` preferences |
| Caches, saved state (if MacOS created them) | `~/Library/Caches/`, `~/Library/Saved Application State/`, `~/Library/HTTPStorages/`, `~/Library/Application Support/`, each under `io.github.amcocan.everflask` |

## Disclaimer

EverFlask is provided "as is", without warranty of any kind (see [LICENSE](LICENSE)). You're responsible for how you use it. In particular, keeping chat apps Active changes how your computer and other apps judge whether you're present. The authors aren't responsible for any consequences of using it, including breaches of employer, school or other policies, missed security protections such as automatic screen lock, or decisions others make based on your displayed status. Don't use it to misrepresent your availability where that would be dishonest or against the rules that apply to you.

## Building
    cargo build --release
    cargo test

The keep-chat-apps-Active check is a manual test because it needs the Accessibility permission and an idle machine:

    cargo test -p everflask-platform --test macos_activity -- --ignored

## Developing

**Every rebuild of the dev bundle loses the Accessibility grant.** Dev builds are ad-hoc signed, so MacOS identifies them by a hash of their code, and each rebuild looks like a new app. Settings may still show EverFlask switched on, but the grant no longer applies, and MacOS won't prompt again. To re-grant after a rebuild:

    for s in Accessibility PostEvent; do tccutil reset $s io.github.amcocan.everflask; done

Then fill the flask, allow the prompt, and relaunch EverFlask. Signed releases don't have this problem.

To see what EverFlask is doing, launch it with the diagnostic log:

    open --env EVERFLASK_DEBUG=1 --stderr /tmp/everflask.log target/debug/EverFlask.app

The log records whether the permission is active, and each minute whether a nudge was needed and sent.

## Layout
- `crates/everflask-core`: state, double-click detection and nudge scheduling (no OS code)
- `crates/everflask-platform`: per-OS keep-awake and activity backends
- `crates/everflask-app`: menu-bar icon and event loop
- `assets/icons`: menu-bar icon sources (SVG) and the design file
- `assets/app-icon`: app icon source; run `scripts/make-app-icon.sh` after editing it
- `packaging`: per-OS bundle files
- `docs`: threat model and QA checklist

## Security
See [SECURITY.md](SECURITY.md) to report a vulnerability, and [docs/threat-model.md](docs/threat-model.md) for the design rationale.
