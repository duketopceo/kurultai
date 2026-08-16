---
title: "chore: Wave G sequence — Railway hub, then agent IDs, then desktop wrap"
date: 2026-08-15
type: chore
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: legacy-requirements
execution: docs
authority: "User /ce-plan 2026-08-15 — Railway + machines-to-one + agent IDs + desktop wrap; stop-if-out-of-hand"
depth: standard
origin: "docs/brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md · docs/plans/phase-6-next-work-orders.md · GBrain topologies + company-brain"
---

# Wave G sequence — Railway hub, then agent IDs, then desktop wrap

**Target repo:** `duketopceo/kurultai`
**Base:** `main` after HUB-1 (#178) + HUB-2 (#176) + API-key scaffold (#190) + write-policy (#221)
**Process:** PR-only. Work **one plan at a time**. This file is the queue, not an LFG.

## Do not stop. Narrow.

Wanting one cloud brain that every machine talks to is the right product. Treating Railway, real agent IDs, and a desktop app as three equal first ships is the overreach.

GBrain does not ship those together. It ships **one hosted brain**, then **thin clients**, then identity as a wrap.

Kurultai already designed the same shape. Do not invent a fourth product.

## How GBrain actually does this (and its memory)

GBrain memory is **markdown in a git repo**. That repo is the system of record. Postgres / PGLite / Supabase is the search index. Delete in git becomes a soft-delete in the DB. You can clone the repo to a second machine and the brain is still yours.

Three deployment shapes (`docs/architecture/topologies.md`):

1. **Single brain** — one machine, local PGLite or remote Supabase. Default.
2. **Cross-machine thin client** — one brain-host runs `gbrain serve --http`. Other machines have **no local DB**. They talk HTTP MCP + OAuth. `gbrain sync` / embed / serve are refused on the client.
3. **Split-engine** — per-worktree local code index + remote artifact brain. Agent picks the alias. No magic router.

Company brain (`docs/tutorials/company-brain.md`):

- Same install, not a different product.
- One Postgres. Multiple **sources** (shared wiki, customers, internal).
- One **OAuth client per person / agent**: `--source` is write authority, `--federated-read` is read scope.
- Thin-client laptops point at `https://brain.example.com/mcp`.
- Not multi-tenant SaaS. Self-host (Render / Railway class). Garry’s published recipe is under $100/mo for ~25 people.
- Personal brain can stay local. Team brain is a **mount**, a second database.

Kurultai’s equivalent, already in-tree:

| GBrain | Kurultai today |
|--------|----------------|
| Git markdown SoR | Connectors + atoms (not a git SoR) |
| PGLite personal / Postgres company | SQLite personal forever / Postgres hub (`--features postgres`, `KURULTAI_FEATURE_HUB`) |
| Thin client, no local DB | Local SQLite stays; R8 merge (local + hub in one ask) is **not shipped** |
| OAuth client per person | Self-asserted `KURULTAI_AGENT_ID` + hashed API-key list in env (#190). Not issued/revoked identities. |
| `gbrain serve --bind 0.0.0.0 --public-url` | Bind helpers exist (`KURULTAI_HUB_BIND`, `KURULTAI_HUB_AUTH`). No start-fail if public + no auth. No Railway recipe. |
| `gbrain whoami` + admin dashboard | `quality_audit.actor` stamps `mcp:<id>`. No queryable “who wrote what when” API. |
| Desktop = Claude Desktop pointing at remote MCP | Embedded `/ui/` in the daemon. No packaged desktop app. |

**Trap GBrain also refuses:** do not put the personal SQLite file in the cloud. Kurultai already errors personal upserts on `PostgresStore` (AE4). Keep that.

## What already shipped (do not rebuild)

- HUB-1 visibility `personal | team | company` — #178 closed
- HUB-2 `PostgresStore` — #176 closed
- Bearer API-key middleware on `/api/*` when `KURULTAI_HUB_AUTH=api_key` — #190
- Write containment + self-asserted `agent_id` stamp — #221, `src/write_policy.rs`
- Offline `.kurultai` export/import for device handoff
- Embedded Brain UI at `/ui/`

## Open issues this sequence owns

| Order | Plan | Issue | Ship |
|------:|------|-------|------|
| 1 | [001 HUB-3 Railway transport](2026-08-15-001-feat-hub3-railway-transport-plan.md) | [#177](https://github.com/duketopceo/kurultai/issues/177) | Public or Tailscale bind, refuse unauth public, Dockerfile + Railway recipe, machines talk to one hub |
| 2 | [002 HUB-4 agent IDs + write log](2026-08-15-002-feat-hub4-agent-ids-write-log-plan.md) | [#179](https://github.com/duketopceo/kurultai/issues/179) | Issue/revoke device+agent keys, `team_id` filter, queryable write log |
| 3 | [003 desktop Brain UI wrap](2026-08-15-003-feat-desktop-brain-ui-wrap-plan.md) | none (UI was out of the hub brainstorm) | Thin Tauri/Wails window over `/ui/`. Last. Does not block Railway. |

HUB-5 ingest tagging (#180) stays after 002. Do not steal it into 001.

## Explicitly out

- Multi-tenant SaaS (many unrelated orgs on one Kurultai-operated platform)
- Password / session / JWT accounts in v1 (brainstorm R5)
- Dumping `store.db` onto Railway
- Desktop wrap as a prerequisite for the hub
- Replacing `.kurultai` export/import (keep for offline solo)

## Next LFG

Start **001 only**. 002 and 003 stay docs until 001 is on `main`.
