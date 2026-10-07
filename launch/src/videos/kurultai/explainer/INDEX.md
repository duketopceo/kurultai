---
index: kurultai/v1
folder: launch/src/videos/kurultai/explainer
parent: launch/src/videos/kurultai/INDEX.md
updated: 2026-10-06
version: 1
---

# `launch/src/videos/kurultai/explainer`

**Does:** concept explainer — "how does an agent remember anything between sessions?" (~80s, diagram-first, user-VO timed). KodeKloud-style stage boxes + dashed draw-on arrows; persistent prompt anchor; VO lands post-draft.
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../../../docs/agent-index.md)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`index.tsx`](index.tsx) | KurultaiExplainer + KurultaiExplainer916 comps, persistent Anchor, BEATS timing map | all scenes + StageFit | Root.tsx | 2026-10-06 | 1 | 2026-10-06 added |
| [`Diagram.tsx`](Diagram.tsx) | StageBox / DArrow / MTag diagram primitives | tokens | — | 2026-10-06 | 1 | 2026-10-06 added |
| [`E1Hook.tsx`](E1Hook.tsx) | Two agents — one learned, one empty | tokens · Diagram | — | 2026-10-06 | 1 | 2026-10-06 added |
| [`E2Context.tsx`](E2Context.tsx) | Context window fills then wipes on session end | tokens · Diagram | — | 2026-10-06 | 1 | 2026-10-06 added |
| [`E3Notes.tsx`](E3Notes.tsx) | notes.txt — query misses, answer was inside | tokens · Diagram | — | 2026-10-06 | 1 | 2026-10-06 added |
| [`E4Embed.tsx`](E4Embed.tsx) | Memory → vector → meaning space, query lights neighbors | tokens · Diagram | — | 2026-10-06 | 1 | 2026-10-06 added |
| [`E5Mechanism.tsx`](E5Mechanism.tsx) | MCP ring, source+trust card, propose→approve | tokens · Diagram · SynthCursor | — | 2026-10-06 | 1 | 2026-10-06 added |
| [`E6End.tsx`](E6End.tsx) | End card — mark, repo, MIT | tokens · SynapseMark | — | 2026-10-06 | 1 | 2026-10-06 added |

## Recent

- 2026-10-06 — added (draft pacing 4800f; re-times to VO timing.json)
