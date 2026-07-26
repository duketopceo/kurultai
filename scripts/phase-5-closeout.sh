#!/usr/bin/env bash
# Phase 5 tracker closeout — run as a maintainer with issue/milestone write access.
# Run after the closeout PR is on main of the canonical repo.
# Usage: ./scripts/phase-5-closeout.sh
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-duketopceo/kurultai}"
MILESTONE_6=6
DEFER_COMMENT='Phase 5 product exit shipped: daemon poll (#65), notify watch (#66), local ONNX embeddings (#84), multi-agent MCP init (#83). This issue remains deferred ops/infra — remilestoned to Milestone 6. See docs/plans/phase-5-complete.md.'

main_file_exists() {
  local path="$1"
  gh api "repos/${REPO}/contents/${path}?ref=main" --jq .sha >/dev/null 2>&1
}

main_file_text() {
  local path="$1"
  gh api "repos/${REPO}/contents/${path}?ref=main" --jq .content | tr -d '\n' | base64 -d
}

echo "Preflight: PRs #65/#66/#83/#84 merged into main on $REPO…"
for pr in 65 66 83 84; do
  meta="$(gh pr view "$pr" --repo "$REPO" --json state,baseRefName,mergedAt)"
  state="$(echo "$meta" | jq -r .state)"
  base="$(echo "$meta" | jq -r .baseRefName)"
  merged="$(echo "$meta" | jq -r .mergedAt)"
  if [[ "$state" != "MERGED" || "$base" != "main" || "$merged" == "null" ]]; then
    echo "Abort: PR #$pr state=$state base=$base mergedAt=$merged (expected MERGED into main)" >&2
    exit 1
  fi
done

echo "Preflight: closeout docs on canonical $REPO@main…"
for path in \
  docs/plans/phase-5-complete.md \
  docs/plans/phase-5-closeout.md \
  scripts/phase-5-closeout.sh \
  README.md
do
  if ! main_file_exists "$path"; then
    echo "Abort: $path missing on $REPO@main (merge the Phase 5 closeout PR first)." >&2
    exit 1
  fi
done

readme="$(main_file_text README.md)"
if ! echo "$readme" | grep -Eq '\]\([^)]*phase-5-complete\.md\)'; then
  echo "Abort: README.md on $REPO@main has no Markdown link targeting phase-5-complete.md." >&2
  exit 1
fi
if ! echo "$readme" | grep -Eq '\]\([^)]*phase-4-complete\.md\)'; then
  echo "Abort: README.md on $REPO@main has no Markdown link targeting phase-4-complete.md." >&2
  exit 1
fi
# Product-exit signal: Phase 5 complete link must appear with ✅ on the same line
if ! echo "$readme" | grep -E '✅.*phase-5-complete|phase-5-complete.*✅' >/dev/null; then
  echo "Abort: README.md on $REPO@main has no Phase 5 ✅ product-exit signal (expected ✅ … phase-5-complete on one line)." >&2
  exit 1
fi

echo "Remilestoning deferred ops #20/#29/#35 to Milestone $MILESTONE_6 on $REPO…"
failed=0
for issue in 20 29 35; do
  if gh api -X PATCH "repos/${REPO}/issues/${issue}" -f milestone="$MILESTONE_6" >/dev/null \
    && gh issue comment "$issue" --repo "$REPO" --body "$DEFER_COMMENT" >/dev/null; then
    echo "  #$issue → milestone $MILESTONE_6"
  else
    echo "  FAILED: #$issue (milestone PATCH and/or comment)" >&2
    failed=1
  fi
done

if [[ "$failed" -ne 0 ]]; then
  echo "Abort: one or more remilestone/comment steps failed; Milestone 5 not clear yet. Re-run after fixing access/errors." >&2
  exit 1
fi

echo "Reminder: #9 already closed (product exit via #65/#66/#83/#84)."
echo "Then close Milestone 5 when clear:"
echo "  gh api -X PATCH repos/$REPO/milestones/5 -f state=closed"

echo "Done. See docs/plans/phase-5-closeout.md"
