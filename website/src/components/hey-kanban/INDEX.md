---
index: kurultai/v1
folder: website/src/components/hey-kanban
parent: website/src/components/INDEX.md
updated: 2026-09-16
version: 1
---

# `website/src/components/hey-kanban`

**Does:** Hey kanban view — messages with a leading `[lane]` token rendered as draggable cards
**Up:** [`website/src/components/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`HeyKanban.tsx`](HeyKanban.tsx) | Kanban board: headless `@mshafiqyajid/react-kanban` columns, card move → lane-token rewrite + PATCH | `./kanbanMapping` · `@mshafiqyajid/react-kanban` | — | 2026-09-16 | 1 | 2026-09-16 indexed (v1 seed) |
| [`kanbanMapping.ts`](kanbanMapping.ts) | Lane token parse/rewrite mapping (message content ↔ column) | — | — | 2026-09-16 | 1 | 2026-09-16 indexed (v1 seed) |
| [`kanbanMapping.test.ts`](kanbanMapping.test.ts) | Lane mapping unit tests | `./kanbanMapping` | — | 2026-09-16 | 1 | 2026-09-16 indexed (v1 seed) |

## Recent

- 2026-09-16 — indexed this folder (v1 seed, #331)
