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
| [`mod.rs`](mod.rs) |  GoldenSet/Matcher types, `run_eval` HTTP runner (`POST /api/search` + `/api/ask`), report + aggregate, judge circuit breaker  | `reqwest` · `src/eval/judge` · `src/eval/metrics` | `src/main.rs` | 2026-09-18 | 3 | 2026-09-18 `pub mod review` (pre-merge commit judge) · 2026-09-18 judge circuit breaker (3 consecutive failures → labels-only, `judge_disabled_reason` in report) · 2026-09-17 added |
| [`metrics.rs`](metrics.rs) | Pure metrics: recall_at_k, precision_at_k, mrr, ndcg_at_k, mean | — | `src/eval/mod.rs` | 2026-09-17 | 1 | 2026-09-17 added |
| [`review.rs`](review.rs) | Pre-merge commit review: `collect_commits` (git rev-list, bot-excluded) + `review_commits` (11-question Jev suite) + hard/advisory flag thresholds | `git` CLI · `src/eval/judge` | `src/main.rs` | 2026-09-18 | 1 | 2026-09-18 added |
| [`judge.rs`](judge.rs) | `Judge` trait + `OpenRouterJudge` (decisions API, pinned `typesafe/jev-1.13`) + `NullJudge` + `judge_from_env` | `reqwest` · `src/security` | `src/eval/mod.rs` · `src/mcp/brain.rs` | 2026-09-18 | 2 | 2026-09-18 `judge_from_config` honors `[judge] enabled`/`model` · 2026-09-17 added |

## Recent

- 2026-09-18 — `review.rs` pre-merge commit judge + `Commands::Review` (exit 1 on hard flags)
- 2026-09-18 — `mod.rs` judge circuit breaker + `judge_disabled_reason` report field
- 2026-09-17 — added for retrieval evals phase (`kurultai eval`)
