---
index: kurultai/v1
folder: website/src/next
parent: website/src/INDEX.md
updated: 2026-09-27
version: 1
---

# `website/src/next`

**Does:** ui-next app — design-lab winner components stitched onto `api.ts` (plan 2026-09-25-001 U4); parallel surface, does not replace `src/App.tsx` yet
**Up:** [`website/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

- [`gen/`](gen/) — extracted winning variants (one dir per surface: tokens, chrome, brain-hero, inspector, worksurface, chatboard, ontology-board, settings-access, store-db). Self-scoped CSS per surface; App owns the only cross-surface sheet.

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`App.tsx`](App.tsx) | ui-next shell: TopBar+CommandStrip+BrainHero(BrainStage)+WorkSurface+Inspector+SettingsPanel, all wired to `../api` | `../api` · `../types` · `../perf` · `../components/BrainStage` · `../components/hey-kanban/kanbanMapping` · `../version` | — | 2026-09-27 | 1 | 2026-09-27 U4 stitch: adapters for hey/pulse/ontology/ask/store, search, tier, inspector promote |
| [`bridge.css`](bridge.css) | Font import, `--k-*`/`--kt-*` global var bridge onto `tokens.css`, app-shell layout | `./tokens.css` | `gen/chrome` · `gen/brain-hero` | 2026-09-27 | 1 | 2026-09-27 added |
| [`main.tsx`](main.tsx) | ui-next Vite entry: auth probe + gen HumanAccess gate | `../auth` · `./App` · `gen/settings-access` | — | 2026-09-27 | 1 | 2026-09-27 added |
| [`tokens.css`](tokens.css) | Design tokens (design-lab tokens winner: opus-5.5 v2) — canvas/accent/text/surface scale | — | `bridge.css` · `gen/*` | 2026-09-27 | 1 | 2026-09-27 extracted from `design-lab/out/tokens/anthropic-claude-opus-5.5-2.md` |
