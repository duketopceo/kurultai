#!/usr/bin/env bash
# Phase 2 tracker closeout — run as a maintainer with issue/PR write access.
# Usage: ./scripts/phase-2-closeout.sh
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-duketopceo/kurultai}"

echo "Closing Phase 2 trackers on $REPO…"
gh issue close 6 --repo "$REPO" --comment "Phase 2 search shipped in #51. Distillation remains #12."

echo "Closing obsolete plan PRs…"
gh pr close 50 --repo "$REPO" --comment "Obsolete: search plan landed via #51." || true
gh pr close 52 --repo "$REPO" --comment "Obsolete: testing plan landed via #53 / Phase 2 closeout." || true
# Close #53 only if its commits are already on main via closeout merge
gh pr close 53 --repo "$REPO" --comment "Superseded: testing content merged into Phase 2 closeout PR." || true

echo "Reminder: tick #23 Phase 2 checklist in the issue body (cannot automate reliably)."
echo "Then close Milestone 2 when clear:"
echo "  gh api -X PATCH repos/$REPO/milestones/2 -f state=closed"

echo "Done. See docs/plans/phase-2-closeout.md"
