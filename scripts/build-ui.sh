#!/usr/bin/env bash
# Rebuild website/ → ui/ for rust-embed (daemon GET /ui/).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root/website"
if [[ -f package-lock.json ]]; then
  npm ci
else
  npm install
fi
npm run build
echo "Brain UI written to $root/ui/ (embedded on next cargo build)."
