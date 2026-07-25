# Phase 4 complete — wrap-up

**Status:** ✅ Solo expansion on `main` (2026-07-24–25)  
**Plans:** [Pond + Dayflow](2026-07-24-001-feat-phase-4-pond-dayflow-connectors-plan.md) · [GitHub FS](2026-07-25-001-feat-phase-4-github-connector-plan.md) · [LFG closeout](2026-07-25-002-chore-phase4-lfg-closeout-plan.md)  
**Tracking:** [#8](https://github.com/duketopceo/kurultai/issues/8) (close) · [#21](https://github.com/duketopceo/kurultai/issues/21) (closed via #62)  
**Exit path:**

```text
markdown notes ∥ Pond chats ∥ Dayflow activity ∥ local GitHub checkout
  → IndexPipeline → SQLite brain → search / ask
```

---

## What shipped

| Work order | PR | Notes |
|------------|-----|--------|
| Dayflow + Pond connectors | [#62](https://github.com/duketopceo/kurultai/pull/62) | Closes #21; `kind=dayflow` / `kind=pond` |
| GitHub filesystem connector | [#63](https://github.com/duketopceo/kurultai/pull/63) | Local `root_path` checkout; no API/CodeGraph |

**Solo audience exit:** notes + agent history + activity journal + code on disk — all FTS-first without new cloud keys.

---

## Deferred (not Phase 4 exit)

| Item | Tracker | Notes |
|------|---------|--------|
| Composio meta-connector | #8 remnant / future WO | Hermes vs direct API undecided |
| Plugin system (Python/WASM) | [#14](https://github.com/duketopceo/kurultai/issues/14) | Ecosystem phase |
| CodeGraph / tree-sitter edges | #8 remnant | GitHub FS is enough for FTS |
| AppFlowy | [#4](https://github.com/duketopceo/kurultai/issues/4) | Deferred since Phase 1 |
| OpenRouter batch / fallback embed | #8 remnant | Quality follow-up |
| TechTracker composite | #8 remnant | Dayflow covers activity slice |
| #23 Phase 4 coverage ≥60% / cargo-deny | [#23](https://github.com/duketopceo/kurultai/issues/23) | Cross-cutting; not solo exit |

---

## Exit criteria — verified

1. `SourceKind::Dayflow` / `Pond` / `GitHub` register and index via `kurultai index`  
2. Fixture/CI-safe Dayflow + GitHub tests; Pond graceful without binary  
3. Symlink-safe GitHub walk (`O_NOFOLLOW`); skip `node_modules` / `target` / `.git`  
4. README connectors list markdown · Dayflow · Pond · GitHub FS  

---

## Hardened invariants (do not regress)

| Invariant | Where |
|-----------|--------|
| Connectors are read-only source adapters; brain stays SQL | `connectors/*` |
| Atom `source` = config source **name** | markdown / GitHub / dayflow / pond |
| MCP never dumps full `content` by default | `AgentAtomView` |
| Missing optional tooling (pond binary, Dayflow DB) → clear error, no panic | pond / dayflow |

---

## Tracker closeout

Maintainer: [phase-4-closeout.md](phase-4-closeout.md) / `scripts/phase-4-closeout.sh`  
(Agent token often lacks `closeIssue`.)

---

## Next: Phase 5

1. [#9](https://github.com/duketopceo/kurultai/issues/9) perf + shared daemon  
2. [#20](https://github.com/duketopceo/kurultai/issues/20) self-hosted CI (ARC)  
3. Optional later: Composio / plugins as Expansion follow-ups (not Phase 5 blockers)  
