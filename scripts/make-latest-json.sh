#!/usr/bin/env bash
# Generate the Tauri updater manifest (latest.json) for release artifacts.
# Each installer is minisign-signed in place (creates <file>.sig) and the
# signature is embedded into latest.json under the matching platform key.
#
# Usage:
#   bash scripts/make-latest-json.sh \
#     --key <signing-key-file> \
#     --version <app-version> \
#     --url-prefix <base-url> \
#     --out <output-json> \
#     [--notes <release-notes>] \
#     <installer> [<installer>...]
#
# Platform keys are derived from the installer filename:
#   *aarch64*.dmg -> darwin-aarch64
#   *.dmg         -> darwin-x86_64
#   *x64*setup.exe / *.exe -> windows-x86_64
set -euo pipefail
cd "$(dirname "$0")/.."

KEY=""
VERSION=""
URL_PREFIX=""
OUT=""
NOTES="媒体批处理工具发布版"
INSTALLERS=()

while [ $# -gt 0 ]; do
  case "$1" in
    --key) KEY="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    --url-prefix) URL_PREFIX="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    --notes) NOTES="$2"; shift 2 ;;
    -*) echo "unknown option: $1" >&2; exit 1 ;;
    *) INSTALLERS+=("$1"); shift ;;
  esac
done

if [ -z "$KEY" ] || [ -z "$VERSION" ] || [ -z "$URL_PREFIX" ] || [ -z "$OUT" ]; then
  echo "usage: bash scripts/make-latest-json.sh --key <key> --version <v> --url-prefix <url> --out <json> [--notes <n>] <installer>..." >&2
  exit 1
fi
if [ "${#INSTALLERS[@]}" -eq 0 ]; then
  echo "no installers given" >&2
  exit 1
fi

PLATFORMS="{}"
for f in "${INSTALLERS[@]}"; do
  [ -f "$f" ] || { echo "missing installer: $f" >&2; exit 1; }

  pnpm tauri signer sign -f "$KEY" -p "" "$f" >/dev/null
  SIG="$(cat "$f.sig")"
  URL="$URL_PREFIX/$(basename "$f")"

  case "$(basename "$f")" in
    *aarch64*.dmg) PK="darwin-aarch64" ;;
    *.dmg) PK="darwin-x86_64" ;;
    *x64*setup.exe|*.exe) PK="windows-x86_64" ;;
    *) echo "cannot map $(basename "$f") to an updater platform" >&2; exit 1 ;;
  esac

  PLATFORMS="$(python3 - "$PLATFORMS" "$PK" "$SIG" "$URL" <<'EOF'
import json, sys
obj = json.loads(sys.argv[1])
obj[sys.argv[2]] = {"signature": sys.argv[3], "url": sys.argv[4]}
print(json.dumps(obj, ensure_ascii=False))
EOF
)"
done

PUB_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
python3 - "$OUT" "$VERSION" "$NOTES" "$PUB_DATE" "$PLATFORMS" <<'EOF'
import json, sys
out, version, notes, pub_date, platforms = sys.argv[1:6]
payload = {
    "version": version,
    "notes": notes,
    "pub_date": pub_date,
    "platforms": json.loads(platforms),
}
with open(out, "w", encoding="utf-8") as fh:
    json.dump(payload, fh, ensure_ascii=False, indent=2)
    fh.write("\n")
print(f"latest.json written to {out}")
EOF
