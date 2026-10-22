#!/bin/sh
# Regenerates packaging/macos/AppIcon.icns from assets/app-icon/AppIcon.svg.
# Run after editing the SVG and commit the result. Needs `resvg`
# (cargo install --locked resvg) and MacOS's iconutil.
set -eu
cd "$(dirname "$0")/.."
SRC=assets/app-icon/AppIcon.svg
OUT=packaging/macos/AppIcon.icns
SET=$(mktemp -d)/AppIcon.iconset
mkdir -p "$SET"
for size in 16 32 128 256 512; do
    resvg -w $size -h $size "$SRC" "$SET/icon_${size}x${size}.png"
    resvg -w $((size * 2)) -h $((size * 2)) "$SRC" "$SET/icon_${size}x${size}@2x.png"
done
iconutil -c icns -o "$OUT" "$SET"
rm -rf "$(dirname "$SET")"
echo "$OUT"
