#!/usr/bin/env bash
# Generate the Tauri updater manifest (latest.json) for a macOS release.
# Thin wrapper around scripts/make-latest-json.sh — the canonical manifest
# generator that the Release CI workflow also uses.
#
# Usage:
#   1) build the release bundles (bash scripts/build-release.sh)
#   2) run this script from the repo root
set -euo pipefail
cd "$(dirname "$0")/.."

APP_VERSION="$(python3 -c "import json; print(json.load(open('src-tauri/tauri.conf.json'))['version'])")"
DMG="src-tauri/target/release/bundle/dmg/MediaBatchTool_${APP_VERSION}_aarch64.dmg"

if [ ! -f "$DMG" ]; then
  echo "Missing $DMG — run scripts/build-release.sh first" >&2
  exit 1
fi

bash scripts/make-latest-json.sh \
  --key scripts/updater.key \
  --version "$APP_VERSION" \
  --url-prefix "https://github.com/whg4/media-batch-tool/releases/latest/download" \
  --out src-tauri/target/release/bundle/dmg/latest.json \
  "$DMG"
