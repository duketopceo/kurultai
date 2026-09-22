---
index: kurultai/v1
folder: website/src
parent: website/INDEX.md
updated: 2026-09-18
version: 9
---

# `website/src`

**Does:** Dashboard React/TS source
**Up:** [`website/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

- [`assets/`](assets/INDEX.md) — Static source assets (GLB etc.)
- [`brain/`](brain/INDEX.md) — 3D view
- [`components/`](components/INDEX.md) — Dashboard panels

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`App.tsx`](App.tsx) | Brain dashboard shell: top pill, brain, floating inspector, right Command rail, bottom Mission Control | `website/src/state` · `website/src/api` · `website/src/perf` · `website/src/components/TopBar` · `website/src/components/BrainStage` · `website/src/components/CommandStrip` · `website/src/components/CommandRail` · `website/src/components/MissionControl` | — | 2026-09-20 | 6 | 2026-09-20 perf wiring: `initPerf`, `setPerfTier`, `tier_load_ms` sample per fetch (#102) · 2026-09-14 pass `onOntologyChanged` to BrainStage board (#316) · 2026-09-13 v2 chrome layout 2026-09-13 v2 chrome layout · 2026-09-11 mount ProposalsPanel (#118) · 2026-09-05 fetchGraph limit by load tier · 2026-09-04 token prompt + logout wiring · 2026-08-16 indexed (v1 seed) |
| [`api.ts`](api.ts) | Fetch wrappers for daemon /api/* | `website/src/types` · `website/src/auth` · `website/src/proposals` | — | 2026-09-14 | 7 | 2026-09-14 `createOntologyEntity`/`createOntologyLink` human-lane fns (#316) · 2026-09-11 O3 proposal fns (#118) 2026-09-11 O3 proposal fns: fetchOntologyProposals/decideOntologyProposal (#118) · 2026-09-11 hey write fns: postHeyMessage/reactHeyMessage/fetchHeyUnread · 2026-09-05 tiered / limited graph query helpers · 2026-09-04 use auth module; 401 → token-invalid · 2026-08-16 indexed (v1 seed) |
| [`auth.ts`](auth.ts) | Human token storage, probe open/locked, auth events | — | `website/src/main.tsx` · `website/src/api.ts` | 2026-09-04 | 1 | 2026-09-04 human access probe + settings events |
| [`assets.d.ts`](assets.d.ts) | Asset module declarations | — | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`main.tsx`](main.tsx) | Vite entry; probe auth; human gate only when locked | `website/src/App` · `website/src/auth` · `website/src/components/HumanAccess` | — | 2026-09-04 | 3 | 2026-09-04 probe open vs locked; HumanLoginGate · 2026-08-16 indexed (v1 seed) |
| [`perf.ts`](perf.ts) | Client perf reporter — nav/tier-load ms, FPS, long tasks, heap → POST /api/metrics/client (30s flush + pagehide) | `website/src/auth` | `website/src/App` · `website/src/components/BrainStage` | 2026-09-20 | 1 | 2026-09-20 added (#102 client half) |
| [`repoLattice.test.ts`](repoLattice.test.ts) | Repo lattice unit tests | `website/src/repoLattice.ts` | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`repoLattice.ts`](repoLattice.ts) | Repo lattice helper | — | `website/src/repoLattice.test.ts` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`state.ts`](state.ts) | App state | `website/src/brain/layout/mode` · `website/src/types` | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`styles.css`](styles.css) | Dashboard CSS (black/white/purple) + v2 chrome grid + ontology board styles | — | — | 2026-09-19 | 9 | 2026-09-19 mobile overflow fix: `.topbar-nav` hidden (dead `.nav-links` selector), strip/mission wrap moved after base rules (cascade order) · 2026-09-19 `kb-expanded` overlay styles + `.kb-panes` base grid + conversation-head flex for Expand button · 2026-09-18 Hey rail spacing fix: add missing `.panel-head` rule, kanban stacks vertically in rail, threads as chip row, `.kb-peer` max-width ellipsis · 2026-09-17 thin dark scrollbars; floating-inspector offset clears command rail · 2026-09-14 `.onto-*` board/menu/dialog styles (#316) · 2026-09-13 v2 chrome layout, chrome accent tokens 2026-09-13 v2 chrome layout, chrome accent tokens · 2026-09-11 `.proposals-*` review queue styles (#118) · 2026-09-04 human login + access panel styles · 2026-08-16 indexed (v1 seed) |
| [`types.ts`](types.ts) | TS types for atoms/graph/ontology | — | `src/brain/mod.rs` · `src/config/loader.rs` · `src/config/mod.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`proposals.ts`](proposals.ts) | O3 proposal type + describeProposal (pure, node-testable) | — | `website/src/api` · `website/src/proposals.test.ts` · `website/src/components/ProposalsPanel` | 2026-09-11 | 1 | 2026-09-11 added (#118) |
| [`proposals.test.ts`](proposals.test.ts) | describeProposal unit tests | `website/src/proposals` | — | 2026-09-11 | 1 | 2026-09-11 added (#118) |
| [`version.ts`](version.ts) | UI version string | — | — | 2026-09-11 | 2 | 2026-09-11 bump to 0.6.0 · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-20 — client perf telemetry (#102): `perf.ts` reporter (nav_ms, tier_load_ms, fps, long_tasks, heap_mb → `POST /api/metrics/client`); `App.tsx` inits + tags tier + reports per-load ms; `BrainStage` BrainHud feeds FPS samples
- 2026-09-19 — mobile overflow fix (`styles.css`): dead `.nav-links` selector replaced with `.topbar-nav`; command-strip/mission wrap rules moved into the ≤960px block *after* their base rules — equal-specificity cascade had silently dropped them
- 2026-09-19 — Hey whole-thread view: Chatboard gains `kb-expanded` portal overlay (Esc/close) + UUID thread labels shorten to 8 chars; HeyPanel active-thread resolution prefers id over name (UUID-named threads caused thread hopping); `styles.css` gains expanded-overlay rules + base `.kb-panes` grid
- 2026-09-18 — Hey rail spacing repair in `styles.css`: `.panel-head` rule added (HeyPanel/ProposalsPanel headers were unstyled), `.hey-kanban` stacks vertically inside the rail (columns no longer clipped), threads render as a wrapping chip row, `.kb-peer` ellipsis works; `ui/` rebuilt
- 2026-09-17 — density attenuation in `brain/BrainView.ts` (sprite/edge/corona/soma all scale with `sizeScale`); thin dark scrollbars + inspector rail offset in `styles.css`; `ui/` rebuilt
- 2026-09-15 — `api.ts` deleteOntologyEntity/deleteOntologyLink; OntologyBoard edge menu + delete-entity confirm + Escape (#320)
- 2026-09-14 — ontology board wiring: `App.tsx` passes `onOntologyChanged`; `api.ts` human-lane fns; `.onto-*` styles (#316)
- 2026-09-13 — v2 chrome layout: `CommandRail.tsx`, `MissionControl.tsx`, `LogsPanel.tsx`, `SettingsPanel.tsx`; `App.tsx` + `styles.css` grid; top pill, right rail, bottom strip, floating inspector, Settings + Logs tabs
- 2026-09-12 — brain/ neuron motif (plan 2026-09-12-001): `dendrite.ts`/`spikes.ts` new — soma facets, dendrite corona sprites, traveling activity-encoded spikes; BrainView wired; +13 tests
- 2026-09-12 — `brain/BrainView.ts` cortex-doctrine rebalance: pale-white nodes, tighter halos, per-synapse shimmer + zap tick
- 2026-09-11 — O3 review queue: `proposals.ts` (type + describeProposal), `proposals.test.ts`, api.ts proposal fns, `App.tsx` mounts `ProposalsPanel`, `.proposals-*` styles (#118)
- 2026-09-05 — tiered graph fetch: api.ts GraphQuery + App.tsx load-tier limits
- 2026-09-04 — human login: probe open/locked, Access settings, agent keys left alone
- 2026-09-04 — token prompt + 401 auth clear flow; TopBar logout/changeset
- 2026-08-16 — indexed this folder (v1 seed)
