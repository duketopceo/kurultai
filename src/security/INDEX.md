---
index: kurultai/v1
folder: src/security
parent: src/INDEX.md
updated: 2026-08-16
version: 1
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
| [`mod.rs`](mod.rs) | Security helpers | — | `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`paths.rs`](paths.rs) | Path sandbox / O_NOFOLLOW | `src/error` | `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` | 2026-07-18 | 1 | 2026-08-16 indexed (v1 seed) |
| [`redact.rs`](redact.rs) | Secret redaction | — | `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` | 2026-07-18 | 1 | 2026-08-16 indexed (v1 seed) |
| [`secrets.rs`](secrets.rs) | Secret scanning | `src/error` | `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` | 2026-07-18 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-16 — indexed this folder (v1 seed)

