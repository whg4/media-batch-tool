#!/usr/bin/env bash
# Download static ffmpeg/ffprobe builds for Tauri sidecar bundling.
# Run from the repo root before `tauri build`.
#
# Sources:
#   macOS  : https://evermeet.cx/ffmpeg (static, GPL)
#   Windows: https://www.gyan.dev/ffmpeg/builds/ (static, GPL)
# For LGPL builds replace the URL with your preferred static build.
set -euo pipefail

cd "$(dirname "$0")/../src-tauri/binaries"
mkdir -p .staging

case "$(uname -s)" in
  Darwin)
    echo "Fetching macOS static ffmpeg..."
    curl -L -o .staging/ffmpeg.zip https://evermeet.cx/ffmpeg/getrelease/zip
    curl -L -o .staging/ffprobe.zip https://evermeet.cx/ffmpeg/getrelease/ffprobe/zip
    unzip -o -j .staging/ffmpeg.zip 'ffmpeg' -d .staging
    unzip -o -j .staging/ffprobe.zip 'ffprobe' -d .staging
    mv .staging/ffmpeg ffmpeg
    mv .staging/ffprobe ffprobe
    chmod +x ffmpeg ffprobe
    ;;
  MINGW*|MSYS*|CYGWIN*)
    echo "Fetching Windows static ffmpeg..."
    curl -L -o .staging/ffmpeg.zip https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip
    unzip -o .staging/ffmpeg.zip -d .staging
    find .staging -name 'ffmpeg.exe' -exec cp {} ffmpeg.exe \;
    find .staging -name 'ffprobe.exe' -exec cp {} ffprobe.exe \;
    ;;
  *)
    echo "Unsupported host OS" >&2
    exit 1
    ;;
esac

rm -rf .staging
echo "Sidecar binaries ready in src-tauri/binaries/"
