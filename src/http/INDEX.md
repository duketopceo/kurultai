---
index: kurultai/v1
folder: src/http
parent: src/INDEX.md
updated: 2026-08-16
version: 1
---

# `src/http`

**Does:** Daemon HTTP + Brain UI + MCP SSE
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`auth.rs`](auth.rs) | Hub API-key / admin token gate | `src/hashutil` | `src/daemon/mod.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`ingest.rs`](ingest.rs) | Opt-in POST /ingest dump | `src/embed` · `src/hashutil` · `src/ingest` · `src/quality` · `src/store` | `src/daemon/mod.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` · `src/query/hybrid.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mcp.rs`](mcp.rs) | MCP HTTP/SSE transport | `src/mcp` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Axum daemon: /api/*, /ui/, optional /mcp SSE | `src/brain` · `src/daemon` · `src/error` · `src/mcp` · `src/metrics` | `src/daemon/mod.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`ui.rs`](ui.rs) | Embedded ui/ static + /ui/ slash redirect | — | `src/daemon/mod.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-16 — indexed this folder (v1 seed)

