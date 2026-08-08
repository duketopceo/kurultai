---
title: "Plan: Clean up stale v0.3.0 feature branches and worktrees"
date: 2026-08-08
status: implementation-ready
author: Agent Zero
origin: user request /ce-plan
---

# Plan: Clean up stale v0.3.0 feature branches and worktrees

## Objective
Remove the now-merged v0.3.0 feature branches, their isolated git worktrees, and the redundant `release/v0.3.0` integration branch. Ensure local tags are synced with origin.

## Scope

### In scope
- Local branches:
  - `feat/v030-db-ingestion`
  - `feat/v030-backend-hardening`
  - `feat/v030-ui-rewrite`
  - `release/v0.3.0`
- Git worktrees under `.harness/worktrees/`
- Remote tracking branch `origin/release/v0.3.0`
- Missing local `v0.3.0` tag

### Out of scope
- `main` branch
- Any unmerged work or open PRs
- `.harness/` directory itself (only the worktrees inside)

## Settled decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Delete local feature branches | Yes | All three are merged into `main` (and upstream). |
| Delete `release/v0.3.0` branch | Yes | It was an integration branch; `main` has since moved ahead. |
| Delete remote `release/v0.3.0` | Yes | Redundant after merge; keep origin tidy. |
| Keep worktree source code? | No | Worktrees are just checkouts; source is preserved in branches. |
| Fetch tags | Yes | Origin has `v0.3.0`; local copy is missing. |

## Implementation units

### U1: Remove isolated worktrees
**Files touched:** `.harness/worktrees/`
**Commands:**
```bash
git worktree remove .harness/worktrees/feat-v030-db-ingestion
git worktree remove .harness/worktrees/feat-v030-backend-hardening
git worktree remove .harness/worktrees/feat-v030-ui-rewrite
```
**Acceptance:** `git worktree list` shows only the main worktree.

### U2: Delete local stale branches
**Commands:**
```bash
git branch -D feat/v030-db-ingestion
git branch -D feat/v030-backend-hardening
git branch -D feat/v030-ui-rewrite
git branch -D release/v0.3.0
```
**Acceptance:** `git branch --format='%(refname:short)'` no longer lists the four branches.

### U3: Delete remote release branch
**Command:**
```bash
git push origin --delete release/v0.3.0
```
**Acceptance:** `git branch -r` no longer shows `origin/release/v0.3.0`.

### U4: Sync tags
**Command:**
```bash
git fetch origin --tags
```
**Acceptance:** `git tag -l` includes `v0.3.0` and `git rev-parse v0.3.0` matches origin's tag.

## Verification

```bash
git status --short          # should be clean
git branch -a               # only main + origin/main
git worktree list           # only main worktree
git tag -l | grep v0.3.0    # tag present
```

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Accidentally delete unmerged work | Verify each branch is merged to `main` with `git branch --merged main` before deletion. |
| Worktree removal fails due to uncommitted changes | Check `git status` in each worktree first; abort if dirty. |
| Remote branch deletion is irreversible | Only delete `release/v0.3.0`, which is already merged. |

## Open questions
None.
