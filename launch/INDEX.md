---
index: kurultai/v1
folder: launch
parent: INDEX.md
updated: 2026-10-05
version: 1
---

# `launch`

**Does:** Remotion launch film project — 'One brain. Every agent.' (~36s, 1080p + square + readme loop)
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

- [`public/`](public/INDEX.md) — fonts + brand SVG
- [`scripts/`](scripts/INDEX.md) — render/qc/finish pipeline
- [`src/`](src/INDEX.md) — compositions + primitives

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`package.json`](package.json) | Remotion deps + render scripts | — | package-lock.json | 2026-10-05 | 1 | 2026-10-05 added |
| [`package-lock.json`](package-lock.json) | Locked dep tree | package.json | — | 2026-10-05 | 1 | 2026-10-05 added |
| [`remotion.config.ts`](remotion.config.ts) | Remotion config — codec, fps, entry | src/index.ts | — | 2026-10-05 | 1 | 2026-10-05 added |
| [`tsconfig.json`](tsconfig.json) | TS config for compositions | — | all src/ | 2026-10-05 | 1 | 2026-10-05 added |
| [`.gitignore`](.gitignore) | out/ + node_modules ignore | — | — | 2026-10-05 | 1 | 2026-10-05 added |

## Recent

- 2026-10-05 — added (launch film PR #411)
