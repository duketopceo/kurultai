---
title: Phase 5 closeout — tracker hygiene
tags:
  - phase-5
  - closeout
  - docs
---

# Phase 5 closeout — tracker hygiene

**Product Phase 5 production-readiness is shipped** (#65 poll, #66 watch, #84 local embeddings, #83 multi-agent MCP).  
This file is **issue/milestone hygiene** so Milestone 5 can close.

Agent tokens often **cannot** edit milestones / issues (403). A maintainer runs the commands below.

---

## Landing vehicle

Merge **[#97](https://github.com/duketopceo/kurultai/pull/97)** (`cursor/phase5-lfg-closeout-7a74`) to put this file, `phase-5-complete.md`, `scripts/phase-5-closeout.sh`, and the README Phase 5 ✅ roadmap table on `main` (coexists with the v0.3.0 README rewrite from [#98](https://github.com/duketopceo/kurultai/pull/98)).

Maintainer sequence: merge #97 → `./scripts/phase-5-closeout.sh` → close Milestone 5.

**Remilestone status (2026-07-26):** agent `gh` integration got **403** on `PATCH /issues/{20,29,35}` milestone updates — deferred ops still on Milestone 5 until a maintainer runs the script.

---

## Remilestone (deferred ops — leave open)

After the closeout PR is on `main`:

```bash
./scripts/phase-5-closeout.sh
```

The script preflights that #65/#66/#83/#84 are merged into `main`, that wrap docs exist on `origin/main`, then remilestones [#20](https://github.com/duketopceo/kurultai/issues/20) / [#29](https://github.com/duketopceo/kurultai/issues/29) / [#35](https://github.com/duketopceo/kurultai/issues/35) to **Milestone 6** with deferred-ops comments. It does **not** close those issues. [#9](https://github.com/duketopceo/kurultai/issues/9) is already closed.

---

## Already closed

- [#9](https://github.com/duketopceo/kurultai/issues/9) Perf + shared daemon — closed early; product exit via #65/#66/#84/#83

---

## Leave open (deferred / Milestone 6)

- [#20](https://github.com/duketopceo/kurultai/issues/20) Self-hosted CI (ARC) — ops/infra  
- [#29](https://github.com/duketopceo/kurultai/issues/29) Environments hardening — foundation in #30; gates/secrets/RBAC remain  
- [#35](https://github.com/duketopceo/kurultai/issues/35) GlitchTip projects — error tracking  

---

## Milestone 5

When Milestone 5 has no blocking open issues (ops remilestoned):

```bash
gh api -X PATCH repos/duketopceo/kurultai/milestones/5 -f state=closed
```

---

## Done when

1. Phase 5 product PRs on `main` (#65, #66, #83, #84)  
2. Verification on the closeout landing commit (record results):  
   - required files exist: `docs/plans/phase-5-complete.md`, `docs/plans/phase-5-closeout.md`, `scripts/phase-5-closeout.sh`  
   - README contains Markdown links to `phase-4-complete.md` and `phase-5-complete.md`  
   - README Phase 5 row is ✅ (embeddings not listed as still-pending)  
   - optional: `cargo test --locked` / `cargo clippy --all-targets -- -D warnings`  
3. `./scripts/phase-5-closeout.sh` succeeds (#20/#29/#35 on Milestone 6)  
4. Milestone 5 closable  
