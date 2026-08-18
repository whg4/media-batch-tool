#!/usr/bin/env bash
# Generate the Tauri updater manifest (latest.json) for a release.
#
# Signing key: scripts/updater.key (gitignored) + scripts/updater.key.pub (committed).
# CI stores the key content in the GitHub secret TAURI_SIGNING_PRIVATE_KEY
# (the updater.key file content; use -f with TAURI_SIGNING_PRIVATE_KEY_PATH in CI).
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

# 1. sign the artifact
cargo tauri signer sign -f scripts/updater.key -p "" "$DMG"

# 2. extract the public signature (base64 of the minisign SignatureBox)
SIGNATURE="$(cat "$DMG.sig")"

# 3. write latest.json (update the URL to your GitHub Release asset URL)
PUB_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
cat > src-tauri/target/release/bundle/dmg/latest.json <<JSON
{
  "version": "$APP_VERSION",
  "notes": "媒体批处理工具发布版",
  "pub_date": "$PUB_DATE",
  "platforms": {
    "darwin-aarch64": {
      "signature": "$SIGNATURE",
      "url": "https://github.com/whg4/media-batch-tool/releases/latest/download/MediaBatchTool_${APP_VERSION}_aarch64.dmg"
    }
  }
}
JSON

echo "latest.json written to src-tauri/target/release/bundle/dmg/latest.json"
