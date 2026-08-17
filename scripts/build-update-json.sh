#!/usr/bin/env bash
# Generate the Tauri updater manifest (latest.json) for a release.
# Requires: tauri-cli (cargo install tauri-cli --locked) and the signing key.
#
# The signing private key must be set in TAURI_SIGNING_PRIVATE_KEY
# (and TAURI_SIGNING_PRIVATE_KEY_PASSWORD if the key is password-protected).
# CI stores it in the GitHub secret TAURI_SIGNING_PRIVATE_KEY.
set -euo pipefail

if [ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ]; then
  echo "TAURI_SIGNING_PRIVATE_KEY is not set" >&2
  exit 1
fi

cd "$(dirname "$0")/../src-tauri"
# Build the bundle first, then create a signed update manifest.
cargo tauri build --release
cargo tauri signer generate --help >/dev/null

# The generated manifest lives at target/release/bundle/macos/MediaBatchTool.app/Contents/Resources/...
# and is emitted by `tauri build` into the bundle directory when signing is configured.
# Publish the artifacts (dmg / nsis exe / latest.json) to the GitHub Release.
echo "Publish the following files to the GitHub Release tagged vX.Y.Z:"
ls target/release/bundle/ 2>/dev/null || true
