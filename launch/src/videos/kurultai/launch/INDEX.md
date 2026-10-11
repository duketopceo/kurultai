---
index: kurultai/v1
folder: launch/src/videos/kurultai/launch
parent: launch/src/videos/kurultai/INDEX.md
updated: 2026-10-05
version: 1
---

# `launch/src/videos/kurultai/launch`

**Does:** The 8-beat launch film — scenes + storyboard
**Up:** [`INDEX.md`](../../../../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../../../docs/agent-index.md)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`index.tsx`](index.tsx) | Composition wiring — KurultaiLaunch 1080p, Square 1:1, ReadmeLoop 7.5s | all Scene*.tsx | Root.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`storyboard.md`](storyboard.md) | Beat-by-beat board with timing + verbatim copy source | — | all scenes | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene01Hook.tsx`](Scene01Hook.tsx) | Cold open — 'Every agent forgets.' kinetic hook | core/* | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene01Silos.tsx`](Scene01Silos.tsx) | Silo problem beat — five agents, five brains | KineticType · Stage | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene02Fleet.tsx`](Scene02Fleet.tsx) | Fleet ring — agents connect to one brain | Stage · SynthCursor | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene02Wire.tsx`](Scene02Wire.tsx) | Wiring beat — init --agent + mcp.json terminal | TerminalScene | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene03Atom.tsx`](Scene03Atom.tsx) | Atom lifecycle — quarantine→trusted flip | Stage · tokens | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene04Search.tsx`](Scene04Search.tsx) | Cited search + gap-aware ask beat | UIZoom · KineticType | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene05Hey.tsx`](Scene05Hey.tsx) | Hey board — meridian-demo thread | UIZoom · Stage | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene05SelfWire.tsx`](Scene05SelfWire.tsx) | Self-wiring beat — agent writes its own memory | TerminalScene · Stage | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene06Ontology.tsx`](Scene06Ontology.tsx) | Ontology graph + promote_atom proposal | Stage · tokens | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene07Mcp.tsx`](Scene07Mcp.tsx) | 8-tool MCP grid | KineticType · Stage | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene07Onboard.tsx`](Scene07Onboard.tsx) | kurultai init --docs onboarding terminal | TerminalScene | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Scene08EndCard.tsx`](Scene08EndCard.tsx) | End card — 'One brain. Every agent.' + mark | BrainMark · KineticType | index.tsx | 2026-10-05 | 1 | 2026-10-05 added |
| [`Probe.tsx`](Probe.tsx) | WebGL canary comp — verifies headless @remotion/three renders on this stack | @remotion/three | Root.tsx | 2026-10-05 | 1 | 2026-10-05 added (v3 gate) |

## Recent

- 2026-10-05 — Probe.tsx WebGL canary added for v3
- 2026-10-05 — added (launch film PR #411)
