---
index: kurultai/v1
folder: src/mcp
parent: src/INDEX.md
updated: 2026-10-20
version: 4
---

# `src/mcp`

**Does:** MCP stdio + broker stdio relay + agent init
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`brain.rs`](brain.rs) | BrainService implementing AgentRead; `ask_with_web` ephemeral web augmentation + Jev sufficiency gate; `remember_with` stamps member `team_id`/`visibility=team` from `PgScope` (KTD5) | `src/activity` · `src/brain` · `src/embed` · `src/eval` · `src/web` · `src/store/scope.rs` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-10-20 | 5 | 2026-10-20 hub adoption U2: `remember_with` member stamping from `PgScope::current()` (visibility=team + server team_id, client team_id overwritten); `multi_hop_expand` re-enters scope inside `JoinSet` spawns (task-locals don't cross spawn) · 2026-10-02 ask paths set `answer.gaps` |
| [`broker_stdio.rs`](broker_stdio.rs) | `mcp --broker` stdio↔broker relay: frames → `POST /mcp` with `X-Kurultai-Session` | `src/mcp/server.rs` · `reqwest` | `src/main.rs` | 2026-09-30 | 1 | 2026-09-30 added — agent lane holds no upstream credentials (plan 2026-09-28-001 U3) |
| [`init.rs`](init.rs) | kurultai init --agent cursor/claude/codex/hermes | `src/config` · `src/error` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`interface.rs`](interface.rs) | AgentRead trait | `src/synthesize` · `src/types` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-07-23 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | MCP module: stdio server + init wiring | — | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`server.rs`](server.rs) | MCP tool dispatch (search, ask, ontology_*, hey_*); frame/error/response helpers pub(crate) for broker_stdio | `src/error` · `src/mcp` · `src/ontology` · `src/project` · `src/write_policy` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/broker_stdio.rs` | 2026-09-30 | 5 | 2026-09-30 `read_stdin_frame`/`rpc_error`/`write_response`/`StdinFrame` → pub(crate) for broker relay · 2026-09-18 ask tool `web` param → ask_with_web · 2026-09-11 `ontology_propose` (full surface) + `ontology_proposals` (read) (#118) · 2026-09-04 hey_* message board tools + clamp + readonly list update · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-10-20 — `brain.rs` (hub adoption U2, KTD5): `remember_with` reads `PgScope::current()` — a member scope forces `visibility=team` + `metadata.team_id=scope.team_id` (overwrites client-supplied value); `multi_hop_expand` captures the scope and re-enters it inside `JoinSet` spawned hop queries so member asks can't merge other teams' atoms
- 2026-10-02 — `brain.rs`: `ask_with_team`/`ask_with_web` populate `Answer.gaps` (local hits only)

- 2026-09-30 — `broker_stdio.rs` (U3): `mcp --broker` relays JSON-RPC frames to the device broker `POST /mcp` with `X-Kurultai-Session`; `server.rs` frame helpers → pub(crate)
- 2026-09-17 — `brain.rs`: `ask_with_web` + `with_web_searcher`/`with_judge` — Perplexity augmentation behind sufficiency gate
- 2026-09-15 — `brain.rs`: `BrainService.tier_policy` + `with_tier_policy`; tier calls use configured policy (#325)
- 2026-09-11 — `ontology_propose`/`ontology_proposals` tools: agents submit drafts, humans decide via REST/UI (#118)
- 2026-09-04 — `server.rs` adds `hey_threads/read/poll` tools and read-only tool list
- 2026-08-16 — indexed this folder (v1 seed)
