# Phase 6 — next work orders (post–v0.4.0)

**Status:** Planning · ready for `/lfg`  
**Date:** 2026-07-31 (atlas gap fill)  
**Base:** `main` @ `v0.4.0` (Brain UI: solar · pulse · purple · max)  
**Umbrella:** [#10](https://github.com/duketopceo/kurultai/issues/10) Open Source Launch · Milestone 6  
**Audience spine:** developer ✅ → solo 🔜 → team → company ([#25](https://github.com/duketopceo/kurultai/issues/25))  
**Atlas gaps detail:** [`phase-6-atlas-gaps.md`](phase-6-atlas-gaps.md)

## Reality check (do not fight shipped versions)

| Fact | Implication |
|------|-------------|
| Phases 1–5 **product** exit shipped | Milestone 5 leftovers (#20 ARC, #29 env, #35 GlitchTip) are **ops**, not the next LFG product slice |
| Phase 6 **started** (yurt [#100](https://github.com/duketopceo/kurultai/pull/100)) | Continue Phase 6; do not reopen Phase 5 product |
| **v0.4.0** already = Brain UI release | Devin Year-1 doc ([#123](https://github.com/duketopceo/kurultai/pull/123)) still calls “v0.4.0 Team” — **renumber** that track to **v0.5.0+** before merging |
| Solo kernel = SQLite forever | Postgres/Redis are for **shared/team**, not replacing laptop `store.db` |
| Agent memories ≠ Atlas | Atoms/FTS/ask are the memory path; typed catalogs + projections are a parallel maturity track |

Related long-horizon maps (do not execute wholesale in one LFG):

- Devin Year 1: [#123](https://github.com/duketopceo/kurultai/pull/123) `YEAR-1-MILESTONES.md` (needs version renumber)
- Scale parents: [#12](https://github.com/duketopceo/kurultai/issues/12) · [#13](https://github.com/duketopceo/kurultai/issues/13) · [#14](https://github.com/duketopceo/kurultai/issues/14)
- Maturity formalization: [#122](https://github.com/duketopceo/kurultai/issues/122)

---

## Wave B — Foundation (next LFG queue)

Ship **solo excellence + remote-agent reach** before multi-tenant money path.

| Order | ID | Work order | Issue | Size | Status |
|------:|----|------------|-------|------|--------|
| 0 | P6-0 | Merge CodeRabbit auto-review off (quota) | [#109](https://github.com/duketopceo/kurultai/pull/109) | XS | Human approve |
| 1 | P6-1 | HTTP/SSE MCP transport | [#104](https://github.com/duketopceo/kurultai/issues/104) · [#125](https://github.com/duketopceo/kurultai/pull/125) | M | In PR |
| 2 | P6-2 | Built-in metrics + health histograms (thin) | [#102](https://github.com/duketopceo/kurultai/issues/102) · [#126](https://github.com/duketopceo/kurultai/pull/126) | M | In PR |
| 3 | P6-3 | Soft multi-label scores + vocabulary | [#113](https://github.com/duketopceo/kurultai/issues/113) · [#127](https://github.com/duketopceo/kurultai/pull/127) | M | In PR |
| 4 | P6-4 | Cloud Brain UI tunnel (GitHub login → local daemon) | [#101](https://github.com/duketopceo/kurultai/issues/101) | L | Needs P6-1 auth story |

---

## Wave C — Team shared brain (v0.5.x track)

Cashflow-shaped; one `Store` backend switch; personal SQLite stays.

| Order | ID | Work order | Issue | Depends |
|------:|----|------------|-------|---------|
| 1 | P6-T1 | Postgres + pgvector `Store` for **shared** index | [#111](https://github.com/duketopceo/kurultai/issues/111) | — |
| 1b | **P6-T1b** | **Postgres Store holds objects + edges** (not atoms-only) | extends [#111](https://github.com/duketopceo/kurultai/issues/111) · [#131](https://github.com/duketopceo/kurultai/issues/131) | T1 + O1/A1 |
| 2 | P6-T2 | Redis L2 hot-query cache (optional) | [#112](https://github.com/duketopceo/kurultai/issues/112) | T1 optional |
| 3 | P6-T3 | Personal vs promoted-shared vs company ACL | [#115](https://github.com/duketopceo/kurultai/issues/115) | T1 |
| 4 | P6-T4 | Clerk Org → store binding (enforce) | part of [#115](https://github.com/duketopceo/kurultai/issues/115) | T3 |
| 5 | P6-T5 | Promote-to-shared workflow (API + audit) | extends promote today | T3 |

---

## Wave D — Enterprise sources & distillation

| Order | ID | Work order | Issue |
|------:|----|------------|-------|
| 0 | **P6-E0** | **Structured connector contract** (objects + edges + atoms) | [#133](https://github.com/duketopceo/kurultai/issues/133) |
| 1 | P6-E1 | Slack connector **with channel allowlist** + org ACL | [#114](https://github.com/duketopceo/kurultai/issues/114) · [#12](https://github.com/duketopceo/kurultai/issues/12) |
| 2 | P6-E2 | LLM distillation pipeline (summary / Q / tags / soft labels) | [#12](https://github.com/duketopceo/kurultai/issues/12) · feeds [#113](https://github.com/duketopceo/kurultai/issues/113) |
| 3 | P6-E3a | **Notion** workspace sync (split from bucket) | [#135](https://github.com/duketopceo/kurultai/issues/135) · [#121](https://github.com/duketopceo/kurultai/issues/121) |
| 3b | P6-E3b | Remaining #121 bucket (Confluence, Drive, Linear/Jira, richer local files) | [#121](https://github.com/duketopceo/kurultai/issues/121) — split further on demand |
| 4 | P6-E4 | Hot/warm/cold **storage** (Redis/Postgres/S3) beyond time-class tiers | [#34](https://github.com/duketopceo/kurultai/issues/34) · [#13](https://github.com/duketopceo/kurultai/issues/13) |
| 5 | **P6-E5** | **Webhook / real-time ingest runtime** | [#134](https://github.com/duketopceo/kurultai/issues/134) · [#12](https://github.com/duketopceo/kurultai/issues/12) |
| 6 | **P6-E6** | **CodeGraph / code-structure connector** | [#78](https://github.com/duketopceo/kurultai/issues/78) |

Do **not** ship Notion/SQL-catalog at Atlas fidelity before **E0**.

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

## Wave E′ — Atlas scale (typed corpus + projections)

**After O1 + E0.** Memories stay atoms; Atlas is typed inventory + navigable projections.

| Order | ID | Work order | Issue | Depends |
|------:|----|------------|-------|---------|
| 1 | **P6-A1** | Object class registry + corpus stats API | [#128](https://github.com/duketopceo/kurultai/issues/128) | O1 (or paired) |
| 2 | **P6-A2** | Projections: strata / domains / timeline / attention (+ constellation→O2) | [#129](https://github.com/duketopceo/kurultai/issues/129) | A1 |
| 3 | **P6-A3** | Schema/lineage catalog ingest (SQL or dbt first) | [#130](https://github.com/duketopceo/kurultai/issues/130) | E0 |
| 4 | **P6-A4** | Typed edge index + bulk import | [#131](https://github.com/duketopceo/kurultai/issues/131) | O1 |
| 5 | **P6-A5** | Promote atom → typed object/entity + audit | [#132](https://github.com/duketopceo/kurultai/issues/132) | A1 |

UI rule: extend Brain `ui/` (Atlas **mode**), do not add a second dashboard product.

---

## Wave F — Launch packaging (can interleave)

From [#10](https://github.com/duketopceo/kurultai/issues/10) — mostly non-kernel:

- Docs site / landing polish  
- crates.io + Homebrew + Docker release automation (CI already partial)  
- Show HN / launch content  
- Plugin system MVP → [#14](https://github.com/duketopceo/kurultai/issues/14) (builds on **E0** contract)

---

## Explicitly deferred (not next LFG)

| Item | Why |
|------|-----|
| Managed Kurultai Cloud hosting | Year-2 ops; tunnel [#101](https://github.com/duketopceo/kurultai/issues/101) first |
| Full plugin marketplace | [#14](https://github.com/duketopceo/kurultai/issues/14) after plugin runtime |
| ARC self-hosted runners [#20](https://github.com/duketopceo/kurultai/issues/20) | Ops; not product exit |
| AppFlowy [#4](https://github.com/duketopceo/kurultai/issues/4) | Low demand vs Slack/Notion/schema |
| Replacing RRF with a custom invent-from-scratch ranker | Extend RRF + optional plugin later |
| TextQL-clone Atlas chrome | Kurultai visual language; projections only |

---

## `/lfg` playbook (this queue)

1. Human: approve/merge [#109](https://github.com/duketopceo/kurultai/pull/109), [#125](https://github.com/duketopceo/kurultai/pull/125)–[#127](https://github.com/duketopceo/kurultai/pull/127) if still open.  
2. Next product LFG (pick one): **P6-O0** ROADMAP · **P6-E0** structured contract · **P6-O1** ontology primitives.  
3. Atlas WOs (A1–A5) only after O1 + E0 green.  
4. Team wave starts only after solo+SSE path is green on `main`.

### Hygiene

- Assign [#111](https://github.com/duketopceo/kurultai/issues/111)–[#115](https://github.com/duketopceo/kurultai/issues/115), [#116](https://github.com/duketopceo/kurultai/issues/116)–[#122](https://github.com/duketopceo/kurultai/issues/122), [#128](https://github.com/duketopceo/kurultai/issues/128)–[#135](https://github.com/duketopceo/kurultai/issues/135) to **Phase 6** milestone (agent token often cannot).  
- Close accidental [#110](https://github.com/duketopceo/kurultai/issues/110).  
- Before merging [#123](https://github.com/duketopceo/kurultai/pull/123): renumber Year-1 “v0.4.0 Team” → **v0.5.0**.  
- Supersedes docs-only sequencing in [#124](https://github.com/duketopceo/kurultai/pull/124) once this lands.
