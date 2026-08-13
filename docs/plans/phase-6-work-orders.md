# Phase 6 — work orders (post–v0.4.0)

**Status:** Wave B foundation ✅ · next queue → [`phase-6-next-work-orders.md`](phase-6-next-work-orders.md)  
**Date:** 2026-07-30 · **Updated:** 2026-08-12  
**Base:** `main` @ `v0.4.0`+ (Brain UI · MCP HTTP/SSE · thin metrics · soft labels)  
**Umbrella:** [#10](https://github.com/duketopceo/kurultai/issues/10) Open Source Launch · Milestone 6  
**Audience spine:** developer ✅ → solo ✅ (kernel) → team 🔜 → company ([#25](https://github.com/duketopceo/kurultai/issues/25))

## Reality check (do not fight shipped versions)

| Fact | Implication |
|------|-------------|
| Phases 1–5 **product** exit shipped | Milestone 5 leftovers (#20 ARC, #29 env, #35 GlitchTip) are **ops**, not the next LFG product slice |
| Phase 6 **started** (yurt [#100](https://github.com/duketopceo/kurultai/pull/100)) | Continue Phase 6; do not reopen Phase 5 product |
| **v0.4.0** already = Brain UI release | Year-1 team cashflow track is **v0.5.0+** ([`YEAR-1-MILESTONES.md`](YEAR-1-MILESTONES.md)) |
| Wave B foundation **shipped** (2026-08) | Next `/lfg` is **Wave G Tiered Hub** — see [`phase-6-next-work-orders.md`](phase-6-next-work-orders.md) |
| Solo kernel = SQLite forever | Postgres/Redis are for **shared/team**, not replacing laptop `store.db` |

Related long-horizon maps (do not execute wholesale in one LFG):

- Year 1: [`YEAR-1-MILESTONES.md`](YEAR-1-MILESTONES.md)
- Tiered Access brainstorm: [`docs/brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md`](../brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md)
- Scale parents: [#12](https://github.com/duketopceo/kurultai/issues/12) · [#13](https://github.com/duketopceo/kurultai/issues/13) · [#14](https://github.com/duketopceo/kurultai/issues/14)
- Maturity formalization: [#122](https://github.com/duketopceo/kurultai/issues/122)
- Atlas gaps: [`phase-6-atlas-gaps.md`](phase-6-atlas-gaps.md)

---

## Wave B — Foundation (shipped)

Ship **solo excellence + remote-agent reach** before multi-tenant money path — **done** for the thin slices below.

| Order | ID | Work order | Issue | Size | Status |
|------:|----|------------|-------|------|--------|
| 0 | P6-0 | Merge CodeRabbit auto-review off (quota) | [#109](https://github.com/duketopceo/kurultai/pull/109) | XS | ✅ Merged |
| 1 | P6-1 | HTTP/SSE MCP transport (remote agents) | [#104](https://github.com/duketopceo/kurultai/issues/104) | M | ✅ Closed · [#125](https://github.com/duketopceo/kurultai/pull/125) |
| 2 | P6-2 | Built-in metrics + health histograms (**thin**; GlitchTip later) | [#102](https://github.com/duketopceo/kurultai/issues/102) · ties [#35](https://github.com/duketopceo/kurultai/issues/35) | M | ✅ Thin · [#126](https://github.com/duketopceo/kurultai/pull/126) · full #102 still open |
| 3 | P6-3 | Soft multi-label scores + vocabulary | [#113](https://github.com/duketopceo/kurultai/issues/113) | M | ✅ Closed · [#127](https://github.com/duketopceo/kurultai/pull/127) |
| — | — | Export/import `.kurultai` packs (multi-device handoff) | [#80](https://github.com/duketopceo/kurultai/issues/80) slice | M | ✅ [#103](https://github.com/duketopceo/kurultai/pull/103) |
| 4 | P6-4 | Cloud Brain UI tunnel (GitHub login → local daemon) | [#101](https://github.com/duketopceo/kurultai/issues/101) | L | ⏸️ Deferred vs Tiered Hub (see next queue) |

**P6-1 DoD (historical):** daemon exposes authenticated MCP-over-HTTP/SSE for `search` / `ask` / `cite` / `who_knows` (read path); stdio MCP unchanged; localhost default; token or shared-secret gate; tests + README; **no** multi-tenant Postgres in that slice.

---

## Wave C — Team shared brain (v0.5.x track)

Cashflow-shaped; one `Store` backend switch; personal SQLite stays.

> **Superseded for sequencing by Wave G** in [`phase-6-next-work-orders.md`](phase-6-next-work-orders.md) (HUB-1…HUB-6 / [#176](https://github.com/duketopceo/kurultai/issues/176)–[#181](https://github.com/duketopceo/kurultai/issues/181)). Keep the P6-T* ids below as legacy aliases.

| Order | ID | Work order | Issue | Depends |
|------:|----|------------|-------|---------|
| 1 | P6-T1 | Postgres + pgvector `Store` for **shared** index | [#111](https://github.com/duketopceo/kurultai/issues/111) → prefer [#176](https://github.com/duketopceo/kurultai/issues/176) | — |
| 2 | P6-T2 | Redis L2 hot-query cache (optional) | [#112](https://github.com/duketopceo/kurultai/issues/112) | P6-T1 optional |
| 3 | P6-T3 | Personal vs promoted-shared vs company ACL on search/ask | [#115](https://github.com/duketopceo/kurultai/issues/115) → prefer [#178](https://github.com/duketopceo/kurultai/issues/178) scopes | P6-T1 |
| 4 | P6-T4 | Clerk Org → store binding (enforce, not just display) | part of [#115](https://github.com/duketopceo/kurultai/issues/115) / multi-user doc | P6-T3 |
| 5 | P6-T5 | Promote-to-shared workflow (API + audit) | extends promote today | P6-T3 |

Year-1 map: WO-201…204 → release branding **v0.5.0 Team** (not v0.4.0).

---

## Wave D — Enterprise sources & distillation

| Order | ID | Work order | Issue |
|------:|----|------------|-------|
| 1 | P6-E1 | Slack connector **with channel allowlist** + org ACL | [#114](https://github.com/duketopceo/kurultai/issues/114) · [#12](https://github.com/duketopceo/kurultai/issues/12) |
| 2 | P6-E2 | LLM distillation pipeline (summary / Q / tags / soft labels) | [#12](https://github.com/duketopceo/kurultai/issues/12) · feeds [#113](https://github.com/duketopceo/kurultai/issues/113) |
| 3 | P6-E3 | Notion (or next high-value source) | [#121](https://github.com/duketopceo/kurultai/issues/121) · [#12](https://github.com/duketopceo/kurultai/issues/12) |
| 4 | P6-E4 | Hot/warm/cold **storage** (Redis/Postgres/S3) beyond time-class tiers | [#34](https://github.com/duketopceo/kurultai/issues/34) · [#13](https://github.com/duketopceo/kurultai/issues/13) |

---

## Wave E — Ontology / graph (solo→company maturity)

Keep atoms first-class; add structure without a second brain.

| Order | ID | Work order | Issue |
|------:|----|------------|-------|
| 0 | P6-O0 | Living ROADMAP solo→team→company | [#122](https://github.com/duketopceo/kurultai/issues/122) |
| 1 | P6-O1 | Lightweight ontology primitives (entities, links, metrics) | [#116](https://github.com/duketopceo/kurultai/issues/116) |
| 2 | P6-O2 | Interactive knowledge graph in Brain UI | [#117](https://github.com/duketopceo/kurultai/issues/117) |
| 3 | P6-O3 | Ontology brain-building mode (agent propose → human approve) | [#118](https://github.com/duketopceo/kurultai/issues/118) |
| 4 | P6-O4 | Versioned / git-backed knowledge definitions | [#119](https://github.com/duketopceo/kurultai/issues/119) |
| 5 | P6-O5 | Multi-hop graph-augmented retrieval + citations | [#120](https://github.com/duketopceo/kurultai/issues/120) |

Do **not** LFG O2–O5 before O1. Graph UI without primitives is decoration.

---

## Wave F — Launch packaging (can interleave)

From [#10](https://github.com/duketopceo/kurultai/issues/10) — mostly non-kernel:

- Docs site / landing polish  
- crates.io + Homebrew + Docker release automation (CI already partial)  
- Show HN / launch content  
- Plugin system MVP → [#14](https://github.com/duketopceo/kurultai/issues/14) (after team path or demand)

---

## Explicitly deferred (not next LFG)

| Item | Why |
|------|-----|
| Managed Kurultai Cloud hosting | Year-2 ops; tunnel [#101](https://github.com/duketopceo/kurultai/issues/101) first |
| Full plugin marketplace | [#14](https://github.com/duketopceo/kurultai/issues/14) after plugin runtime |
| ARC self-hosted runners [#20](https://github.com/duketopceo/kurultai/issues/20) | Ops; not product exit |
| AppFlowy [#4](https://github.com/duketopceo/kurultai/issues/4) | Low demand vs Slack/Notion |
| Replacing RRF with a custom invent-from-scratch ranker | Extend RRF + optional plugin later |

---

## `/lfg` playbook

**Current:** use [`phase-6-next-work-orders.md`](phase-6-next-work-orders.md) — recommended **HUB-1 / #178**.

Historical Wave B (complete):

1. ~~Human: approve/merge [#109](https://github.com/duketopceo/kurultai/pull/109).~~ ✅  
2. ~~Agent `/lfg`: **P6-1** → [`2026-07-30-002-feat-phase6-mcp-http-sse-plan.md`](2026-07-30-002-feat-phase6-mcp-http-sse-plan.md).~~ ✅  
3. ~~P6-2 thin metrics · P6-3 soft labels.~~ ✅  
4. Team / hub wave: **Wave G** (solo+SSE path green on `main`).

### Hygiene

- Prefer Wave G issue ids (#176–#181) over opening duplicate Trackers for the same Store/ACL work.  
- Close accidental [#110](https://github.com/duketopceo/kurultai/issues/110) when a human can.  
- Year-1 team track remains **v0.5.0** (v0.4.0 already shipped as Brain UI).
