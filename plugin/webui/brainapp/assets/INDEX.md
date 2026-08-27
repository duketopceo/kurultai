---
index: kurultai/v1
folder: plugin/webui/brainapp/assets
parent: plugin/webui/brainapp/INDEX.md
updated: 2026-08-27
version: 1
---

# `plugin/webui/brainapp/assets`

**Does:** Hashed Vite bundles + bridge shim — do not edit by hand
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`INDEX.md`](INDEX.md) | INDEX.md | — | — | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |
| [`brain-BKX8gfr4.glb`](brain-BKX8gfr4.glb) | 3D brain model asset | — | webui/brainapp/assets/brain-BUX9E1hZ.js | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |
| [`brain-BUX9E1hZ.js`](brain-BUX9E1hZ.js) | Vite app bundle (3D lattice Brain UI) | — | webui/brainapp/assets/brain-BKX8gfr4.glb · webui/brainapp/assets/fdg.worker-DDygn9NA.js · webui/brainapp/brain.html | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |
| [`brain-DApwbWLK.css`](brain-DApwbWLK.css) | Brain UI styles | — | webui/brainapp/brain.html | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |
| [`bridge.js`](bridge.js) | Fetch/XHR/Worker shim rewriting /api/* to the kproxy daemon route | — | webui/brainapp/brain.html | 2026-08-27 | 2 | 2026-08-27 stop proxying /api/open (host spawn CSRF risk) |
| [`fdg.worker-DDygn9NA.js`](fdg.worker-DDygn9NA.js) | Force-directed graph web worker | — | webui/brainapp/assets/brain-BUX9E1hZ.js | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |

## Recent

- 2026-08-27 — bridge: exclude /api/open from kproxy rewrite set
- 2026-08-27 — indexed (v1 seed)
