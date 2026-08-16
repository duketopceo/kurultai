---
index: kurultai/v1
folder: src/mcp
parent: src/INDEX.md
updated: 2026-08-16
version: 1
---

# `src/mcp`

**Does:** MCP stdio + agent init
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`brain.rs`](brain.rs) | BrainService implementing AgentRead | `src/activity` · `src/brain` · `src/embed` · `src/error` · `src/hashutil` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`init.rs`](init.rs) | kurultai init --agent cursor/claude/codex/hermes | `src/config` · `src/error` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`interface.rs`](interface.rs) | AgentRead trait | `src/synthesize` · `src/types` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-07-23 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | MCP module: stdio server + init wiring | — | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`server.rs`](server.rs) | MCP tool dispatch (search, ask, ontology_*) | `src/error` · `src/mcp` · `src/ontology` · `src/project` · `src/write_policy` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-16 — indexed this folder (v1 seed)

