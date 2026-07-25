# Phase 4 closeout — tracker hygiene

**Product Phase 4 solo expansion is shipped** (#62 Dayflow+Pond, #63 GitHub FS).  
This file is **issue/milestone hygiene** so Milestone 4 can close.

Agent tokens may **cannot** `closeIssue` (403). A maintainer runs the commands below.

---

## Close (shipped / umbrella done)

```bash
gh issue close 8 --comment "Phase 4 solo exit shipped: Dayflow+Pond (#62), GitHub FS (#63). Deferred: Composio, plugins (#14), CodeGraph, AppFlowy (#4), OpenRouter batch — see docs/plans/phase-4-complete.md."
```

One-shot script: `./scripts/phase-4-closeout.sh`

---

## Already closed

- [#21](https://github.com/duketopceo/kurultai/issues/21) Dayflow — closed via #62

---

## Leave open (deferred / other milestones)

- [#4](https://github.com/duketopceo/kurultai/issues/4) AppFlowy — deferred Expansion leftover (still on Milestone 1 historically; remilestone optional)
- [#14](https://github.com/duketopceo/kurultai/issues/14) Plugin ecosystem — later
- [#23](https://github.com/duketopceo/kurultai/issues/23) Testing gates — cross-cutting; tick Phase 4 items when coverage/deny land

---

## Milestone 4

When Milestone 4 has no blocking open issues (#8 closed):

```bash
gh api -X PATCH repos/duketopceo/kurultai/milestones/4 -f state=closed
```

---

## Done when

1. Phase 4 connectors on `main` (#62, #63)  
2. #8 closed; Milestone 4 closable  
3. `phase-4-complete.md` on `main`  
4. README Phase 4 ✅  
