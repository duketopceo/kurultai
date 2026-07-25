#!/usr/bin/env bash
# Phase 4 tracker closeout — run as a maintainer with issue write access.
# Run from a checkout after the closeout PR is on main.
# Usage: ./scripts/phase-4-closeout.sh
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-duketopceo/kurultai}"
CLOSE_COMMENT='Phase 4 solo exit shipped: Dayflow+Pond (#62), GitHub FS (#63). Deferred (not exit): Composio meta-connector; plugins (#14); CodeGraph/tree-sitter; AppFlowy (#4); OpenRouter batch/fallback embed; TechTracker composite; #23 Phase 4 coverage≥60%/cargo-deny — see docs/plans/phase-4-complete.md.'

echo "Preflight: PRs #62/#63 merged into main on $REPO…"
for pr in 62 63; do
  meta="$(gh pr view "$pr" --repo "$REPO" --json state,baseRefName,mergedAt)"
  state="$(echo "$meta" | jq -r .state)"
  base="$(echo "$meta" | jq -r .baseRefName)"
  merged="$(echo "$meta" | jq -r .mergedAt)"
  if [[ "$state" != "MERGED" || "$base" != "main" || "$merged" == "null" ]]; then
    echo "Abort: PR #$pr state=$state base=$base mergedAt=$merged (expected MERGED into main)" >&2
    exit 1
  fi
done

echo "Preflight: closeout docs present on origin/main…"
git fetch origin main --quiet
for path in \
  docs/plans/phase-4-complete.md \
  docs/plans/phase-4-closeout.md \
  scripts/phase-4-closeout.sh \
  README.md
do
  if ! git cat-file -e "origin/main:$path" 2>/dev/null; then
    echo "Abort: $path missing on origin/main (merge the Phase 4 closeout PR first)." >&2
    exit 1
  fi
done
if ! git show "origin/main:README.md" | grep -q 'phase-4-complete'; then
  echo "Abort: README.md on origin/main does not link phase-4-complete." >&2
  exit 1
fi

echo "Closing Phase 4 umbrella on $REPO…"
gh issue close 8 --repo "$REPO" --comment "$CLOSE_COMMENT"

echo "Reminder: #21 already closed via #62."
echo "Then close Milestone 4 when clear:"
echo "  gh api -X PATCH repos/$REPO/milestones/4 -f state=closed"

echo "Done. See docs/plans/phase-4-closeout.md"
