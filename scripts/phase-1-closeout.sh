#!/usr/bin/env bash
# Phase 1 tracker closeout — run as a maintainer with issue write access.
# Usage: ./scripts/phase-1-closeout.sh
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-duketopceo/kurultai}"

echo "Closing shipped Phase 1 trackers on $REPO…"
gh issue close 5  --repo "$REPO" --comment "Shipped: CLI index/status/search in #46. Phase 1 complete."
gh issue close 29 --repo "$REPO" --comment "Shipped: environments in #30."
gh issue close 40 --repo "$REPO" --comment "Shipped: docs/upstream-inspiration.md via #41."
gh issue close 42 --repo "$REPO" --comment "Phase 1 CE plan executed; see docs/plans/phase-1-complete.md."
gh issue close 25 --repo "$REPO" --comment "Audience order documented (#26); umbrella remains #27."

echo "Moving non-exit issues off Milestone 1…"
gh issue edit 4  --repo "$REPO" --milestone "Phase 4: Expansion" || \
  gh issue edit 4 --repo "$REPO" --remove-milestone
gh issue edit 23 --repo "$REPO" --remove-milestone
gh issue edit 33 --repo "$REPO" --milestone "Phase 3: Synthesis & Interface" || \
  gh issue edit 33 --repo "$REPO" --remove-milestone

echo "Closing Milestone 1…"
gh api -X PATCH "repos/$REPO/milestones/1" -f state=closed

echo "Done. Verify: gh api repos/$REPO/milestones/1 --jq '{state,open_issues,closed_issues}'"
