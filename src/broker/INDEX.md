---
index: kurultai/v1
folder: src/broker
parent: src/INDEX.md
updated: 2026-09-30
version: 1
---

# `src/broker`

**Does:** per-device broker daemon (`kurultai broker`) — holds the single upstream seat session; agents board over loopback with minted `sess_*` keys (plan `docs/plans/2026-09-28-001`)
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`board.rs`](board.rs) | `POST /board` mint + `/whoami` + `/revoke` for `sess_*` chat keys | `src/broker/registry.rs` · `src/error` | `src/broker/server.rs` · `src/broker/relay.rs` | 2026-09-30 | 1 | 2026-09-30 added (U2) |
| [`mod.rs`](mod.rs) | Broker module root + security invariants doc | — | — | 2026-09-30 | 1 | 2026-09-30 added |
| [`registry.rs`](registry.rs) | broker.db SQLite registry — hashed `sess_*` → (agent, chat_id, chat_name, instance_id, device) | `src/error` · `rusqlite` | `src/broker/board.rs` · `src/broker/relay.rs` | 2026-09-30 | 1 | 2026-09-30 added (U2); `default_db_path` beside configured store |
| [`relay.rs`](relay.rs) | `POST /mcp` relay → upstream `/mcp` with seat Bearer + `X-Kurultai-{agent,chat,session,device}` stamps | `src/broker/board.rs` · `src/security` · `reqwest` | `src/broker/server.rs` | 2026-09-30 | 1 | 2026-09-30 added (U3) |
| [`server.rs`](server.rs) | Loopback-only axum boot: `/health` `/status` + board/relay merge; fails fast on missing upstream token; optional unix socket | `src/broker/registry.rs` · `src/security` · `src/connect` | `src/main.rs` | 2026-09-30 | 1 | 2026-09-30 added (U1); unix socket listener |

## Recent

- 2026-09-30 — U1–U3: `kurultai broker` daemon, `sess_*` registry + board, `/mcp` relay with stamped identity; upstream `/mcp` accepts seat tokens (full surface) vs shared secret (read-only)
