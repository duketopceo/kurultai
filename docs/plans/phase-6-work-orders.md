# Phase 6 — next work orders (post–v0.4.0)

**Status:** Planning · ready for `/lfg`  
**Date:** 2026-07-30  
**Base:** `main` @ `v0.4.0` (Brain UI: solar · pulse · purple · max)  
**Umbrella:** [#10](https://github.com/duketopceo/kurultai/issues/10) Open Source Launch · Milestone 6  
**Audience spine:** developer ✅ → solo 🔜 → team → company ([#25](https://github.com/duketopceo/kurultai/issues/25))

## Reality check (do not fight shipped versions)

| Fact | Implication |
|------|-------------|
| Phases 1–5 **product** exit shipped | Milestone 5 leftovers (#20 ARC, #29 env, #35 GlitchTip) are **ops**, not the next LFG product slice |
| Phase 6 **started** (yurt [#100](https://github.com/duketopceo/kurultai/pull/100)) | Continue Phase 6; do not reopen Phase 5 product |
| **v0.4.0** already = Brain UI release | Devin Year-1 doc ([#123](https://github.com/duketopceo/kurultai/pull/123)) still calls “v0.4.0 Team” — **renumber** that track to **v0.5.0+** before merging |
| Solo kernel = SQLite forever | Postgres/Redis are for **shared/team**, not replacing laptop `store.db` |

Related long-horizon maps (do not execute wholesale in one LFG):

- Devin Year 1: [#123](https://github.com/duketopceo/kurultai/pull/123) `YEAR-1-MILESTONES.md` (needs version renumber)
- Scale parents: [#12](https://github.com/duketopceo/kurultai/issues/12) · [#13](https://github.com/duketopceo/kurultai/issues/13) · [#14](https://github.com/duketopceo/kurultai/issues/14)
- Maturity formalization: [#122](https://github.com/duketopceo/kurultai/issues/122)

---

## Wave B — Foundation (next LFG queue)

Ship **solo excellence + remote-agent reach** before multi-tenant money path.

| Order | ID | Work order | Issue | Size | First LFG? |
|------:|----|------------|-------|------|------------|
| 0 | P6-0 | Merge CodeRabbit auto-review off (quota) | [#109](https://github.com/duketopceo/kurultai/pull/109) | XS | Human approve only |
| 1 | **P6-1** | **HTTP/SSE MCP transport** (remote agents / Perplexity-class) | [#104](https://github.com/duketopceo/kurultai/issues/104) | M | **Shipped / in PR** — `POST /mcp` + `GET /mcp/sse` |
| 2 | P6-2 | Built-in metrics + health histograms (thin; GlitchTip later) | [#102](https://github.com/duketopceo/kurultai/issues/102) · ties [#35](https://github.com/duketopceo/kurultai/issues/35) | M | After P6-1 or parallel thin slice |
| 3 | P6-3 | Soft multi-label scores + vocabulary (keep hard-tag gate) | [#113](https://github.com/duketopceo/kurultai/issues/113) | M | Structure win; feeds Brain colour / search boost |
| 4 | P6-4 | Cloud Brain UI tunnel (GitHub login → local daemon) | [#101](https://github.com/duketopceo/kurultai/issues/101) | L | Needs P6-1 SSE/auth story |

**P6-1 DoD (LFG capsule):** daemon exposes authenticated MCP-over-HTTP/SSE for `search` / `ask` / `cite` / `who_knows` (read path); stdio MCP unchanged; localhost default; token or shared-secret gate; tests + README; **no** multi-tenant Postgres in this slice.

---

## Wave C — Team shared brain (v0.5.x track)

Cashflow-shaped; one `Store` backend switch; personal SQLite stays.

| Order | ID | Work order | Issue | Depends |
|------:|----|------------|-------|---------|
| 1 | P6-T1 | Postgres + pgvector `Store` for **shared** index | [#111](https://github.com/duketopceo/kurultai/issues/111) | — |
| 2 | P6-T2 | Redis L2 hot-query cache (optional) | [#112](https://github.com/duketopceo/kurultai/issues/112) | P6-T1 optional |
| 3 | P6-T3 | Personal vs promoted-shared vs company ACL on search/ask | [#115](https://github.com/duketopceo/kurultai/issues/115) | P6-T1 |
| 4 | P6-T4 | Clerk Org → store binding (enforce, not just display) | part of [#115](https://github.com/duketopceo/kurultai/issues/115) / multi-user doc | P6-T3 |
| 5 | P6-T5 | Promote-to-shared workflow (API + audit) | extends promote today | P6-T3 |

Devin map: WO-201…204 → renumber release to **v0.5.0 Team** (not v0.4.0).

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

## `/lfg` playbook (this queue)

1. Human: approve/merge [#109](https://github.com/duketopceo/kurultai/pull/109) if still open.  
2. Agent `/lfg`: **P6-1** → implement from plan [`2026-07-30-002-feat-phase6-mcp-http-sse-plan.md`](2026-07-30-002-feat-phase6-mcp-http-sse-plan.md).  
3. Next LFG candidates (pick one): P6-2 metrics · P6-3 soft labels · P6-O0 ROADMAP.  
4. Team wave starts only after solo+SSE path is green on `main`.

### Hygiene

- Assign [#111](https://github.com/duketopceo/kurultai/issues/111)–[#115](https://github.com/duketopceo/kurultai/issues/115) and [#116](https://github.com/duketopceo/kurultai/issues/116)–[#122](https://github.com/duketopceo/kurultai/issues/122) to **Phase 6** milestone (agent token often cannot).  
- Close accidental [#110](https://github.com/duketopceo/kurultai/issues/110).  
- Before merging [#123](https://github.com/duketopceo/kurultai/pull/123): renumber Year-1 “v0.4.0 Team” → **v0.5.0** (v0.4.0 already shipped).
