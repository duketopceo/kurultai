---
title: Trust lanes quality gate
date: 2026-07-25
updated: 2026-08-12
---

# Trust lanes quality gate

Every atom write (MCP `remember`, connector `IndexPipeline`, loopback `POST /ingest`) runs the same synchronous gate:

1. ≥1 non-empty tag (YAML frontmatter `tags:` **or** a dedicated hashtag line such as `#vpn #snipe-it`; YAML wins when both exist)
1. ≥1 non-empty tag (soft labels **never** satisfy this)
2. No other **trusted** atom with the same `content_hash`
3. Cheap heuristics (not an LLM judge):
   - trimmed length &lt; 80 → `low_quality:too_short`
   - thin / boilerplate body → `low_quality:thin`

Failures land in **quarantine** (still stored; default search/ask/list skip them). Promote is explicit (`promote` MCP tool, `POST /api/promote`, `kurultai promote`) and re-runs the gate. Near-duplicate scan runs after `index_all` in the background — never on the `remember` await path.

Headings (`# Title`) and inline `#mentions` in prose are not tags. Soft labels do not satisfy the hard tag gate.
Quarantine writes skip embeddings and clear `atoms_vec` so junk does not pollute the vector index.

Trusted atoms may carry `metadata.quality_score` (`0.0`–`1.0`) for a small post-RRF retrieval boost; `include_quarantine` behavior is unchanged.

Legacy rows migrate as `trust_lane = trusted`.
