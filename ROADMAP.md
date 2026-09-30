# Kurultai Roadmap

Living audience roadmap: **solo → team → company**. Tracks [#122](https://github.com/duketopceo/kurultai/issues/122).

This doc sequences *who the product serves next* and what "done" means per stage. It is deliberately thin:

- Live work queue → [`docs/plans/phase-6-next-work-orders.md`](docs/plans/phase-6-next-work-orders.md)
- Dates, revenue targets, work orders → [`docs/plans/YEAR-1-MILESTONES.md`](docs/plans/YEAR-1-MILESTONES.md)
- Audience spine issues → [#25](https://github.com/duketopceo/kurultai/issues/25) (developer → solo) · [#27](https://github.com/duketopceo/kurultai/issues/27) (team → company) · [#10](https://github.com/duketopceo/kurultai/issues/10) (open-source launch)

Update this file when a stage flips or a scope decision changes — not per PR.

## Stage 1 — Solo: local excellence

*One person, one machine, every agent wired to one brain.* Solo kernel is **SQLite forever**.

**Shipped** (v0.4.x–v0.6.0): hybrid search (FTS5 + optional vectors) · 8 MCP tools · daemon + embedded Brain UI (`/ui/`) · connectors (Dayflow, Pond, GitHub FS, inbox, JSON) · export/import packs · `init --docs` · MCP HTTP/SSE · hot/warm/cold tiered graph load · pond/session sequester out of hot retrieval · device-code hosted login + Cloudflare Access JWT (1Password-friendly) · Hey A2A board (MCP + REST + presence) · recall harness.

**Open work:**

| Issue | Item |
|-------|------|
| [#137](https://github.com/duketopceo/kurultai/issues/137) · [#140](https://github.com/duketopceo/kurultai/issues/140) · [#139](https://github.com/duketopceo/kurultai/issues/139) | Brain viz / Explorer to 100%: layouts, perf, drill-down |
| [#101](https://github.com/duketopceo/kurultai/issues/101) | Cloud Brain UI tunnel (GitHub login → local daemon) — P6-4 |
| [#102](https://github.com/duketopceo/kurultai/issues/102) | Full metrics + error tracking beyond thin slice — P6-2b |
| [#4](https://github.com/duketopceo/kurultai/issues/4) · [#78](https://github.com/duketopceo/kurultai/issues/78) · [#121](https://github.com/duketopceo/kurultai/issues/121) | Connectors: AppFlowy, CodeGraph, more sources |
| [#79](https://github.com/duketopceo/kurultai/issues/79) | Sign in with Anthropic (website account login) |

**Exit criteria:** fresh machine runs `init → index → ask → /ui/` without surprises; Luke's agents dogfood it daily without retrieval noise; Brain showcase holds at real corpus size.

## Stage 2 — Team: shared brain

*One hub, many agents + humans; visibility scopes enforced at ingest and query.* Shared tier uses Postgres/pgvector — never a converted personal `store.db`. Hub ships behind `KURULTAI_FEATURE_HUB` (default off).

**Shipped** (v0.5.0): HUB-1 `personal|team|company` scopes · HUB-2 `PostgresStore` · HUB-3 hub daemon bind policy (Tailscale / Railway) · HUB-4 device keys + durable write log · HUB-5 ingest-time scope tagging · HUB-6 acceptance suite AE1–AE5.

**Open work:**

| Issue | Item |
|-------|------|
| [#115](https://github.com/duketopceo/kurultai/issues/115) | RBAC on search/ask: personal vs promoted-shared vs company |
| [#114](https://github.com/duketopceo/kurultai/issues/114) · [#134](https://github.com/duketopceo/kurultai/issues/134) · [#135](https://github.com/duketopceo/kurultai/issues/135) | Connectors: Slack channel allowlist, webhook runtime, Notion sync |
| [#133](https://github.com/duketopceo/kurultai/issues/133) · [#130](https://github.com/duketopceo/kurultai/issues/130) | Structured source contract (objects+edges+atoms); schema/lineage catalog ingest |
| [#112](https://github.com/duketopceo/kurultai/issues/112) · [#131](https://github.com/duketopceo/kurultai/issues/131) | Store: Redis L2 hot cache; typed edge index + bulk import |
| — | `web/` Next.js + Clerk team app (scaffold shipped; protected routes need keys) |

**Exit criteria:** a second person joins a hub with a per-device key and sees only `team`+`company` atoms; AE suite stays green; onboarding needs no engineer.

## Stage 3 — Company: ontology-grade brain

*The brain becomes a typed, governed graph — agents propose, humans decide.*

**Landed:** O1 labeled property graph seed (`src/ontology/` — seeded classes, promote atom→entity, `POST /api/ontology/promote`) · O3 proposal + approval queue ([#313](https://github.com/duketopceo/kurultai/pull/313): `ontology_propose`/`decide`, `/api/ontology/proposals*`, Brain UI review panel).

**Open work:**

| Issue | Item |
|-------|------|
| [#116](https://github.com/duketopceo/kurultai/issues/116) · [#132](https://github.com/duketopceo/kurultai/issues/132) | Finish O1 primitives (entities/links/metrics); promote→typed object with audit |
| [#128](https://github.com/duketopceo/kurultai/issues/128) · [#129](https://github.com/duketopceo/kurultai/issues/129) | Atlas: object class registry + corpus stats; projection views (strata/domains/timeline/attention) |
| [#119](https://github.com/duketopceo/kurultai/issues/119) · [#120](https://github.com/duketopceo/kurultai/issues/120) | Versioned/git-backed knowledge definitions; multi-hop graph retrieval + citations |
| [#117](https://github.com/duketopceo/kurultai/issues/117) · [#138](https://github.com/duketopceo/kurultai/issues/138) · [#188](https://github.com/duketopceo/kurultai/issues/188) | Interactive graph view; business/ops dashboard; claim-level permissions |

**Brain gate:** ontology *layout* in BrainStage waits on O1 — Slice A (visual) landed, Slice B (O1 primitives) next, Slice C (typed hierarchy mode) after. See [`docs/plans/2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md`](docs/plans/2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md).

**Exit criteria:** ontology mode browses a typed hierarchy on a real (non-empty) brain; every ontology write is human-decided; multi-hop `ask` answers cite through entities/links.

## Cross-cutting

| Issue | Item |
|-------|------|
| [#10](https://github.com/duketopceo/kurultai/issues/10) | Open-source launch: crates.io, Homebrew, Show HN (Launch F) |
| [#20](https://github.com/duketopceo/kurultai/issues/20) · [#29](https://github.com/duketopceo/kurultai/issues/29) | Ops: ARC, env hardening |
| [#122](https://github.com/duketopceo/kurultai/issues/122) | This roadmap — keep it living |

## Non-goals / deferred

- **Multi-tenant SaaS** for many unrelated orgs on one Kurultai-operated platform — explicitly rejected.
- **Password/session hub accounts** — v1 is per-device API keys (+ verified CF Access JWT for humans).
- **Tier-gated UI redesign** — out of scope; Brain doctrine unchanged.
- **Replacing `.kurultai` export/import** — stays for offline solo; hub supersedes only shared tiers.
- Year-2 deferrals (managed cloud, centralized marketplace, mobile, HIPAA/FedRAMP, learning-to-rank, v2 graph queries) — see `YEAR-1-MILESTONES.md`.
