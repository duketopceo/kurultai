#!/usr/bin/env bash
# Phase 4 tracker closeout — run as a maintainer with issue write access.
# Usage: ./scripts/phase-4-closeout.sh
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-duketopceo/kurultai}"

echo "Closing Phase 4 umbrella on $REPO…"
gh issue close 8 --repo "$REPO" --comment "Phase 4 solo exit shipped: Dayflow+Pond (#62), GitHub FS (#63). Deferred: Composio, plugins (#14), CodeGraph, AppFlowy (#4), OpenRouter batch — see docs/plans/phase-4-complete.md."

echo "Reminder: #21 already closed via #62."
echo "Then close Milestone 4 when clear:"
echo "  gh api -X PATCH repos/$REPO/milestones/4 -f state=closed"

echo "Done. See docs/plans/phase-4-closeout.md"
