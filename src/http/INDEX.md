---
index: kurultai/v1
folder: src/http
parent: src/INDEX.md
updated: 2026-08-29
version: 2
---

# `src/http`

**Does:** Daemon HTTP + Brain UI + MCP SSE
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`auth.rs`](auth.rs) | Hub API-key / admin token gate | `src/hashutil` | `src/daemon/mod.rs` | 2026-08-29 | 2 | 2026-08-29 enforce auth on query routes · 2026-08-16 indexed (v1 seed) |
| [`hub_listen.rs`](hub_listen.rs) | Pure bind × auth start-fail (HUB-3) | `src/http/auth.rs` · `src/error` | `src/http/mod.rs` · `src/main.rs` | 2026-08-29 | 1 | 2026-08-29 HUB-3 hub_listen_decision |
| [`ingest.rs`](ingest.rs) | Opt-in POST /ingest dump | `src/embed` · `src/hashutil` · `src/ingest` · `src/quality` · `src/store` | `src/daemon/mod.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` · `src/query/hybrid.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mcp.rs`](mcp.rs) | MCP HTTP/SSE transport | `src/mcp` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Axum daemon: /api/*, /ui/, optional /mcp SSE | `src/brain` · `src/daemon` · `src/error` · `src/mcp` · `src/metrics` · `src/http/hub_listen.rs` | `src/daemon/mod.rs` | 2026-08-29 | 3 | 2026-08-29 secure unprefixed aliases · 2026-08-29 resolve_listen_socket · 2026-08-16 indexed |
| [`ui.rs`](ui.rs) | Embedded ui/ static + /ui/ slash redirect | — | `src/daemon/mod.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-29 — secure unprefixed query aliases (/search, /ask, etc.) under `HubAuth::ApiKey`
- 2026-08-29 — HUB-3 `hub_listen.rs` bind policy; `serve_with` start-fail before listen
- 2026-08-16 — indexed this folder (v1 seed)

