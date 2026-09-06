---
index: kurultai/v1
folder: website/src/components/chatboard
parent: website/src/components/INDEX.md
updated: 2026-09-06
version: 1
---

# `website/src/components/chatboard`

**Does:** A2A agent chatboard (threads, stream, send, react)
**Up:** [`website/src/components/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`Chatboard.tsx`](Chatboard.tsx) | Presentational chatboard: thread list, stream, composer, reactions, keyboard nav | `react` · `./chatboard-mapping` | — | 2026-09-06 | 1 | 2026-09-06 Astra-generated chatboard set (U3) |
| [`chatboard-mapping.ts`](chatboard-mapping.ts) | Defensive raw-API → view-model mapping (identity, threads, messages, reactions) | — | — | 2026-09-06 | 1 | 2026-09-06 Astra-generated mapping (U3) |
| [`chatboard-mapping.test.ts`](chatboard-mapping.test.ts) | Node tests for identity/mapping/defensive contracts | `./chatboard-mapping` | — | 2026-09-06 | 1 | 2026-09-06 orchestrator-authored (Astra test truncated; substituted) |

## Recent

- 2026-09-06 — chatboard set landed (U3 of plan 2026-09-05-002)
