# Phase 1 closeout — tracker hygiene

**Product Phase 1 is already shipped** on `main` ([phase-1-complete.md](phase-1-complete.md)).  
This file is the **issue/milestone hygiene** pass so Milestone 1 can close.

Agent tokens for this repo **cannot** `closeIssue` / edit milestones (403). A maintainer must run the commands below.

---

## Close (shipped)

```bash
gh issue close 5  --comment "Shipped: CLI index/status/search in #46. Phase 1 complete."
gh issue close 29 --comment "Shipped: environments in #30."
gh issue close 40 --comment "Shipped: docs/upstream-inspiration.md via #41 (living matrix)."
gh issue close 42 --comment "Phase 1 CE plan executed; see docs/plans/phase-1-complete.md."
gh issue close 25 --comment "Audience order documented (#26); umbrella remains #27."
```

One-liner:

```bash
gh issue close 5 29 40 42 25
```

---

## Move off Milestone 1 (not Phase 1 exit blockers)

| Issue | Action |
|-------|--------|
| [#4](https://github.com/duketopceo/kurultai/issues/4) AppFlowy | Remove from Milestone 1 → Milestone 4 (Expansion) or no milestone. Deferred by design. |
| [#23](https://github.com/duketopceo/kurultai/issues/23) Testing gates | Remove from Milestone 1 — **cross-cutting**; keep open. Tick Phase 1 boxes if still unchecked. |
| [#33](https://github.com/duketopceo/kurultai/issues/33) Schema post-train | Remove from Milestone 1 → Phase 3+ / backlog. |

```bash
# AppFlowy → Phase 4 Expansion (milestone number may vary — confirm with: gh api repos/duketopceo/kurultai/milestones)
gh issue edit 4  --milestone "Phase 4: Expansion"
gh issue edit 23 --milestone ""
gh issue edit 33 --milestone "Phase 3: Synthesis & Interface"

# Close Milestone 1 when open_issues == 0
gh api -X PATCH repos/duketopceo/kurultai/milestones/1 -f state=closed
```

---

## #23 Phase 1 checklist (edit issue body)

Mark Phase 1 “Add in Phase 1” items done where true on `main`:

- [x] Store integration tests  
- [x] Config loader fixture tests  
- [x] Connector registry tests  
- [ ] `cargo test` required check on PRs (branch protection — repo settings, not code)

Leave Phase 2+ sections for after [#53](https://github.com/duketopceo/kurultai/pull/53) merges.

---

## Done when

1. Issues **#5 #29 #40 #42 #25** closed  
2. **#4 #23 #33** off Milestone 1  
3. Milestone **Phase 1** `state=closed`  
4. This PR merged so the closeout script stays in-tree  

Phase 2 search (#6 / #51) is already on `main`. Next product work: merge #53–#55, continue #7.
