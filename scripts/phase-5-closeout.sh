#!/usr/bin/env bash
# Phase 5 tracker closeout — run as a maintainer with issue/milestone write access.
# Run after the closeout PR is on main of the canonical repo.
# Usage: ./scripts/phase-5-closeout.sh
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-duketopceo/kurultai}"
MILESTONE_6=6
DEFER_COMMENT='Phase 5 product exit shipped: daemon poll (#65), notify watch (#66), local ONNX embeddings (#84), multi-agent MCP init (#83). This issue remains deferred ops/infra — remilestoned to Milestone 6. See docs/plans/phase-5-complete.md.'
# Stable marker for idempotent comments (substring of DEFER_COMMENT).
DEFER_MARKER='Phase 5 product exit shipped:'

main_file_exists() {
  local path="$1"
  gh api "repos/${REPO}/contents/${path}?ref=main" --jq .sha >/dev/null 2>&1
}

# Portable base64 decode: GNU uses -d, BSD/macOS uses -D.
if printf 'dGVzdA==' | base64 -d 2>/dev/null | grep -qx test; then
  B64_DECODE=(base64 -d)
else
  B64_DECODE=(base64 -D)
fi

main_file_text() {
  local path="$1"
  gh api "repos/${REPO}/contents/${path}?ref=main" --jq .content | tr -d '\n' | "${B64_DECODE[@]}"
}

issue_has_defer_comment() {
  local issue="$1"
  gh api "repos/${REPO}/issues/${issue}/comments?per_page=100" --jq '.[].body' \
    | grep -Fq "$DEFER_MARKER"
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
# Exact basename links (reject …foo-phase-5-complete.md style false positives).
if ! echo "$readme" | grep -Eq '\]\((\.\/)?(docs/plans/)?phase-5-complete\.md\)'; then
  echo "Abort: README.md on $REPO@main has no Markdown link targeting phase-5-complete.md." >&2
  exit 1
fi
if ! echo "$readme" | grep -Eq '\]\((\.\/)?(docs/plans/)?phase-4-complete\.md\)'; then
  echo "Abort: README.md on $REPO@main has no Markdown link targeting phase-4-complete.md." >&2
  exit 1
fi
# Product-exit signal: Phase 5 table row with ✅ and phase-5-complete link on the same line.
if ! echo "$readme" | grep -E '^\|[[:space:]]*5[[:space:]].*✅.*\(docs/plans/phase-5-complete\.md\)' >/dev/null; then
  echo "Abort: README.md on $REPO@main has no Phase 5 table row with ✅ and docs/plans/phase-5-complete.md." >&2
  exit 1
fi

echo "Remilestoning deferred ops #20/#29/#35 to Milestone $MILESTONE_6 on $REPO…"
failed=0
for issue in 20 29 35; do
  if ! gh api -X PATCH "repos/${REPO}/issues/${issue}" -f milestone="$MILESTONE_6" >/dev/null; then
    echo "  FAILED: #$issue (milestone PATCH)" >&2
    failed=1
    continue
  fi
  if issue_has_defer_comment "$issue"; then
    echo "  #$issue → milestone $MILESTONE_6 (comment already present; skipped)"
  elif gh issue comment "$issue" --repo "$REPO" --body "$DEFER_COMMENT" >/dev/null; then
    echo "  #$issue → milestone $MILESTONE_6"
  else
    echo "  FAILED: #$issue (comment)" >&2
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
