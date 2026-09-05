---
index: kurultai/v1
folder: website/src/components
parent: website/src/INDEX.md
updated: 2026-09-05
version: 3
---

# `website/src/components`

**Does:** Dashboard panels
**Up:** [`website/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`ActivityPanel.tsx`](ActivityPanel.tsx) | Activity feed | `website/src/components/../api` · `website/src/components/../types` | — | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`HeyPanel.tsx`](HeyPanel.tsx) | Agent hey board panel (non-Brain) | `../api` | — | 2026-09-04 | 1 | dogfood slice |
| [`HumanAccess.tsx`](HumanAccess.tsx) | Human login gate + Access settings panel | `../auth` | — | 2026-09-04 | 1 | 2026-09-04 owner/hub key UX; agent keys separate |
| [`AskPanel.tsx`](AskPanel.tsx) | Ask UI | `website/src/components/../api` | — | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`BrainStage.tsx`](BrainStage.tsx) | 3D stage host | `website/src/components/../brain/BrainView` · `website/src/components/../types` | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`CommandStrip.tsx`](CommandStrip.tsx) | Command strip | `website/src/components/../types` · `website/src/components/../api` · `website/src/components/../repoLattice` | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`InspectorPanel.tsx`](InspectorPanel.tsx) | Atom inspector + ontology promote | `../api` · `../types` | — | 2026-09-05 | 2 | 2026-09-05 suggest+promote · 2026-08-16 indexed (v1 seed) |
| [`RepoBrain.tsx`](RepoBrain.tsx) | Repo brain panel | `website/src/components/../brain/BrainView` · `website/src/components/../api` · `website/src/components/../repoLattice` · `website/src/components/../types` | — | 2026-09-05 | 2 | 2026-09-05 fetchGraph with limit for repo list/repo view · 2026-08-16 indexed (v1 seed) |
| [`StatsPanel.tsx`](StatsPanel.tsx) | Stat cards | `website/src/components/../types` | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`TopBar.tsx`](TopBar.tsx) | Top bar + Access settings + theme | `../version` · `../auth` · `HumanAccess` | — | 2026-09-04 | 3 | 2026-09-04 Access settings button · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-05 — RepoBrain fetchGraph limit wiring for repo list/repo view
- 2026-09-05 — BrainStage forbidden zone: secondary chrome tokens/CSS only below cortex; do not restyle BrainView palette/camera
- 2026-09-04 — HumanAccess gate + Access settings in TopBar
- 2026-09-04 — add token reset button and fix /ui/ nav link
- 2026-08-16 — indexed this folder (v1 seed)

