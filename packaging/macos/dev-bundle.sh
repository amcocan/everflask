#!/bin/sh
# Builds target/debug/EverFlask.app, ad-hoc signed, so MacOS attributes
# permission prompts (Accessibility) to EverFlask itself rather than to the
# terminal it was launched from. Dev use only; releases use cargo-packager.
set -eu
cd "$(dirname "$0")/../.."
cargo build -p everflask
APP=target/debug/EverFlask.app
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
cp packaging/macos/Info.plist "$APP/Contents/"
cp target/debug/everflask "$APP/Contents/MacOS/"
# Hardened runtime: MacOS ignores DYLD_* injection, so injected code
# can never inherit the Accessibility grant.
codesign --force --options runtime --sign - "$APP"
echo "$APP"
