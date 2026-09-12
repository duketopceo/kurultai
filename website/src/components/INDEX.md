---
index: kurultai/v1
folder: website/src/components
parent: website/src/INDEX.md
updated: 2026-09-11
version: 7
---

# `website/src/components`

**Does:** Dashboard panels
**Up:** [`website/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

- [`chatboard/`](chatboard/INDEX.md) — A2A agent chatboard

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`ActivityPanel.tsx`](ActivityPanel.tsx) | Activity feed | `website/src/components/../api` · `website/src/components/../types` | — | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`HeyPanel.tsx`](HeyPanel.tsx) | Hey API boundary: polls `/api/hey/*`, maps VMs, mounts Chatboard | `../api` · `./chatboard/Chatboard` · `./chatboard/chatboard-mapping` | — | 2026-09-11 | 3 | 2026-09-11 real endpoints wired (unread map, presence map, reaction index, post/react) · 2026-09-06 thin wrapper → chatboard (U3) · dogfood slice |
| [`HumanAccess.tsx`](HumanAccess.tsx) | Human login gate + Access settings panel | `../auth` | — | 2026-09-04 | 1 | 2026-09-04 owner/hub key UX; agent keys separate |
| [`AskPanel.tsx`](AskPanel.tsx) | Ask UI | `website/src/components/../api` | — | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`BrainStage.tsx`](BrainStage.tsx) | 3D stage host + `__kurultaiBrain` metrics handle + BrainHud chip | `website/src/components/../brain/BrainView` · `website/src/components/../types` | — | 2026-09-06 | 2 | 2026-09-07 buildLinks strength = shared-tag count · 2026-09-06 BrainHud live FPS/neurons/synapses chip · 2026-09-06 expose `window.__kurultaiBrain` for `metrics()` · 2026-08-16 indexed (v1 seed) |
| [`CommandStrip.tsx`](CommandStrip.tsx) | Command strip | `website/src/components/../types` · `website/src/components/../api` · `website/src/components/../repoLattice` | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`InspectorPanel.tsx`](InspectorPanel.tsx) | Atom inspector + ontology promote | `../api` · `../types` | — | 2026-09-05 | 2 | 2026-09-05 suggest+promote · 2026-08-16 indexed (v1 seed) |
| [`RepoBrain.tsx`](RepoBrain.tsx) | Repo brain panel | `website/src/components/../brain/BrainView` · `website/src/components/../api` · `website/src/components/../repoLattice` · `website/src/components/../types` | — | 2026-09-05 | 2 | 2026-09-05 fetchGraph with limit for repo list/repo view · 2026-08-16 indexed (v1 seed) |
| [`StatsPanel.tsx`](StatsPanel.tsx) | Stat cards | `website/src/components/../types` | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`DbView.tsx`](DbView.tsx) | Read-only store browser page (`#/db`) | `../api` · `TopBar` | — | 2026-09-08 | 1 | 2026-09-08 added |
| [`ProposalsPanel.tsx`](ProposalsPanel.tsx) | O3 ontology review queue: pending proposals, approve/reject (humans) | `../api` · `../types` | `website/src/App.tsx` | 2026-09-11 | 1 | 2026-09-11 added (#118) |
| [`TopBar.tsx`](TopBar.tsx) | Top bar + Access settings + theme | `../version` · `../auth` · `HumanAccess` | — | 2026-09-04 | 3 | 2026-09-04 Access settings button · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-11 — `ProposalsPanel.tsx`: O3 ontology review queue — pending proposals with approve/reject, refreshes ontology on approve (#118)
- 2026-09-11 — merged: chatboard child + HeyPanel wrapper (U3 of plan 2026-09-05-002) integrated alongside incoming BrainStage HUD/metrics + DbView store browser work
- 2026-09-08 — `DbView.tsx` read-only store browser at `#/db` (tabs, sort, filter, pager); TopBar nav link
- 2026-09-07 — `buildLinks`: pair strength = number of shared tags (drives synapse opacity + layout springs)
- 2026-09-06 — `BrainStage`: `BrainHud` top-right mono readout (FPS · NEURONS · SYNAPSES), hidden on mobile; `.brain-hud` style in styles.css
- 2026-09-06 — `BrainStage`: expose `window.__kurultaiBrain` (metrics console handle), removed on dispose
- 2026-09-05 — RepoBrain fetchGraph limit wiring for repo list/repo view
- 2026-09-05 — BrainStage forbidden zone: secondary chrome tokens/CSS only below cortex; do not restyle BrainView palette/camera
- 2026-09-04 — HumanAccess gate + Access settings in TopBar
- 2026-09-04 — add token reset button and fix /ui/ nav link
- 2026-08-16 — indexed this folder (v1 seed)

