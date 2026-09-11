#!/usr/bin/env bash
# Rebuild website/ → ui/ for rust-embed (daemon GET /ui/).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"

cd "$root"

# Drop previous hashed bundles and legacy landing-page cruft so stale files
# cannot be served from /ui/ after a new build.
rm -f \
  "$root/ui/assets/"*.js \
  "$root/ui/assets/"*.css \
  "$root/ui/assets/"*.glb \
  "$root/ui/index.html" \
  "$root/ui/index.css" \
  "$root/ui/index.js" \
  "$root/ui/kurultai_logo.jpg" \
  "$root/ui/neural_tech_banner.jpg"

cd "$root/website"
if [[ -f package-lock.json ]]; then
  npm ci
else
  npm install
fi
npm run build

echo "Brain UI written to $root/ui/ (embedded on next cargo build)."
