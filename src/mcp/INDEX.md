---
index: kurultai/v1
folder: src/mcp
parent: src/INDEX.md
updated: 2026-09-30
version: 3
---

# `src/mcp`

**Does:** MCP stdio + broker stdio relay + agent init
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`brain.rs`](brain.rs) | BrainService implementing AgentRead; `ask_with_web` ephemeral web augmentation + Jev sufficiency gate | `src/activity` · `src/brain` · `src/embed` · `src/eval` · `src/web` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-10-02 | 4 | 2026-10-02 ask paths set `answer.gaps` |
| [`broker_stdio.rs`](broker_stdio.rs) | `mcp --broker` stdio↔broker relay: frames → `POST /mcp` with `X-Kurultai-Session` | `src/mcp/server.rs` · `reqwest` | `src/main.rs` | 2026-09-30 | 1 | 2026-09-30 added — agent lane holds no upstream credentials (plan 2026-09-28-001 U3) |
| [`init.rs`](init.rs) | kurultai init --agent cursor/claude/codex/hermes | `src/config` · `src/error` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`interface.rs`](interface.rs) | AgentRead trait | `src/synthesize` · `src/types` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-07-23 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | MCP module: stdio server + init wiring | — | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`server.rs`](server.rs) | MCP tool dispatch (search, ask, ontology_*, hey_*); frame/error/response helpers pub(crate) for broker_stdio | `src/error` · `src/mcp` · `src/ontology` · `src/project` · `src/write_policy` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/broker_stdio.rs` | 2026-09-30 | 5 | 2026-09-30 `read_stdin_frame`/`rpc_error`/`write_response`/`StdinFrame` → pub(crate) for broker relay · 2026-09-18 ask tool `web` param → ask_with_web · 2026-09-11 `ontology_propose` (full surface) + `ontology_proposals` (read) (#118) · 2026-09-04 hey_* message board tools + clamp + readonly list update · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-10-02 — `brain.rs`: `ask_with_team`/`ask_with_web` populate `Answer.gaps` (local hits only)

- 2026-09-30 — `broker_stdio.rs` (U3): `mcp --broker` relays JSON-RPC frames to the device broker `POST /mcp` with `X-Kurultai-Session`; `server.rs` frame helpers → pub(crate)
- 2026-09-17 — `brain.rs`: `ask_with_web` + `with_web_searcher`/`with_judge` — Perplexity augmentation behind sufficiency gate
- 2026-09-15 — `brain.rs`: `BrainService.tier_policy` + `with_tier_policy`; tier calls use configured policy (#325)
- 2026-09-11 — `ontology_propose`/`ontology_proposals` tools: agents submit drafts, humans decide via REST/UI (#118)
- 2026-09-04 — `server.rs` adds `hey_threads/read/poll` tools and read-only tool list
- 2026-08-16 — indexed this folder (v1 seed)
