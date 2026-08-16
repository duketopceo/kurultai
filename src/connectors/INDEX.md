---
index: kurultai/v1
folder: src/connectors
parent: src/INDEX.md
updated: 2026-08-16
version: 1
---

# `src/connectors`

**Does:** Source ingest adapters
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`appflowy.rs`](appflowy.rs) | AppFlowy connector | `src/connectors` · `src/error` · `src/types` | `src/app/context.rs` · `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` | 2026-07-18 | 1 | 2026-08-16 indexed (v1 seed) |
| [`dayflow.rs`](dayflow.rs) | Dayflow connector | `src/connectors` · `src/error` · `src/hashutil` · `src/security` · `src/types` | `src/app/context.rs` · `src/connectors/appflowy.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`github.rs`](github.rs) | GitHub filesystem connector | `src/connectors` · `src/error` · `src/hashutil` · `src/security` · `src/types` | `src/app/context.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`inbox.rs`](inbox.rs) | Inbox dump adapter (config-not-code) | `src/connectors` · `src/error` · `src/ingest` · `src/security` · `src/types` | `src/app/context.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/json.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`json.rs`](json.rs) | JSON file/folder ingest | `src/connectors` · `src/error` · `src/ingest` · `src/security` · `src/types` | `src/app/context.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`markdown.rs`](markdown.rs) | Markdown folder ingest + hashtag-line tags | `src/connectors` · `src/error` · `src/ingest` · `src/security` · `src/types` | `src/app/context.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Connector trait (poll / full_sync) | `src/error` · `src/types` | `src/app/context.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`pond.rs`](pond.rs) | Pond connector | `src/connectors` · `src/error` · `src/hashutil` · `src/types` | `src/app/context.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`registry.rs`](registry.rs) | Named connector registry | `src/connectors` · `src/error` · `src/types` | `src/app/context.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/inbox.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-16 — indexed this folder (v1 seed)

