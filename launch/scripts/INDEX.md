---
index: kurultai/v1
folder: launch/scripts
parent: launch/INDEX.md
updated: 2026-10-05
version: 1
---

# `launch/scripts`

**Does:** Render/QC/finish shell scripts for the launch film pipeline
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`render.sh`](render.sh) | Remotion render driver (all three outputs) | launch/package.json | out/*.mp4 | 2026-10-05 | 1 | 2026-10-05 added |
| [`qc.sh`](qc.sh) | Post-render QC checks (duration, resolution, size) | render.sh | out/*.mp4 | 2026-10-05 | 1 | 2026-10-05 added |
| [`finish.sh`](finish.sh) | Final packaging/copy step for shipped files | qc.sh | out/ | 2026-10-05 | 1 | 2026-10-05 added |

## Recent

- 2026-10-05 — added (launch film PR #411)
