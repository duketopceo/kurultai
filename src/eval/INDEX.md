---
index: kurultai/v1
folder: src/eval
parent: src/INDEX.md
updated: 2026-09-17
version: 1
---

# `src/eval`

**Does:** Retrieval eval harness — golden-set types, HTTP runner, pure metrics, optional Jev judge
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

- (none)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`mod.rs`](mod.rs) | GoldenSet/Matcher types, `run_eval` HTTP runner (`POST /api/search` + `/api/ask`), report + aggregate | `reqwest` · `src/eval/judge` · `src/eval/metrics` | `src/main.rs` | 2026-09-17 | 1 | 2026-09-17 added |
| [`metrics.rs`](metrics.rs) | Pure metrics: recall_at_k, precision_at_k, mrr, ndcg_at_k, mean | — | `src/eval/mod.rs` | 2026-09-17 | 1 | 2026-09-17 added |
| [`judge.rs`](judge.rs) | `Judge` trait + `OpenRouterJudge` (decisions API, pinned `typesafe/jev-1.13`) + `NullJudge` + `judge_from_env` | `reqwest` · `src/security` | `src/eval/mod.rs` · `src/mcp/brain.rs` | 2026-09-17 | 1 | 2026-09-17 added |

## Recent

- 2026-09-17 — added for retrieval evals phase (`kurultai eval`)
