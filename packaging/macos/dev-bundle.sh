#!/bin/sh
# Builds target/debug/EverFlask.app, ad-hoc signed, so MacOS attributes
# permission prompts (Accessibility) to EverFlask itself rather than to the
# terminal it was launched from. Dev use only; releases use cargo-packager.
set -eu
cd "$(dirname "$0")/../.."
cargo build -p everflask
APP=target/debug/EverFlask.app
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp packaging/macos/Info.plist "$APP/Contents/"
cp packaging/macos/AppIcon.icns "$APP/Contents/Resources/"
cp target/debug/everflask "$APP/Contents/MacOS/"
# Hardened runtime: MacOS ignores DYLD_* injection, so injected code
# can never inherit the Accessibility grant.
codesign --force --options runtime --sign - "$APP"

# Ad-hoc signing identifies the app by code hash, so the previous build's
# Accessibility grant no longer applies; clear it so Settings doesn't show a
# stale "on" switch. Only EverFlask's entries are touched.
pkill -x everflask 2>/dev/null || true
for svc in Accessibility PostEvent; do
    /usr/bin/tccutil reset "$svc" io.github.amcocan.everflask >/dev/null 2>&1 || true
done
echo "Cleared EverFlask's Accessibility grant (rebuilds invalidate it); allow it again after launching." >&2
echo "$APP"
