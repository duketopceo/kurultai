# hey-board Skill

> The hey.md board is the single source of truth for agent work coordination. Every agent reads it at session start, writes status updates as they work, and records completions when done.

## When to Use

- Agent starts a session and needs to know what to work on
- Agent completes a work order and needs to record it
- Agent reviews another agent's work and needs to log the result
- Any coordination between agents on the Khan or Kurultai repos

## Prerequisites

- Repo cloned locally with git configured
- `hey.md` exists at repo root (or create from template below)
- GitHub CLI (`gh`) via `api_credentials=["github"]`
- Linear team: `Khan` (key: `KHAN`), workspace: `imluketheduke`

## Lifecycle — Work Order Driven

Every phase starts with a **phase load**: the coordinator (perplexity or Luke) creates work orders in Linear, assigns them to agents, and seeds the hey.md phase table. Agents don't pick work randomly — they work what's assigned.

### Work Order Lifecycle

```
Backlog → Working → Review Ready → Reviewing → Changes Requested | Done/Merged
```

| Status | Meaning | Who acts |
|---|---|---|
| Backlog | Assigned but not started | Implementer starts |
| Working | Implementer is actively working | Implementer |
| Review Ready | PR open, implementation complete | Reviewer picks up |
| Reviewing | Reviewer is checking the PR | Reviewer |
| Changes Requested | Reviewer found issues | Implementer fixes |
| Done/Merged | PR merged, Linear issue Done | Merge captain |

### Rules

1. **No self-review.** The implementer never reviews their own PR.
2. **One reviewer per issue.** Assigned at phase load, not picked ad hoc.
3. **Reviewer is merge captain.** If the review passes, the reviewer merges the PR, moves the Linear issue to Done, and updates hey.md — all in one sweep.
4. **Codegraph-first.** If `.codegraph/` exists, use it before broad `grep`. Saves tokens.
5. **hey.md pushes directly to main.** PR-gate exempt for board edits only.
6. **Every work order has:** Linear issue, implementer, reviewer, acceptance criteria, branch/PR link.
7. **Attribution.** Every hey.md edit includes agent alias and version bump.

## hey.md Format

```markdown
# hey.md — <repo-name>

**Version:** v0.XXXX.Y
**Last edit:** <alias> v0.XXXX.Y — <note>

## Current Phase: <phase name>

| KHAN-ID | Title | Implementer | Reviewer | Status | Branch/PR |
|---|---|---|---|---|---|
| KHAN-XXX | <title> | <agent> | <agent> | Working | feature/xxx |
| KHAN-XXX | <title> | <agent> | <agent> | Done/Merged | PR #XX (merged) |

## Phase History

### <previous phase> — Complete
- All issues merged: KHAN-XXX through KHAN-XXX
- Merge sweep: <alias> on <date>

## Changelog
- v0.XXXX.Y — <alias> — <note>
```

## Version Scheme

- Format: `v0.NNNN.Y` where NNNN is the phase number, Y is the edit count
- Bump Y on every meaningful edit
- Bump NNNN on new phase

## Commit Convention

```
docs(hey): <alias> v0.XXXX.Y <note>
```

## How to Edit hey.md

1. **Pull main first:**
   ```bash
   cd /path/to/repo
   git pull origin main
   ```

2. **Edit the file** — update status, add rows, record completions

3. **Bump version** — increment Y, update last-edit line

4. **Commit and push:**
   ```bash
   git add hey.md
   git commit -m "docs(hey): <alias> v0.XXXX.Y <note>"
   git push origin main
   ```

## Review Checklist (for reviewers / merge captains)

When reviewing a PR:

- [ ] No hardcoded secrets or API keys
- [ ] Auth on new routes (401 for unauthenticated)
- [ ] No breaking changes to existing API contracts
- [ ] New env vars documented and noted for deployment
- [ ] Test coverage for new logic
- [ ] No test/demo data leaking into production
- [ ] Static assets synced if touching UI

## Merge

Once review passes:

```bash
gh pr merge <number> --squash --delete-branch
```

Then:
1. Move Linear issue to Done
2. Update hey.md row to "Done/Merged"
3. Commit hey.md with version bump

## Canon

GitHub Issues/Projects win on conflict. Linear is the downstream mirror. hey.md is the live coordination layer between both.
