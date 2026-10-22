# Threat Model

Living document. Update it whenever a change adds a permission, an input, an output, or a dependency with OS access.

## What EverFlask does
- Holds IOKit power assertions (`PreventUserIdleDisplaySleep`, `PreventUserIdleSystemSleep`) while the flask is full.
- While the flask is full and Accessibility is granted, posts a zero-distance mouse-move event, at most once per 60 s and only after 45 s of real idle time. The user opts in by granting the permission; without it, full means keep-awake only.
- Uninstall: resets its own permissions, deletes its own preferences and folders, and moves its own bundle to the Trash.
- Single instance: a second launch exits immediately.
- No network access, no IPC, no URL scheme, no config files (yet).

## Assets worth protecting
1. The user's **Accessibility grant**, once given. This is the main one: it lets the holder send input to any app.
2. The machine's **auto-lock and sleep** behaviour, which is a security control on managed devices.
3. The user's **trust in the binary** they downloaded.

## Trust boundaries
- User ↔ app: menu clicks only.
- App ↔ MacOS: IOKit power assertions; CoreGraphics event posting (TCC-gated).
- Build ↔ release: crates.io dependencies, GitHub Actions, the release artifact.

## Threats and mitigations

| # | Threat | Mitigation | Status |
|---|---|---|---|
| T1 | Compromised EverFlask inherits the Accessibility grant (malicious dependency, tampered download, dylib injection) | Developer ID signing, notarization, hardened runtime with library validation; minimal dependencies; `cargo audit` + `cargo deny` in CI; signed checksums and GitHub build attestations for releases | Partly: dependency checks in CI; signing comes in the packaging phase |
| T2 | The input capability is abused to send arbitrary input | Only one fixed event exists in the code: a mouse move to the pointer's current location. No text, clicks, keys or parameters. No external trigger (no IPC/URL scheme) | Done |
| T3 | The event stream is used as cover or amplified | Rate-limited to one event per 60 s; `Nudger` never bursts after sleep or a long gap (unit-tested); runs only while full; skipped entirely while the user is active (`needs_nudge`, unit-tested) | Done |
| T4 | Auto-lock is defeated without the user realising | Requires the user to grant Accessibility after a MacOS prompt; README warning placed before first use; tooltip states "Keeping awake and active."; manual lock (⌃⌘Q) still works. **Accepted risk**: since the merge, filling the flask with the permission granted always defeats auto-lock; there's no keep-awake-only mode other than not granting it | Accepted |
| T5 | Revoked permission leaves the UI claiming activity | Every nudge checks `CGPreflightPostEventAccess`; on failure the tooltip switches to the keep-awake-only message | Done |
| T6 | Crash leaves the machine stuck awake | Kernel releases power assertions when the process exits; also released on toggle-off, quit and drop | Done (integration-tested) |
| T7 | UI claims "awake" when the assertion failed | State flips only after the OS call succeeds; half-enabled state is rolled back | Done |
| T8 | Swapped binary keeps the user's grant | TCC binds grants to the code signature, so a replacement loses it. Unsigned/ad-hoc builds need re-granting after each update (a usability cost, not a hole) | Inherent to MacOS |
| T9 | Policy or ethical misuse (faking presence at work) | Disclaimer and warning in README; neutral, explicit labelling; no stealth (visible icon state and tooltip) | Done |
| T10 | Code injected at launch (`DYLD_INSERT_LIBRARIES`) inherits the Accessibility grant | Hardened runtime (`codesign --options runtime`), so MacOS ignores DYLD injection. **Found in testing**: an injected probe library ran in the ad-hoc build; blocked after the fix | Fixed in dev bundle; must carry over to release signing |
| T11 | Two instances double the nudge rate and split state | Single instance via `NSRunningApplication`: the earlier launch wins, PID breaks ties. **Found in testing**: two copies ran; now 3 sequential and 2 simultaneous launches leave exactly one | Fixed |
| T12 | Uninstall deletes something that isn't ours | Every path and key is derived from the compile-time bundle ID or app name; preference keys are filtered by exact pattern (unit-tested); symlinks removed as links, never followed; the app bundle goes to the Trash (recoverable) and only if running from an `.app` with our bundle ID; `tccutil` called by absolute path with fixed arguments and never without a bundle ID (which would reset every app) | Done |
| T14 | Synthetic event jumps the pointer to (0, 0) | **Found in review**: if CoreGraphics failed to create the probe event, the location of a missing event, (0, 0), would have been posted. Now the nudge aborts instead | Fixed |
| T15 | Synthetic input interferes with real use (e.g. a move event mid-drag) | Nudges only after 45 s of real idle time, so never while the user is working | Done |
| T13 | A tampered or vulnerable dependency, or a moved CI action tag | `cargo audit` + `cargo deny` in CI; actions pinned to commit SHAs; CI token read-only; checkout doesn't keep credentials | Done |

## Decision: unsigned releases
EverFlask ships without an Apple Developer ID signature or notarization (decided 2026-10-21). Users can build from source or use the pre-built releases. Consequences, accepted:
- Gatekeeper warns on first launch; users must use **Open Anyway** (README → Installing).
- T1/T8: a downloaded copy's origin can't be verified by signature. Mitigation: SHA-256 checksums and GitHub build attestations on every release; building from source is documented.
- Each update loses the Accessibility grant (the ad-hoc signature is a code hash).
- The hardened runtime (T10) is still applied: ad-hoc signing supports it.

## Known gaps
- Releases are unsigned by decision (above). `build-release.sh` enforces the hardened runtime (T10) and writes checksums; build attestations only start once the repository is public (GitHub limits them on private repos).
- `cargo deny` checks only the Mac targets. The Linux tray stack (GTK) brings in RUSTSEC-2024-0370 (unmaintained) and RUSTSEC-2024-0429 (unsound); deal with them when the Linux port starts.
- The manual activity test must run from a properly bundled, signed `.app` to validate TCC behaviour end to end.
