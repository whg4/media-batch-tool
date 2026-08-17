#!/usr/bin/env bash
# Build a release bundle (macOS .app + .dmg).
# Usage: bash scripts/build-release.sh
set -euo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.cargo/bin:$PATH"

# 1. Build the .app (frontend + rust + bundle)
pnpm tauri build --bundles app

# 2. Build the .dmg with the Finder-beautify step skipped
#    (the AppleScript step needs an interactive GUI session; CI/headless
#     environments hit AppleEvent timeouts, so we skip it).
DMG_DIR="src-tauri/target/release/bundle/dmg"
APP="$(pwd)/src-tauri/target/release/bundle/macos/MediaBatchTool.app"
DMG="$(pwd)/$DMG_DIR/MediaBatchTool_0.1.0_aarch64.dmg"
if [ -f "$DMG" ]; then unlink "$DMG"; fi
bash "$DMG_DIR/bundle_dmg.sh" --skip-jenkins "$DMG" "$APP"

echo ""
echo "Done:"
echo "  $APP"
echo "  $DMG"
