---
title: Trust lanes quality gate
date: 2026-07-25
---

# Trust lanes quality gate

Every atom write (MCP `remember` and connector `IndexPipeline`) runs the same synchronous gate:

1. ≥1 non-empty tag (YAML frontmatter `tags:` **or** a dedicated hashtag line such as `#vpn #snipe-it`; YAML wins when both exist)
2. No other **trusted** atom with the same `content_hash`

Failures land in **quarantine** (still stored; default search/ask/list skip them). Promote is explicit (`promote` MCP tool, `POST /api/promote`, `kurultai promote`) and re-runs the gate. Near-duplicate scan runs after `index_all` in the background — never on the `remember` await path.

Headings (`# Title`) and inline `#mentions` in prose are not tags. Soft labels do not satisfy the hard tag gate.

Legacy rows migrate as `trust_lane = trusted`.
