---
index: kurultai/v1
folder: src/web
parent: src/INDEX.md
updated: 2026-09-17
version: 1
---

# `src/web`

**Does:** Ephemeral web search for `ask --web` — Perplexity `/search`, REST-direct, no atom writes
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

- (none)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`mod.rs`](mod.rs) | `WebSearcher` trait + `PerplexitySearcher` + `NullWebSearcher` + `web_searcher_from_env` (`PERPLEXITY_API_KEY`) | `reqwest` · `src/security` | `src/mcp/brain.rs` · `src/main.rs` | 2026-09-17 | 1 | 2026-09-17 added |

## Recent

- 2026-09-17 — added for `ask --web` ephemeral augmentation
