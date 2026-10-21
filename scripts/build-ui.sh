#!/usr/bin/env bash
# Rebuild website/ → ui/ for rust-embed (daemon GET /ui/).
# NOTE: run under node 22 (see .nvmrc) — hash-named ui-next-* assets differ
# across node majors, and the CI embedded-asset check builds with node 22.
# NOTE: the bundle embeds the website/ TREE hash of HEAD (vite.config.ts).
# Any change under website/ — even INDEX.md/README — bumps the stamp, so
# rebuild + commit ui/ as the last change on any PR touching website/.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"

node_major="$(node --version | sed -E 's/^v?([0-9]+).*/\1/')"
if [[ "$node_major" != "22" && "${KURULTAI_UI_ANY_NODE:-0}" != "1" ]]; then
  echo "error: build-ui.sh must run under node 22 (got $(node --version))." >&2
  echo "       Hash-named ui-next-* assets differ across node majors and the CI" >&2
  echo "       embedded-asset check builds with node 22 (.nvmrc)." >&2
  echo "       fix: mise x node@22 -- bash scripts/build-ui.sh   (or: nvm use)" >&2
  exit 1
fi

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
# Rolldown chunk output is nondeterministic under parallel codegen — hash-named
# assets flip between runs, breaking the CI embedded-asset check. Single-thread.
export RAYON_NUM_THREADS=1
if [[ -f package-lock.json ]]; then
  npm ci
else
  npm install
fi
npm run build

echo "Brain UI written to $root/ui/ (embedded on next cargo build)."
