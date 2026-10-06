---
index: kurultai/v1
folder: launch/src/core
parent: launch/src/INDEX.md
updated: 2026-10-05
version: 1
---

# `launch/src/core`

**Does:** Reusable Remotion primitives shared by every scene (Argus core port)
**Up:** [`INDEX.md`](../../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`Fonts.tsx`](Fonts.tsx) | @remotion/google-fonts + static woff2 loader | launch/public/fonts | Root | 2026-10-05 | 1 | 2026-10-05 added |
| [`Stage.tsx`](Stage.tsx) | Frame stage — canvas bg, letterbox, grain overlay | tokens.ts | all scenes | 2026-10-05 | 1 | 2026-10-05 added |
| [`KineticType.tsx`](KineticType.tsx) | Word/char-staggered kinetic type primitive | tokens.ts · Fonts.tsx | hook, beats, end card | 2026-10-05 | 1 | 2026-10-05 added |
| [`SynthCursor.tsx`](SynthCursor.tsx) | Synthetic cursor with easing trails | tokens.ts | demo scenes | 2026-10-05 | 1 | 2026-10-05 added |
| [`TerminalScene.tsx`](TerminalScene.tsx) | Terminal chrome + typed-line renderer | Fonts.tsx | Scene02Wire · Scene07Onboard | 2026-10-05 | 1 | 2026-10-05 added |
| [`UIZoom.tsx`](UIZoom.tsx) | Camera-zoom wrapper for UI-callout beats | tokens.ts | Scene04Search · Scene05Hey | 2026-10-05 | 1 | 2026-10-05 added |
| [`Brain3D.tsx`](Brain3D.tsx) | R3F 3D brain — cortex hull, emissive neurons, edge growth/pending/supersede states | tokens.ts · @remotion/three | v3 scenes | 2026-10-05 | 1 | 2026-10-05 added |

## Recent

- 2026-10-05 — added (launch film PR #411)
- 2026-10-05 — Brain3D added for v3 3D-brain cut (WebGL probe-verified headless on Asahi)
