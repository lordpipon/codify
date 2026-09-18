#!/usr/bin/env bash
# Build a Codify.app bundle and wrap it in a .dmg for macOS.
# Requires: cargo, hdiutil (built into macOS). No logos, just the name.
set -euo pipefail

cd "$(dirname "$0")/../.."

BIN_NAME=codify
VERSION="${CODIFY_VERSION:-0.0.0}"
APP_DIR="dist/Codify.app"
DMG="dist/Codify-$VERSION.dmg"

if [ ! -f Cargo.toml ]; then
    echo "run from repo root" >&2
    exit 1
fi

echo "==> building release binary"
cargo build --release

echo "==> assembling .app bundle"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"
cp "target/release/$BIN_NAME" "$APP_DIR/Contents/MacOS/$BIN_NAME"
cp "packaging/macos/codify.icns" "$APP_DIR/Contents/Resources/codify.icns"

cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Codify</string>
  <key>CFBundleDisplayName</key><string>Codify</string>
  <key>CFBundleIdentifier</key><string>org.codify.Codify</string>
  <key>CFBundleExecutable</key><string>codify</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>CFBundleIconFile</key><string>codify</string>
  <key>CFBundleIconName</key><string>codify</string>
</dict>
</plist>
PLIST

echo "==> creating dmg"
mkdir -p dist
hdiutil create -volname Codify -srcfolder "$APP_DIR" -ov -format UDZO "$DMG"
echo "==> done: $DMG"
