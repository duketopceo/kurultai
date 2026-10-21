---
index: kurultai/v1
folder: docs
parent: INDEX.md
updated: 2026-10-20
version: 12
---

# `docs`

**Does:** Product + agent docs
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

- [`agent-zero/`](agent-zero/INDEX.md) — v1 Agent Zero issue pack (historical)
- [`brainstorms/`](brainstorms/INDEX.md) — Requirements / research notes
- [`design/`](design/INDEX.md) — UI/UX rebuild definitions + research bases (plan 2026-09-25-001)
- [`deploy/`](deploy/INDEX.md) — Hub / Railway operator recipes
- [`eval/`](eval/INDEX.md) — Eval notes
- [`ideas/`](ideas/INDEX.md) — Parked ideas
- [`research/`](research/INDEX.md) — Landscape research (agent-memory systems)
- [`plans/`](plans/INDEX.md) — LFG plans and work-order packs (live queue: phase-6-next)
- [`residual-review-findings/`](residual-review-findings/INDEX.md) — Review residual records
- [`screenshots/`](screenshots/INDEX.md) — Visual captures referenced by docs/PRs
- [`solutions/`](solutions/INDEX.md) — Documented past solutions

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`AGENT_CONNECTION_PROMPT.md`](AGENT_CONNECTION_PROMPT.md) | Copy-paste agent prompt for reading/writing the Kurultai brain (local MCP + hosted solo/hub instances — `connect` easy path + `hub key` admin path) | — | — | 2026-10-20 | 3 | 2026-10-20 hub adoption U6 rewrite: `connect <api-hub>` join (team picker + mint-at-approve), per-agent remote MCP wiring (cursor/claude url+Bearer, codex/hermes `mcp --broker` bridge), `hub key issue/list/revoke` + `/api/hub/*` admin surface; `login` marked solo-only · 2026-10-06 §B corrected: seat-token bearer on /mcp grants full write surface, `api-*` hostnames bearer-only · 2026-09-29 ported from archived kurultai-private |
| [`AGENT_SETUP_PROMPT.md`](AGENT_SETUP_PROMPT.md) | Copy-paste agent setup prompt | — | — | 2026-10-20 | 3 | 2026-10-20 U6 clap parity: license MIT→AGPL-3.0-only, full MCP tool list (+recall/ontology_propose(s)/hey_*), `connect` hosted-join pointer, `inbox` kind · 2026-10-19 install tag v0.4.0 → v0.7.0 · 2026-08-16 indexed (v1 seed) |
| [`PROJECT_SCOPING.md`](PROJECT_SCOPING.md) | Project scoping notes | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`agent-index.md`](agent-index.md) | Agent INDEX.md schema, skip list, update ritual | — | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`crew-instance-node3.md`](crew-instance-node3.md) | Crew instance notes | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mac-dev.md`](mac-dev.md) | macOS dev setup | — | — | 2026-10-19 | 2 | 2026-10-19 install tag v0.4.1 → v0.7.0 · 2026-08-16 indexed (v1 seed) |
| [`HANDOFF-omarchy-macbook-m1.md`](HANDOFF-omarchy-macbook-m1.md) | m1→max migration handoff: shipped PRs, watcher hot-loop bug, new-machine connect bootstrap | — | — | 2026-09-19 | 1 | 2026-09-19 added |
| [`multi-user-kurultai.md`](multi-user-kurultai.md) | Multi-user / hub product notes | — | — | 2026-08-29 | 2 | 2026-08-29 hosted-hub row points at railway-hub.md · 2026-08-16 indexed (v1 seed) |
| [`upstream-inspiration.md`](upstream-inspiration.md) | Upstream inspiration notes | — | — | 2026-07-21 | 1 | 2026-08-16 indexed (v1 seed) |
| [`runbook-hub.md`](runbook-hub.md) | Operator runbook for the Postgres hub on server-001: env/Access/tunnel prereqs, `init-hub-db.sql` bootstrap, migrate via migrate DSN, first admin key, connect-join, AE6 RLS proof, rollback, `pg_dump` warning | `deploy/server-001/docker-compose.kurultai.yml` · `deploy/server-001/init-hub-db.sql` · `deploy/server-001/.env.example` | — | 2026-10-20 | 2 | 2026-10-20 U6 sync to U1–U5 code: solo `:?` vars incl. `PERSONAL_ADMIN_TOKEN`, `web.` tunnel row, `KURULTAI_AUTH_HOST` override, key storage + broker bridge in connect-join, HTTP admin endpoint list, `hub log`, RLS table coverage · 2026-10-20 added (hub adoption U7) |

## Recent

- 2026-10-20 — hub adoption U6 docs/CLI parity (plan 2026-10-19-2053): `AGENT_CONNECTION_PROMPT.md` rewritten for the hub (connect-join easy path + `hub key` power path, per-agent remote MCP wiring); `AGENT_SETUP_PROMPT.md` clap-parity fixes (AGPL license, full tool list); `runbook-hub.md` synced to U1–U5 code
- 2026-10-20 — `runbook-hub.md` added (U7): operator runbook for the server-001 Postgres hub (bootstrap → AE6 → rollback)
- 2026-10-19 — plans/ added `2026-10-19-2053-feat-hub-adoption-easy-connect-plan.md` (first team/company adoption: RLS, connect-join, hub deploy, web/ team app)
- 2026-10-19 — `AGENT_SETUP_PROMPT.md` + `mac-dev.md`: cargo install tag refs bumped v0.4.x → v0.7.0 (stale since release)
- 2026-10-05 — plans/ added 2026-10-05-001 program roadmap plan (U1-U21, plan classification)
- 2026-09-30 — plans/ added 2026-09-30-001 load + chaos harness plan (scripts/hammer.mjs + tests/chaos.rs); hosted write-hammer evidence: ~133 wps, zero errors
- 2026-10-06 — `AGENT_CONNECTION_PROMPT.md`: remote MCP section documents seat-token write surface + api-* hostnames
- 2026-09-29 — `AGENT_CONNECTION_PROMPT.md` added: agent connect prompt ported from kurultai-private
- 2026-09-19 — `HANDOFF-omarchy-macbook-m1.md` migration handoff + `brainstorms/` connect-flow requirements doc
- 2026-09-12 — plans/ added 2026-09-12-001 Brain neuron-motif visual language plan (first-principles derivation)
- 2026-09-05 — plans/ added 002 Brain 0→100 showcase + A2A chatboard plan
- 2026-09-03 — `docs/plans/2026-09-03-001-feat-agent-message-board-plan.md` added
- 2026-09-01 — v0.5.0 Team released; queue and Year-1 milestones marked shipped
- 2026-09-01 — HUB-5 ✅ (#250) · HUB-6 ✅; final stretch to v0.5.0 plan added
- 2026-08-29 — `deploy/` HUB-3 Railway recipe; multi-user hosted-hub row
- 2026-08-16 — `agent-index.md` protocol for the repo-wide map
- 2026-08-16 — indexed this folder (v1 seed)

