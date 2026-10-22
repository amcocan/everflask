#!/bin/sh
# Builds the release DMG: universal (Apple Silicon + Intel) EverFlask.app,
# ad-hoc signed with the hardened runtime, plus a SHA-256 checksum.
# Output: target/dist/EverFlask-<version>.dmg and .dmg.sha256
#
# Releases are unsigned by decision (docs/threat-model.md). Unlike
# dev-bundle.sh this never touches the Accessibility permission.
set -eu
cd "$(dirname "$0")/../.."

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
[ -n "$VERSION" ] || { echo "no version in Cargo.toml" >&2; exit 1; }
# Must match LSMinimumSystemVersion in Info.plist; also stops the Intel
# slice claiming support for older systems than the app really has.
export MACOSX_DEPLOYMENT_TARGET=11.0

DIST=target/dist
APP=$DIST/EverFlask.app
DMG=$DIST/EverFlask-$VERSION.dmg
rm -rf "$DIST"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

for target in aarch64-apple-darwin x86_64-apple-darwin; do
    cargo build --locked --release -p everflask --target "$target"
done
lipo -create -output "$APP/Contents/MacOS/everflask" \
    target/aarch64-apple-darwin/release/everflask \
    target/x86_64-apple-darwin/release/everflask

cp packaging/macos/Info.plist "$APP/Contents/"
cp packaging/macos/AppIcon.icns "$APP/Contents/Resources/"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" "$APP/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $VERSION" "$APP/Contents/Info.plist"

# Hardened runtime: MacOS ignores DYLD_* injection (threat model T10).
codesign --force --options runtime --sign - "$APP"

# Fail the build rather than ship something that doesn't match the plan.
codesign --verify --strict "$APP"
codesign -dv "$APP" 2>&1 | grep -q "flags=.*runtime" || { echo "hardened runtime missing" >&2; exit 1; }
archs=$(lipo -archs "$APP/Contents/MacOS/everflask")
[ "$archs" = "x86_64 arm64" ] || { echo "unexpected architectures: $archs" >&2; exit 1; }
for arch in arm64 x86_64; do
    minos=$(otool -arch "$arch" -l "$APP/Contents/MacOS/everflask" | awk '/LC_BUILD_VERSION/{f=1} f&&/minos/{print $2; exit}')
    [ "$minos" = "11.0" ] || { echo "$arch minimum MacOS is $minos, expected 11.0" >&2; exit 1; }
done

# Disk image: the app plus a shortcut to Applications to drag it onto.
STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
hdiutil create -quiet -volname "EverFlask $VERSION" -srcfolder "$STAGE" \
    -fs HFS+ -format UDZO -ov "$DMG"

(cd "$DIST" && shasum -a 256 "$(basename "$DMG")" > "$(basename "$DMG").sha256")
echo "$DMG"
cat "$DMG.sha256"
