---
index: kurultai/v1
folder: src/security
parent: src/INDEX.md
updated: 2026-09-16
version: 2
---

# `src/security`

**Does:** Paths, redaction, hub keys
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`admin_keys.rs`](admin_keys.rs) | Hub device/admin API keys | `src/error` · `src/hashutil` | `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Security helpers | — | `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` · `src/connect.rs` | 2026-09-16 | 2 | 2026-09-16 re-export agent key file helpers · 2026-08-16 indexed (v1 seed) |
| [`paths.rs`](paths.rs) | Path sandbox / O_NOFOLLOW | `src/error` | `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` | 2026-07-18 | 1 | 2026-08-16 indexed (v1 seed) |
| [`redact.rs`](redact.rs) | Secret redaction | — | `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` | 2026-07-18 | 1 | 2026-08-16 indexed (v1 seed) |
| [`secrets.rs`](secrets.rs) | Secret scanning + key-file storage | `src/error` | `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` · `src/connect.rs` | 2026-09-16 | 3 | 2026-09-16 `agent_key_file_path`/`write_agent_key_file` (`agent-keys/` 0600) for `connect` · 2026-09-15 key file fallback `openrouter.key` (0600) + `write_key_file` (#329) |

## Recent

- 2026-09-16 — `secrets.rs`: `agent_key_file_path`/`write_agent_key_file` — 0600 `agent-keys/<lane>-<codename>-agent-token.key` fallback when `omaseal` is absent
- 2026-09-15 — `secrets.rs`: `api_key_from_keyfile`/`write_key_file`/`key_file_path` (#329)
- 2026-08-16 — indexed this folder (v1 seed)

