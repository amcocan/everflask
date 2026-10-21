# Threat Model

Living document. Update it whenever a change adds a permission, an input, an output, or a dependency with OS access.

## What Everflask does
- Holds IOKit power assertions (`PreventUserIdleDisplaySleep`, `PreventUserIdleSystemSleep`) while the flask is full.
- Optional, off by default: **Simulate activity** posts a zero-distance mouse-move event every 60 s while the flask is full.
- No network access, no IPC, no URL scheme, no config files (yet).

## Assets worth protecting
1. The user's **Accessibility grant**, once given. This is the main one: it lets the holder send input to any app.
2. The machine's **auto-lock and sleep** behaviour, which is a security control on managed devices.
3. The user's **trust in the binary** they downloaded.

## Trust boundaries
- User ↔ app: menu clicks only.
- App ↔ macOS: IOKit power assertions; CoreGraphics event posting (TCC-gated).
- Build ↔ release: crates.io dependencies, GitHub Actions, the release artifact.

## Threats and mitigations

| # | Threat | Mitigation | Status |
|---|---|---|---|
| T1 | Compromised Everflask inherits the Accessibility grant (malicious dependency, tampered download, dylib injection) | Developer ID signing, notarization, hardened runtime with library validation; minimal dependencies; `cargo audit` + `cargo deny` in CI; signed checksums and GitHub build attestations for releases | Partly: dependency checks are planned for CI; signing comes in the packaging phase |
| T2 | The input capability is abused to send arbitrary input | Only one fixed event exists in the code: a mouse move to the pointer's current location. No text, clicks, keys or parameters. No external trigger (no IPC/URL scheme) | Done |
| T3 | The event stream is used as cover or amplified | Rate-limited to one event per 60 s; `Nudger` never bursts after sleep or a long gap (unit-tested); runs only while full **and** opted in | Done |
| T4 | Auto-lock is defeated without the user realising | Opt-in only, off by default, explicit menu label, README warning; manual lock (⌃⌘Q) still works | Done |
| T5 | Revoked permission leaves the UI claiming the mode is active | Every nudge checks `CGPreflightPostEventAccess`; on failure the mode switches off and unchecks itself | Done |
| T6 | Crash leaves the machine stuck awake | Kernel releases power assertions when the process exits; also released on toggle-off, quit and drop | Done (integration-tested) |
| T7 | UI claims "awake" when the assertion failed | State flips only after the OS call succeeds; half-enabled state is rolled back | Done |
| T8 | Swapped binary keeps the user's grant | TCC binds grants to the code signature, so a replacement loses it. Unsigned/ad-hoc builds need re-granting after each update (a usability cost, not a hole) | Inherent to macOS |
| T9 | Policy or ethical misuse (faking presence at work) | Disclaimer and warning in README; neutral, explicit labelling; no stealth (visible checkmark and tooltip) | Done |

## Known gaps
- Releases aren't signed or notarized yet (T1, T8).
- No CI step yet for `cargo audit` and `cargo deny`.
- The manual activity test must run from a properly bundled, signed `.app` to validate TCC behaviour end to end.
