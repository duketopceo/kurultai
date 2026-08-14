#!/usr/bin/env bash
# Rebuild website/ → ui/ for rust-embed (daemon GET /ui/).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
# Drop previous hashed bundles so a stale brain-*.js cannot outlive brain.html.
rm -f "$root/ui/assets/"*.js "$root/ui/assets/"*.css
cd "$root/website"
if [[ -f package-lock.json ]]; then
  npm ci
else
  npm install
fi
npm run build
echo "Brain UI written to $root/ui/ (embedded on next cargo build)."
