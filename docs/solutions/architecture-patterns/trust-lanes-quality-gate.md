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

## Shared-store write containment (`KURULTAI_FEATURE_SHARED_WRITE=1`, default off)

The gate above is a **quality** control, not a **trust** control: attacker-controlled
frontmatter `tags:` clears the tag check, so content injected into a session can reach
the trusted lane on its own merits.

When several agent sessions share one store as one unix user, set
`KURULTAI_FEATURE_SHARED_WRITE=1`. Then:

- Agent-reachable writes (`remember`, `POST /ingest`, daemon HTTP) are forced to
  **quarantine** regardless of gate outcome, with `quarantine_reason =
  agent_write_containment`. They are never in another session's default search.
- Every write is stamped with `metadata.agent_id`, `metadata.project_id` and
  `metadata.write_transport`, so a poisoned session's atoms can be attributed and
  bulk-revoked. `agent_id` is **self-asserted** — provenance, not authorization.
- Only `kurultai promote` (actor `cli`) may move an atom quarantine → trusted. The
  `promote` MCP tool and `POST /api/promote` are refused.
- `POST /api/promote` and `POST /api/touch` additionally require
  `Authorization: Bearer $KURULTAI_ADMIN_TOKEN`, and return 503 if that env is unset
  (fail closed). Without the flag they behave exactly as before.

Default (flag off) behaviour is unchanged for the single-operator install.
Quarantine writes skip embeddings and clear `atoms_vec` so junk does not pollute the vector index.

Trusted atoms may carry `metadata.quality_score` (`0.0`–`1.0`) for a small post-RRF retrieval boost; `include_quarantine` behavior is unchanged.

Legacy rows migrate as `trust_lane = trusted`.
