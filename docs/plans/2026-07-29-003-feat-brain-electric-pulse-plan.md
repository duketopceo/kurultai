# feat: Brain electric pulse (soft orbs + synapse zap + hover arc)

**Date:** 2026-07-29  
**Branch:** `cursor/brain-electric-pulse-7a74`  
**Origin:** User `/lfg` A+B+C — replace faceted “cube” look with electrical pulse; keep orbs + connectors.

## Goal

Keep memory orbs and line connectors. Soften node geometry and add electric pulse motion:

- **A** Soft orbs + heartbeat pulse (higher-segment spheres + glow halo)
- **B** Synapse zap on edges (dashed charge flow at rest)
- **C** Hover arc (hot node + connected edges pulse brighter/faster)

## Settled

| Decision | Choice |
|----------|--------|
| Stack | Stay vanilla Three.js in `ui/brain.js` — no React |
| Motion | Respect `prefers-reduced-motion` (static soft orbs, no dash/pulse) |
| Palette | Unchanged black / white / purple |
| Layout | No layout-mode changes in this PR (solar stays on #105) |

## Files

- `ui/brain.js` — geometry, materials, frame animation, hover
- `docs/plans/2026-07-29-003-feat-brain-electric-pulse-plan.md` — this plan

## Verify

- Nodes look smooth (not faceted boxes)
- Edges show traveling dash charge
- Hover brightens node + linked edges and speeds the zap
- Escape / leave clears hover
- Reduced-motion: no pulse/dash motion
