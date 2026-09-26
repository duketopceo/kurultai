---
index: kurultai/v1
folder: design-lab
updated: 2026-09-26
version: 1
---

# `design-lab`

**Does:** batch-model UI design generation scratch — committed prompts, gitignored outputs (`out/`)
**Up:** [`../INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`README.md`](README.md) | How the batch run works: prompts committed, `out/` scratch, runner usage, scoring rubric | `scripts/ui-batch.mjs` | OPENROUTER_API_KEY | 2026-09-26 | 1 | v1 seed |
| [`prompts/tokens.md`](prompts/tokens.md) | Design-token generation prompt (tokens.css + tokens.md contract) | docs/design/ui-definitions.md | — | 2026-09-26 | 1 | v1 seed |
| [`prompts/chrome.md`](prompts/chrome.md) | TopBar + CommandStrip generation prompt | docs/design/ui-definitions.md §3.1–3.2 | — | 2026-09-26 | 1 | v1 seed |
| [`prompts/worksurface.md`](prompts/worksurface.md) | Below-brain tabbed console generation prompt (Hey/Pulse/Ontology/Ask/Store) | docs/design/ui-definitions.md §3.5–3.8 | — | 2026-09-26 | 1 | v1 seed |
| [`prompts/inspector.md`](prompts/inspector.md) | Floating Inspector panel generation prompt | docs/design/ui-definitions.md §3.4 | — | 2026-09-26 | 1 | v1 seed |

## Recent

- 2026-09-26 — folder created (plan U3): 4 surface prompts + runner contract; `out/` gitignored
