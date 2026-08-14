#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MACOS="$ROOT/helper/macos"
DIST="$ROOT/dist/ComputerUseHelper.app"
cd "$MACOS"
swift build -c release
BIN="$(swift build -c release --show-bin-path)/computer-use-helper"
rm -rf "$DIST"
mkdir -p "$DIST/Contents/MacOS"
cp "$BIN" "$DIST/Contents/MacOS/computer-use-helper"
cp "$MACOS/Info.plist" "$DIST/Contents/Info.plist"
chmod +x "$DIST/Contents/MacOS/computer-use-helper"
codesign --force --sign - --identifier dev.computeruse.helper --timestamp=none "$DIST/Contents/MacOS/computer-use-helper" >/dev/null
codesign --force --sign - --identifier dev.computeruse.helper --timestamp=none "$DIST" >/dev/null
echo "built $DIST"
