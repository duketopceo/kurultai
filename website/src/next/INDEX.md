---
index: kurultai/v1
folder: website/src/next
parent: website/src/INDEX.md
updated: 2026-11-07
version: 3
---

# `website/src/next`

**Does:** ui-next app — design-lab winner components stitched onto `api.ts` (plan 2026-09-25-001 U4); parallel surface, does not replace `src/App.tsx` yet
**Up:** [`website/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

- [`gen/`](gen/) — extracted winning variants (one dir per surface: tokens, chrome, brain-hero, inspector, worksurface, chatboard, ontology-board, settings-access, store-db). Self-scoped CSS per surface; App owns the only cross-surface sheet.

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`App.tsx`](App.tsx) | ui-next shell: TopBar+CommandStrip+BrainHero(BrainStage)+WorkSurface+Inspector+SettingsPanel, all wired to `../api` | `../api` · `../types` · `../perf` · `../components/BrainStage` · `../components/hey-kanban/kanbanMapping` · `../version` | — | 2026-11-09 | 3 | 2026-11-09 fetches MCP flag; key-save copy no longer asks to restart · 2026-11-07 OpenRouter connect wiring (`?or_connect=start|1`) · 2026-09-27 U4 stitch: adapters for hey/pulse/ontology/ask/store, search, tier, inspector promote |
| [`V2Shell.tsx`](V2Shell.tsx) | ONE shell for Brain v2: top bar (search, Brain/Repos/Ontology switch, panel, status, menu with 32-bit), canvas views, one resizable right panel (Inspect/Hey/Pulse/Ask/Store), Ctrl+K palette | `website/src/brain/v2/palette.ts` · `website/src/brain/v2/why.ts` · `website/src/types` | `website/src/next/App.tsx` | 2026-11-09 | 4 | 2026-11-09 "Connect an agent" menu item + MCP panel · 2026-11-07 menu: Connect OpenRouter item · 2026-11-08 unified shell |
| [`v2shell.css`](v2shell.css) | V2Shell styles on tokens.css (ember only on the path chain) | `website/src/next/tokens.css` | `website/src/next/V2Shell.tsx` | 2026-11-09 | 3 | 2026-11-09 `.v2-palette pre` wrap · 2026-11-08 segmented switch, slide-over panel, 32-bit pixel theme · 2026-11-07 added |
| [`bridge.css`](bridge.css) | Font import, `--k-*`/`--kt-*` global var bridge onto `tokens.css`, app-shell layout | `./tokens.css` | `gen/chrome` · `gen/brain-hero` | 2026-09-27 | 1 | 2026-09-27 added |
| [`main.tsx`](main.tsx) | ui-next Vite entry: auth probe + gen HumanAccess gate | `../auth` · `./App` · `gen/settings-access` | — | 2026-09-27 | 1 | 2026-09-27 added |
| [`tokens.css`](tokens.css) | Design tokens (design-lab tokens winner: opus-5.5 v2) — canvas/accent/text/surface scale | — | `bridge.css` · `gen/*` | 2026-09-27 | 1 | 2026-09-27 extracted from `design-lab/out/tokens/anthropic-claude-opus-5.5-2.md` |

## Recent

- 2026-11-09 — `v2shell.css`: pre wrapping inside palette dialog
- 2026-11-09 — `V2Shell.tsx`: Connect an agent menu item and panel (endpoint + Claude Code/JSON snippets)
- 2026-11-09 — `App.tsx`: mcpInfo state + no-restart copy
- 2026-11-07 — `App.tsx`: v2 default, what's-new badge, tier mid then all (50k)
- 2026-11-07 — App mounts `repos/RepoFloorplan` from the v2 palette
- 2026-11-08 — panel re-centres the brain (inset); worksurface buttons keep their own colours; Light is the default theme
- 2026-11-08 — Light/Dark/System theme + reduce-effects in the menu; shell restyled monochrome
- 2026-11-07 — `V2Shell` minimal chrome behind `?brain=v2` (App.tsx early-returns it)
