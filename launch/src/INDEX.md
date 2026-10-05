---
index: kurultai/v1
folder: launch/src
parent: launch/INDEX.md
updated: 2026-10-05
version: 1
---

# `launch/src`

**Does:** Remotion source — compositions, brand tokens, shared primitives
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

- [`brands/`](brands/INDEX.md) — brand tokens + marks
- [`core/`](core/INDEX.md) — shared Remotion primitives
- [`videos/`](videos/INDEX.md) — per-product video namespaces

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`Root.tsx`](Root.tsx) | Remotion root — registers all compositions | videos/kurultai/launch | index.ts | 2026-10-05 | 1 | 2026-10-05 added |
| [`index.ts`](index.ts) | Bundle entry — registerRoot(Root) | Root.tsx | remotion.config.ts | 2026-10-05 | 1 | 2026-10-05 added |

## Recent

- 2026-10-05 — added (launch film PR #411)
