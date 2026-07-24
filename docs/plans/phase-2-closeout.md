# Phase 2 closeout — tracker hygiene

**Product Phase 2 search is shipped** (#51). Testing gates are in the closeout branch (from #53).  
This file is **issue/PR hygiene** so Milestone 2 can close.

Agent tokens may **cannot** `closeIssue` (403). A maintainer runs the commands below.

---

## Close (shipped / obsolete)

```bash
gh issue close 6 --comment "Phase 2 search shipped in #51 (RRF + optional rerank + context). Distillation remains #12."

# Plan-only PRs superseded by #51 / #53
gh pr close 50 --comment "Obsolete: search plan landed via #51."
gh pr close 52 --comment "Obsolete: testing plan landed via #53 / Phase 2 closeout."
gh pr close 53 --comment "Superseded: testing content merged into Phase 2 closeout PR."
```

One-shot script: `./scripts/phase-2-closeout.sh`

---

## Update #23 Phase 2 checklist

Mark done in the issue body:

- [x] FTS + vector search integration tests with known embeddings  
- [x] RRF fusion golden-file tests  
- [x] `cargo llvm-cov` in CI (upload artifact, no hard gate yet)  
- [x] `cargo nextest` for faster parallel test runs  

Leave Phase 3+ unchecked. Keep #23 open (cross-cutting).

---

## Milestone 2

When Milestone 2 has no blocking open issues:

```bash
gh api -X PATCH repos/duketopceo/kurultai/milestones/2 -f state=closed
```

---

## Done when

1. Testing gates on `main`  
2. Issues/PRs above closed; #23 Phase 2 ticked  
3. `phase-2-complete.md` on `main`  
4. Milestone 2 closable  
