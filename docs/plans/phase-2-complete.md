# Phase 2 complete — wrap-up

**Status:** ✅ Product on `main` (2026-07-21, #51) · Testing gates landed via closeout (from [#53](https://github.com/duketopceo/kurultai/pull/53))  
**Plans:** [search RRF](2026-07-21-001-feat-search-retrieval-rrf-plan.md) · [testing gates](2026-07-21-002-feat-phase2-testing-gates-plan.md) · [closeout](2026-07-23-001-chore-phase2-closeout-plan.md)  
**Tracking:** [#6](https://github.com/duketopceo/kurultai/issues/6) (close) · [#23](https://github.com/duketopceo/kurultai/issues/23) Phase 2 (tick)  
**Exit path:**

```
FTS ∥ vector → RRF(k=60) → optional rerank → markdown context → capped AgentAtomView
```

---

## What shipped

| Work order | PR | Notes |
|------------|-----|--------|
| Graph orchestration note | [#48](https://github.com/duketopceo/kurultai/pull/48) | Diamond doctrine for #6/#7 |
| Search CE plan | [#50](https://github.com/duketopceo/kurultai/pull/50) | Plan-only (close as obsolete) |
| RRF diamond + rerank + context | [#51](https://github.com/duketopceo/kurultai/pull/51) | Product Phase 2 |
| Testing CE plan | [#52](https://github.com/duketopceo/kurultai/pull/52) | Plan-only (close as obsolete) |
| Hybrid tests + nextest + llvm-cov | [#53](https://github.com/duketopceo/kurultai/pull/53) / closeout merge | #23 Phase 2 gates |

**Deferred (not Phase 2 exit):** [#12](https://github.com/duketopceo/kurultai/issues/12) LLM distillation · coverage ≥50% (#23 Phase 3).

---

## Exit criteria — verified

1. CLI/MCP `search` returns RRF-fused capped views; FTS-only without API key  
2. Soft-fail: embed/vector/rerank failure keeps best available ranking  
3. Integration: `tests/retrieval_hybrid.rs` (overlap, NullEmbedder, stub rerank, context expand)  
4. RRF golden deepen in `src/query/rrf.rs` (`k=60`, tie-break, empty lists)  
5. CI: Linux `cargo nextest` + llvm-cov artifact (**no** `--fail-under`); macOS may keep `cargo test`  

---

## Hardened invariants (do not regress)

| Invariant | Where |
|-----------|--------|
| RRF `k=60`, 1-based ranks, id tie-break | `query/rrf` |
| Parallel FTS ∥ vector; soft-fail arms | `query/hybrid` |
| Optional rerank soft-fail → keep RRF | `query/hybrid` + `rerank` |
| Neighbor expand under excerpt cap | `query/context` |
| MCP/CLI never dump full `content` | `AgentAtomView` / `DEFAULT_EXCERPT_CAP` |

---

## Tracker closeout

Maintainer: [phase-2-closeout.md](phase-2-closeout.md) / `scripts/phase-2-closeout.sh`  
(Agent token often lacks `closeIssue`.)

---

## Next: Phase 3

1. Merge drafted [#54](https://github.com/duketopceo/kurultai/pull/54) ask synthesis (WO1) and [#55](https://github.com/duketopceo/kurultai/pull/55) HTTP daemon (WO2)  
2. Then WO3 planner → WO4 agent capture → WO5 #23 Phase 3 gates  
3. Tracking: [#7](https://github.com/duketopceo/kurultai/issues/7)
