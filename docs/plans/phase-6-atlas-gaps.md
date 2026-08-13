# Phase 6 — Ontology / Atlas / connector gaps

**Date:** 2026-07-31  
**Status:** Work orders filed · ready to sequence after Wave B merges  
**Parent pack:** [`phase-6-work-orders.md`](phase-6-work-orders.md)

## Premise

Kurultai **agent memories** (atoms, FTS+vector, soft labels, Brain orbs) and an **Ontology Atlas** (typed corpus, projections, lineage) are related but not the same product surface.

| Layer | Unit | Job |
|-------|------|-----|
| Memories | `KnowledgeAtom` | Recall what people/agents said and did |
| Atlas | Typed objects + edges | Map what the org *is* (schemas, systems, modules) |

Atoms cite and explain; atlas objects are often the things being cited.

## What Wave E already covered

O0–O5 (#122, #116–#120): roadmap, entity/link/metric primitives, graph UI, agent propose, versioned defs, multi-hop retrieval.

**Enough for a knowledge graph over memories. Not enough for Atlas-scale structured data.**

## Gaps → work orders

### Atlas (Wave E′)

| ID | Gap | Issue |
|----|-----|-------|
| P6-A1 | Object class registry + corpus stats (files/edges/volume/hit-rate class signals) | [#128](https://github.com/duketopceo/kurultai/issues/128) |
| P6-A2 | Projections: Strata · Domains · Timeline · Attention (Constellation → [#117](https://github.com/duketopceo/kurultai/issues/117)) | [#129](https://github.com/duketopceo/kurultai/issues/129) |
| P6-A3 | Schema/lineage connector (SQL catalog or dbt) | [#130](https://github.com/duketopceo/kurultai/issues/130) |
| P6-A4 | Edge index + bulk import at thousands–millions | [#131](https://github.com/duketopceo/kurultai/issues/131) |
| P6-A5 | Promote atom → typed object/entity + audit | [#132](https://github.com/duketopceo/kurultai/issues/132) |

### Connectors (Wave D fixes)

| ID | Gap | Issue |
|----|-----|-------|
| P6-E0 | Structured connector contract `{atoms?, objects, edges}` | [#133](https://github.com/duketopceo/kurultai/issues/133) |
| P6-E3a | Notion split from [#121](https://github.com/duketopceo/kurultai/issues/121) bucket | [#135](https://github.com/duketopceo/kurultai/issues/135) |
| P6-E5 | Webhook / real-time ingest runtime | [#134](https://github.com/duketopceo/kurultai/issues/134) |
| P6-E6 | CodeGraph / code-structure ([#78](https://github.com/duketopceo/kurultai/issues/78) pulled into wave) | [#78](https://github.com/duketopceo/kurultai/issues/78) |

Still in bucket [#121](https://github.com/duketopceo/kurultai/issues/121): Confluence/wiki, Drive/Docs, Linear/Jira, richer local files — split when prioritized.

### Team store bridge

| ID | Gap | Notes |
|----|-----|-------|
| P6-T1b | Postgres Store must persist **objects + edges**, not atoms-only | Call out on [#111](https://github.com/duketopceo/kurultai/issues/111); harden edges via [#131](https://github.com/duketopceo/kurultai/issues/131) |

## Sequencing (do not violate)

```text
E0 structured contract ──┬──► A3 schema/lineage
                         └──► E3a Notion / E6 CodeGraph (richer later)

O1 ontology primitives ──┬──► A1 object registry ──► A2 projections
                         ├──► A4 edge index
                         └──► A5 atom→object promote

T1 Postgres ──► T1b objects+edges on shared backend
```

- No Atlas UI before **O1 + E0**.  
- No TB-scale projections before **T1/T1b** when multi-writer is real.  
- Soft labels (#113) and query metrics (#102) **feed** Attention/colour; they do not replace the catalog.

## UI doctrine

One Brain product (`ui/` embedded in daemon). Atlas is a **mode / projection set**, not a second Vite app or `website/` dashboard.

Intended Brain toggles (research, 2026-08-13): **brain-shape** volumetric FDG inside the cortex hull, and **algorithmic ontology** (schema scaffold, instances on expand). Galaxy/solar is not a peer mode. See [`docs/brainstorms/2026-08-13---brain-shape-algorithmic-ontology.md`](../brainstorms/2026-08-13---brain-shape-algorithmic-ontology.md) and plan [`2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md`](2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md). Ontology **layout** still waits on O1.

## Explicit non-goals (this gap fill)

- Cloning TextQL Ontology Atlas chrome  
- Managed multi-tenant webhook mesh  
- Replacing SQLite personal kernel  
- Implementing A1–A5 in the same LFG as this docs pack
