#!/usr/bin/env bash
# Build a Codify.app bundle and wrap it in a .dmg for macOS.
# Requires: cargo, cargo-bundle (cargo install cargo-bundle), hdiutil (built into macOS).
set -euo pipefail

cd "$(dirname "$0")/../.."

BIN_NAME=codify
BUNDLE_APP="target/release/bundle/osx/$BIN_NAME.app"
DMG="dist/codify-$BIN_NAME.dmg"

# No logo: just the name in Info.plist.
if [ ! -f Cargo.toml ]; then
    echo "run from repo root" >&2
    exit 1
fi

echo "==> building release binary"
cargo build --release

echo "==> bundling .app"
cargo bundle --release

if [ ! -d "$BUNDLE_APP" ]; then
    echo "cargo-bundle not available; falling back to manual .app layout"
    APP_DIR="dist/Codify.app/Contents/MacOS"
    mkdir -p "$APP_DIR"
    cp target/release/$BIN_NAME "$APP_DIR/$BIN_NAME"
    cat > dist/Codify.app/Contents/Info.plist <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Codify</string>
  <key>CFBundleDisplayName</key><string>Codify</string>
  <key>CFBundleIdentifier</key><string>org.codify.Codify</string>
  <key>CFBundleExecutable</key><string>codify</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
</dict>
</plist>
PLIST
    BUNDLE_APP="dist/Codify.app"
fi

echo "==> creating dmg"
mkdir -p dist
hdiutil create -volname Codify -srcfolder "$BUNDLE_APP" -ov -format UDZO "$DMG"
echo "==> done: $DMG"