---
index: kurultai/v1
folder: src/mcp
parent: src/INDEX.md
updated: 2026-09-11
version: 3
---

# `src/mcp`

**Does:** MCP stdio + agent init
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`brain.rs`](brain.rs) | BrainService implementing AgentRead; `ask_with_web` ephemeral web augmentation + Jev sufficiency gate | `src/activity` · `src/brain` · `src/embed` · `src/eval` · `src/web` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-09-17 | 3 | 2026-09-17 `web_searcher`/`judge` fields + `ask_with_web` (web pseudo-hits never touch store/activity/graph_chain) · 2026-09-15 `tier_policy` field + `with_tier_policy` (#325) |
| [`init.rs`](init.rs) | kurultai init --agent cursor/claude/codex/hermes | `src/config` · `src/error` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`interface.rs`](interface.rs) | AgentRead trait | `src/synthesize` · `src/types` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-07-23 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | MCP module: stdio server + init wiring | — | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`server.rs`](server.rs) | MCP tool dispatch (search, ask, ontology_*, hey_*) | `src/error` · `src/mcp` · `src/ontology` · `src/project` · `src/write_policy` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` | 2026-09-18 | 4 | 2026-09-18 ask tool `web` param → ask_with_web · 2026-09-11 `ontology_propose` (full surface) + `ontology_proposals` (read) (#118) · 2026-09-04 hey_* message board tools + clamp + readonly list update · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-17 — `brain.rs`: `ask_with_web` + `with_web_searcher`/`with_judge` — Perplexity augmentation behind sufficiency gate
- 2026-09-15 — `brain.rs`: `BrainService.tier_policy` + `with_tier_policy`; tier calls use configured policy (#325)
- 2026-09-11 — `ontology_propose`/`ontology_proposals` tools: agents submit drafts, humans decide via REST/UI (#118)
- 2026-09-04 — `server.rs` adds `hey_threads/read/poll` tools and read-only tool list
- 2026-08-16 — indexed this folder (v1 seed)
