---
index: kurultai/v1
folder: evals
parent: INDEX.md
updated: 2026-09-17
version: 1
---

# `evals`

**Does:** Retrieval eval golden set + labeling docs (runner lives in `src/eval/`, CLI `kurultai eval`)
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

- (none)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`golden.json`](golden.json) | Frozen golden query set — matchers on title/source_id/title_hash, `deny_sources` noise assertions, `kind: search\|ask` | `kurultai eval` · `src/eval/` | — | 2026-09-17 | 1 | 2026-09-17 seeded ~19 queries (search + ask + negative/noise) |
| [`README.md`](README.md) | Labeling guide, judge + `ask --web` usage, metric definitions | — | — | 2026-09-18 | 2 | 2026-09-18 `kurultai review` pre-merge commit check docs · 2026-09-17 added |

## Recent

- 2026-09-18 — `README.md` `kurultai review` pre-merge check section
- 2026-09-17 — seeded golden.json + labeling guide for `kurultai eval` (retrieval evals phase)
