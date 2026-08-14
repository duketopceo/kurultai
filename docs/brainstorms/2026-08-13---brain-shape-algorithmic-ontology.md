# Dual-state Brain: volumetric FDG + algorithmic ontology

**Status:** Research accepted · implementation plan [`docs/plans/2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md`](../plans/2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md)  
**Date:** 2026-08-13  
**Origin:** Product direction after v0.4.1: default **brain-shape** layout; **ontology** is the other real mode; **galaxy/solar is out**; current ontology toggle is useless.  
**Related:** [#116](https://github.com/duketopceo/kurultai/issues/116) O1 primitives · [#117](https://github.com/duketopceo/kurultai/issues/117) O2 graph UI · [#118](https://github.com/duketopceo/kurultai/issues/118) agent propose → human approve · [`phase-6-atlas-gaps.md`](../plans/phase-6-atlas-gaps.md) · [`phase-6-work-orders.md`](../plans/phase-6-work-orders.md) Wave E

## Verdict

Two different products share one canvas. **Brain mode** is a spatial memory map (forces + a cortex hull). **Ontology mode** is a typed hierarchy (schema first, instances attached). Painting 7,000 atoms with an unconstrained spring blob and calling it ontology does not become ontology.

Do **not** invert the existing Wave E rule: graph UI without primitives is decoration. O1 before a real ontology layout. A brain-shape FDG can ship earlier because it only needs atoms, tags, and the GLB already in the renderer.

## What ships today

| Toggle | What the code actually does | Why it fails the idea |
|--------|-----------------------------|------------------------|
| **brain** (default) | Pin each atom to a **GLB cortex vertex** (`vertexForRegion`). Hemispheres are **degree buckets** (stem / right / left), not tags. | Surface scatter, not a volumetric FDG. Tags do not pull toward subregions. |
| **ontology** | Same vanilla JS n² charge/spring sim used as a blob (`FORCE_REPULSION_*` in `website/src/brain/BrainView.ts`). | No Class → Subclass → Instance, no directed typed edges, no layers. It is a second organic cloud. |
| **galaxy** | Solar orbits by degree (sun + ≤14 planets + moons/asteroids). | Explicitly rejected as a product mode. |

Edges are not stored. `BrainStage.buildLinks` builds **undirected tag cliques** in the browser (`strength: 1`, capped per tag). `/api/graph` returns lean **nodes only** — no link payload. Soft labels ([#113](https://github.com/duketopceo/kurultai/issues/113)) exist on atoms and are unused by layout.

The current sim already admits n² does not scale: at `n > 1500` it caps at 150 iterations because a full 300-iter pass is tens of seconds. 7,000 nodes on that path is not a tuning problem.

[#117](https://github.com/duketopceo/kurultai/issues/117) asked for force-directed **and/or hierarchical** projections at 1k–10k nodes, and listed **3D viz as out of scope**. This direction expands O2 into 3D dual-mode instead of a 2D graph panel. Atlas ([`phase-6-atlas-gaps.md`](../plans/phase-6-atlas-gaps.md)) remains a **mode / projection**, not a second app.

## Algorithms that apply

### Brain mode — constrained FDG inside the existing hull

**Forces.** Fruchterman–Reingold / n-body repulsion + spring attraction + weak center gravity is the standard. Naive pair repulsion is O(n²). [Barnes–Hut](https://jheer.github.io/barnes-hut/) approximates distant groups as a center of mass via a **quadtree (2D) or octree (3D)**, O(n log n), with accuracy θ (typical 0.8–0.9). [d3-force-3d](https://github.com/vasturiano/d3-force-3d/) already does this with an octree for 3D many-body (`theta` default 0.9). Production 3D UIs run that sim **off the main thread**; at ~10k nodes even Barnes–Hut is tens of ms per tick and freezes orbit/hover if it shares the render loop ([Three.js network-graph notes](https://intelligentgraphicandcode.com/development/threejs-interfaces/network-graphs)). GPU Barnes–Hut exists ([vibe-graph-layout-gpu](https://crates.io/crates/vibe-graph-layout-gpu) reports 60+ FPS at 10k on WebGPU) — optional later, not required for a first 2–3k solo graph.

**Envelope.** Academic “draw a boundary, add environmental forces” is 2D polygons ([Zhang & Pang, boundary-constrained FDG](https://escholarship.org/uc/item/0vd969mx); [JGAA paper](https://jgaa.info/index.php/jgaa/article/download/paper401/2568/2375)): interior vertices get a mild inward force; exterior vertices get a stronger restore. A closed-form \(F(x,y,z) \le 1\) brain equation is the wrong primitive here: the renderer **already has a watertight cortex GLB**. Practical constraint:

1. Build a **signed distance / inside test** from the mesh (voxel SDF or ray-cast winding, baked once at load).
2. Each tick: if a node is outside, add \(-\nabla\) toward the interior (or project back along the normal).
3. Keep repulsion so the cloud fills volume instead of collapsing to the surface (today’s pin-to-vertex look).

**Tag gravity (not fake neuroanatomy).** “Episodic → temporal lobe” is decoration unless we maintain a tag→lobe table nobody asked to curate. Do this instead: for each tag (or soft-label) with enough members, compute a **centroid attractor** inside the hull; members get a weak pull; attractors themselves repel so clusters separate. Optional later: a small curated map (`code` → stem, `people` → left, …) as an overlay, not the engine.

**Edges in brain mode.** Keep synapses **low-opacity / hover-only**. Tag cliques at 7k nodes are a hairball. Show the top-N by strength plus the hovered star, same as now.

### Ontology mode — layered hierarchy, not another FDG

Unconstrained FDG is what WebVOWL uses for **OWL class diagrams**, and it is meant to put highly connected **classes** in the center — not 7,000 document instances ([Lohmann et al., VOWL / WebVOWL](https://journals.sagepub.com/doi/10.3233/SW-150200)). Ontodia similarly offers force **or grid**, plus a **class tree** for navigation ([Ontodia](https://github.com/metaphacts/ontodia); [ESWC poster](https://ceur-ws.org/Vol-1486/paper_77.pdf)).

For **taxonomic depth**, the standard is [Sugiyama layered drawing](https://en.wikipedia.org/wiki/Layered_graph_drawing) (1981): cycle removal → assign layers → reduce crossings → assign coordinates. Implemented by Graphviz `dot` and described in the [GD Handbook hierarchical chapter](https://cs.brown.edu/people/rtamassi/gdhandbook/chapters/hierarchical.pdf). The same framework has a **3D layered** variant (layers as Y, permutation in XZ). Property / transversal edges (`triggered_by`, `contradicts`) are a second relation set: draw them as **colored directed vectors** after the is-a layering is fixed, or they will fight the hierarchy (Sugiyama + force hybrids exist; force-first on mixed graphs is why “ontology” looks like a blob today).

**Product fork (must pick before coding layout):**

| Ontology canvas | Typical size | Algorithm | Honest name |
|-----------------|--------------|-----------|-------------|
| **A. Schema** — classes, properties, a few example instances | tens–hundreds | WebVOWL-style FDG **or** Sugiyama on `is-a` | Real ontology |
| **B. Instance cloud** — every atom as a node, Y = inferred type depth | thousands | Layered + clustering; must hide most edges | Memory taxonomy |
| **C. Both** — schema as the scaffold; instances fan out under their class on demand | progressive | A, then expand B | Matches Atlas “projections” |

**C is the one that stays usable at 7k.** Dumping all atoms into a Sugiyama drawing is a 7,000-leaf tree, not a readable ontology.

**Tween.** The existing 850ms `animateLayoutTo` lerp is enough. Cosine vs current ease is cosmetics. The hard work is **two target fields** (`brainPos`, `ontoPos`), not the interpolator. Disable the hull force when \(t\) leaves brain mode; do not run two physics engines during the fade.

## Data model — this is the ontology

[#116](https://github.com/duketopceo/kurultai/issues/116) already named the primitives: **Entity**, **Link** (typed, mixed atom/entity endpoints), **Metric/Definition**, non-destructive **promote** from atoms. Until those exist, layout can only infer.

| Need for “full algorithmic ontology” | In tree today | Enough? |
|--------------------------------------|---------------|---------|
| Stable typed nodes (Class / Entity) | Atoms + tags + soft labels | No — tags are not classes |
| Directed typed edges with provenance | Client-side undirected tag pairs | No |
| Hierarchy depth (Y axis) | — | No — must be `is-a` / `instance-of` |
| Transversal properties (XZ) | — | No |
| Promote atom → structure without deleting source | `promote` is trust-lane only | No |
| Agent propose → human approve | — | [#118](https://github.com/duketopceo/kurultai/issues/118) |

**Store choice for O1:** a **labeled property graph** beside SQLite atoms, not an OWL triple store. RDF/OWL wins at published vocabularies and reasoners; property graphs win at edge metadata (source, confidence, who approved) and traversal — the usual split in KG practice ([TigerGraph comparison](https://www.tigergraph.com/blog/property-graph-vs-rdf/); [Neo4j on RDF vs property graphs](https://neo4j.com/blog/knowledge-graph/rdf-vs-property-graphs-knowledge-graphs/)). Kurultai already has promote + audit instincts; edge properties map cleanly. Optional OWL export is Wave E′ / Atlas, not the kernel.

**Inference v1 (honest, not magical):**

1. Human or agent **defines** a small class tree (or we ship a starter: `Memory` → `Note` / `Code` / `Decision` / `Person` / `System`).
2. Soft labels + tags **suggest** `instance-of` (score, never silent write).
3. Typed links are **proposed** ([#118](https://github.com/duketopceo/kurultai/issues/118)), not extracted as truth from co-occurrence.
4. Co-occurrence / embedding kNN can seed `associates_with` as **low-confidence** edges, visually distinct from approved `is-a`.

That is “algorithmic”: layout is a pure function of the graph. It is not “the engine invents an OWL TBox from 7,000 notes.”

## Recommended sequencing

```text
O1 property-graph primitives (#116)
    │
    ├── Brain-shape FDG (atoms + tags + GLB SDF)     ← visual; no O1 required
    │     deprecate galaxy in the same slice
    │
    └── O2 ontology layout (#117, 3D layered)
              schema scaffold first, instances on expand
              O3 propose/approve (#118) before unsupervised edges
```

1. **Research (this note).** Direction locked: two modes, galaxy gone.  
2. **O1** — tables + MCP + inspector links. Empty ontology is a valid state (O2 empty/error criteria).  
3. **Brain-shape FDG** — Barnes–Hut in a worker, hull constraint, tag attractors, hover synapses. Palette / camera / electric look unchanged. Version/tag first.  
4. **Ontology layout** — Sugiyama (or ELK) on `is-a` for the schema; instances under their class; transversal edges color-coded and toggleable.  
5. **O3** — proposals fill the graph so the layout has something true to show.

Do not wait on hub/Postgres for solo O1; SQLite can hold entities/links. Hub T1b ([#131](https://github.com/duketopceo/kurultai/issues/131) / objects+edges) is the shared-store follow-on.

## Explicit non-goals (this research)

- Implementing layout or schema in the same change as this note  
- Parametric cartoon-brain polynomial instead of the GLB we already embed  
- GPU WebGPU sim as a first slice  
- Shipping OWL/SPARQL in the daemon  
- Keeping galaxy as a third peer mode  
- Mapping tags onto real Brodmann areas / “cerebellum = procedures” without a maintained atlas

## Sources

- [The Barnes-Hut Approximation (Jeffrey Heer)](https://jheer.github.io/barnes-hut/)
- [d3-force-3d (octree many-body)](https://github.com/vasturiano/d3-force-3d/)
- [3D force-directed graphs with Three.js (workers, 10k-node cost)](https://intelligentgraphicandcode.com/development/threejs-interfaces/network-graphs)
- [vibe-graph-layout-gpu (WebGPU Barnes–Hut claims)](https://crates.io/crates/vibe-graph-layout-gpu)
- [Zhang, Boundary Constraints in Force-Directed Graph Layout (UCSC, 2014)](https://escholarship.org/uc/item/0vd969mx)
- [JGAA: graph layout with versatile boundary constraints](https://jgaa.info/index.php/jgaa/article/download/paper401/2568/2375)
- [Layered graph drawing / Sugiyama (Wikipedia)](https://en.wikipedia.org/wiki/Layered_graph_drawing)
- [GD Handbook ch. 13, Hierarchical Drawing Algorithms](https://cs.brown.edu/people/rtamassi/gdhandbook/chapters/hierarchical.pdf)
- [Lohmann et al., Visualizing ontologies with VOWL (Semantic Web, 2016)](https://journals.sagepub.com/doi/10.3233/SW-150200)
- [Ontodia](https://github.com/metaphacts/ontodia) · [ESWC 2015 poster](https://ceur-ws.org/Vol-1486/paper_77.pdf)
- [Property graph vs RDF (TigerGraph)](https://www.tigergraph.com/blog/property-graph-vs-rdf/)
- [RDF vs property graphs (Neo4j)](https://neo4j.com/blog/knowledge-graph/rdf-vs-property-graphs-knowledge-graphs/)
- Kurultai: [#116](https://github.com/duketopceo/kurultai/issues/116), [#117](https://github.com/duketopceo/kurultai/issues/117), [#118](https://github.com/duketopceo/kurultai/issues/118)
