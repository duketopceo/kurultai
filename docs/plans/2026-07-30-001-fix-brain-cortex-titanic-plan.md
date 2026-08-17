---
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: code
product_contract_source: ce-plan-bootstrap
date: 2026-07-30
title: "fix: Brain cortex ghost boxes, surface-conformant edges, titanic removal"
---

# fix: Brain cortex ghost boxes, surface-conformant edges, titanic removal

## Summary

Three fixes for the Brain UI and knowledge store: (1) replace wireframe-box cortex particles with GPU point sprites so the brain surface reads as a smooth particle field instead of visible box outlines, (2) make synapse connection lines follow the brain mesh surface curvature using Catmull-Rom curves with projected intermediate points instead of cutting through the interior volume, (3) add a CLI `delete` subcommand and use it to fully remove the 891-atom titanic test dataset from the knowledge store.

## Problem Frame

The Brain UI (`/ui/` served by the daemon) has two visual defects and the knowledge store has one stale test dataset:

1. **Ghost telemetry boxes**: The brain cortex surface is rendered as instanced `BoxGeometry(0.004)` with `wireframe: true` and a custom ShaderMaterial. Each particle draws as a visible 12-line box outline. Depending on zoom and vertex density, these read as floating "ghost boxes" rather than a smooth electric particle field. The prior electric-pulse plan (`docs/plans/2026-07-29-003-feat-brain-electric-pulse-plan.md`) explicitly called for "nodes look smooth (not faceted boxes)" but the cortex particles themselves were never addressed.

2. **Connection lines ignore brain shape**: Synapse edges use `QuadraticBezierCurve3(a, (a+b)/2 + outward_lift, b)`. The midpoint is a simple average pushed from the origin by 18% of the A-B distance. Lines cut straight through the brain's interior volume rather than wrapping along the cortex surface.

3. **Titanic test dataset**: 891 passenger atoms indexed under source `"titanic"` remain in the knowledge store (`knowledge_atoms`, `atoms_fts`, `atoms_vec`, `merge_candidates`). The source is not in the config file and was loaded as test data. The store trait has `delete_source()` but no CLI command or API endpoint exposes it.

## Requirements

- R1: Cortex particles render as smooth GPU point sprites (`gl.POINTS`) with no visible box outlines, preserving the existing electric-flare shader behavior (`uPointer`, `uHover`, `uTime`, `uIntro` uniforms, additive blending, flicker, proximity flare).
- R2: Synapse edges follow the brain mesh surface using a Catmull-Rom curve with 4-6 intermediate points projected onto the brain mesh surface along normals, replacing the current quadratic bezier through the interior.
- R3: A CLI subcommand `kurultai delete --source <name>` invokes `store.delete_source()` to atomically remove all atoms, FTS entries, and vector entries for a given source.
- R4: Running `kurultai delete --source titanic` removes all 891 titanic atoms from `knowledge_atoms`, `atoms_fts`, `atoms_vec`, and cleans associated `merge_candidates` rows.
- R5: AGENTS.md visual constraints are honored: deep black background, black/white/slight purple only, electric zap/shimmer, hover highlights connections, no new layout modes or dashboard chrome.
- R6: Changes to `website/src/` are rebuildable via `npm run build` (Vite outputs to `ui/`) and embed correctly in the daemon binary via `rust_embed`.

## Key Technical Decisions

### KTD1: GPU point sprites for cortex particles

**Decision:** Replace `BoxGeometry + wireframe: true + InstancedBufferGeometry` with `THREE.Points` + `ShaderMaterial` using `gl.POINTS`.

**Rationale:** Point sprites are the cheapest and smoothest representation for a particle field. They eliminate box outlines entirely, reduce GPU vertex count (1 vertex per particle vs 24 per box instance), and preserve the existing shader uniforms (`uPointer`, `uHover`, `uTime`, `uIntro`) with minimal shader changes. The `aOffset`, `aRotation`, `aSize`, `aColor`, `aSeed` instanced attributes map directly to `THREE.BufferGeometry` attributes on a `Points` object.

**Alternatives rejected:**
- Filled non-wireframe cubes (remove `wireframe: true`): minimal change but still 24 vertices per particle, and small filled cubes still read as faceted at certain zoom levels.
- Small filled tetrahedra: 3D volume feel but 4 vertices per particle and more complex geometry management.

### KTD2: Catmull-Rom with surface-projected intermediate points for edges

**Decision:** Replace `QuadraticBezierCurve3(a, mid, b)` with a `CatmullRomCurve3` passing through 4-6 intermediate points, each projected onto the brain mesh surface along the vertex normal at the nearest mesh vertex.

**Rationale:** The brain GLB model's vertex positions and normals are already loaded in `BrainView` (`this.verts`, `this.norms`). Projecting intermediate points onto the surface makes edges wrap along the cortex instead of cutting through the interior. Catmull-Rom through 4-6 control points gives smooth curvature without the stiffness of a quadratic bezier. The `proxy` mesh (invisible, used for raycasting) provides the surface geometry for projection via raycasting from the curve midpoint outward.

**Alternatives rejected:**
- Projected quadratic bezier (2-3 control points): fewer control points means less surface conformance for long edges.
- Simple outward lift (increase current lift factor): approximate, doesn't actually follow the surface.

### KTD3: CLI delete subcommand for source removal

**Decision:** Add a `kurultai delete` CLI subcommand with `--source <name>` flag that opens the store, calls `delete_source()`, and reports the count removed.

**Rationale:** The store trait already implements `delete_source()` with transactional cleanup of `knowledge_atoms`, `atoms_fts`, and `atoms_vec`. Adding a thin CLI wrapper is the cleanest way to invoke it without exposing a destructive DELETE API endpoint. The command also cleans `merge_candidates` rows referencing deleted atoms.

**Alternatives rejected:**
- Direct SQL manipulation: bypasses the store's transactional logic and risks orphaned FTS/vec entries.
- HTTP DELETE endpoint: exposes a destructive operation over the network; unnecessary for a local-first tool.

## Implementation Units

### U1. Replace wireframe box particles with GPU point sprites

**Goal:** Eliminate ghost box outlines on the brain cortex surface by switching from instanced wireframe `BoxGeometry` to `THREE.Points` with `gl.POINTS`.

**Requirements:** R1, R5, R6

**Dependencies:** None

**Files:**
- `website/src/brain/BrainView.ts` — `buildParticles()` method, `PARTICLE_VERTEX`/`PARTICLE_FRAGMENT` shaders, particle-related fields
- `website/src/brain/BrainView.ts` — `loadModel()` (particle count from mesh vertex count, unchanged)

**Approach:**
1. Replace the `BoxGeometry + InstancedBufferGeometry` construction in `buildParticles()` with a `THREE.BufferGeometry` carrying the same instanced attributes (`aOffset`, `aRotation`, `aSize`, `aColor`, `aSeed`) as regular attributes on a `Points` object.
2. Adapt `PARTICLE_VERTEX` shader: replace `position` (box vertex) with `gl_PointSize` and `gl_PointCoord` for point sprite rendering. The `aOffset` becomes the point position directly. Size scaling from `aSize + c * 7.0 * uHover` maps to `gl_PointSize`. Rotation can be applied via `gl_PointCoord` transform in the fragment shader or dropped (point sprites are rotation-invariant circles).
3. Adapt `PARTICLE_FRAGMENT` shader: use `gl_PointCoord` to create a circular soft-edge discard (distance from center > 0.5 → discard), preserving additive blending and the `vColor`/`vAlpha` varyings.
4. Replace `new THREE.Mesh(geometry, material)` with `new THREE.Points(geometry, material)`. Remove `wireframe: true` (not applicable to points). Keep `transparent: true`, `depthWrite: false`, `blending: THREE.AdditiveBlending`.
5. Keep `particles.frustumCulled = false` and the `brainGroup.add(particles)` wiring.
6. The `particleColorAttr` and `particleColorIndex` fields remain for theme switching via `setTheme()`.

**Patterns to follow:** The existing shader uniform structure (`uPointer`, `uHover`, `uTime`, `uIntro`) and the `setTheme()` color-update path in `BrainView.ts`.

**Test scenarios:**
- Happy path: Cortex surface renders as smooth particle field with no visible box outlines at default zoom.
- Edge case: Particles still respond to cursor proximity flare (`uPointer`/`uHover` uniforms) with the same electric brightening and size scaling.
- Edge case: `uIntro` animation ramp (0 to 1) still scales particles in on load.
- Edge case: Theme toggle (dark/light) updates particle colors via `particleColorAttr` without regenerating geometry.
- Edge case: `prefers-reduced-motion` disables time-based flicker and drift but particles still render.
- Performance: Particle count equals mesh vertex count (same as before); GPU vertex count drops from 24N to N.

**Verification:** Run `npm run build` in `website/`, restart daemon, open `http://127.0.0.1:8421/ui/`, confirm no box outlines on the brain surface and the electric flare still follows the cursor.

---

### U2. Surface-conformant synapse edges via Catmull-Rom projection

**Goal:** Make connection lines wrap along the brain mesh surface instead of cutting through the interior volume.

**Requirements:** R2, R5, R6

**Dependencies:** U1 (both touch `BrainView.ts`; U1 should land first to avoid merge conflicts)

**Files:**
- `website/src/brain/BrainView.ts` — `setData()` link-rendering loop (the `links.slice().sort().slice(0, MAX_EDGES).forEach(...)` block)

**Approach:**
1. Replace the `QuadraticBezierCurve3(a, mid, b)` construction with a `CatmullRomCurve3` through 4-6 control points.
2. Generate intermediate points along the straight line from `a.position` to `b.position` at equal parametric intervals (e.g., t = 0.2, 0.4, 0.6, 0.8 for 4 intermediates).
3. For each intermediate point, project it onto the brain mesh surface: raycast from the point outward along the direction from the brain center (origin) through the point, using the `proxy` mesh. If the ray hits, use the hit point as the control point. If no hit (edge wraps around the far side), use the original point pushed outward along the nearest vertex normal by a small offset.
4. Construct the `CatmullRomCurve3` through `[a.position, ...projectedPoints, b.position]` and sample 24-30 points for the `BufferGeometry`.
5. Keep the existing `LineBasicMaterial` with `edgeRest` color, `opacity: 0.2`, `depthWrite: false`. The hover highlight path (`highlightConnections`/`clearHover`) already sets `edgeActive`/`edgeDim` colors and opacity — no changes needed there.
6. Cap the projection cost: the `proxy` mesh raycast is already available. For performance with `MAX_EDGES = 800`, each edge does 4 raycasts (4 intermediate points). This is a build-time cost (edges are built once in `setData()`), not per-frame.

**Technical design (directional):**

```
// Pseudocode for edge construction
for each link (a, b):
  intermediates = 4 points along a→b at t=0.2,0.4,0.6,0.8
  for each point p in intermediates:
    ray = from origin through p, normalized
    hit = raycaster.intersectObject(proxy, ray from p outward)
    if hit: p = hit.point  // snap to surface
    else: p += nearestVertexNormal(p) * 0.02  // push outward
  curve = CatmullRomCurve3([a.pos, ...intermediates, b.pos])
  geo = BufferGeometry.fromPoints(curve.getPoints(28))
  line = Line(geo, LineBasicMaterial(edgeRest, opacity 0.2))
```

**Patterns to follow:** The existing `proxy` mesh raycast pattern in `onPointerMove()` (raycasting against `this.proxy` for cursor proximity). The `highlightConnections()` method's edge opacity/color management.

**Test scenarios:**
- Happy path: Edges visibly wrap along the brain surface curvature instead of cutting through the interior.
- Edge case: Edges between nodes on opposite hemispheres route over the surface, not through the center.
- Edge case: Hover highlight still brightens connected edges to `edgeActive` with opacity 0.9.
- Edge case: `showSynapses = false` hides all edges; `connectionThreshold` filtering still works.
- Edge case: Solar layout mode — edges still render (positions are orbital, not cortex-pinned; projection may not apply since proxy mesh is only in regions mode; fall back to simple bezier in solar mode).
- Performance: 800 edges with 4 raycasts each at build time completes in under 100ms (one-time cost, not per-frame).

**Verification:** Run `npm run build`, restart daemon, open Brain UI, hover a node and confirm edges wrap along the brain surface rather than passing through the interior.

---

### U3. Add CLI delete subcommand and remove titanic test data

**Goal:** Add a `kurultai delete --source <name>` CLI command and use it to remove all 891 titanic atoms from the knowledge store.

**Requirements:** R3, R4

**Dependencies:** None (independent of U1/U2)

**Files:**
- `src/cli.rs` — add `Delete` subcommand to the `Commands` enum and handler
- `src/store/mod.rs` — `delete_source()` already exists; verify it cleans `merge_candidates` or add cleanup
- `src/store/mod.rs` — add `merge_candidates` cleanup to `delete_source()` if not already present

**Approach:**
1. Add a `Delete` variant to the `Commands` enum in `src/cli.rs` with a `--source` flag (required, string).
2. In the `Delete` handler: open the config, open the store, call `store.delete_source(&source)`, print the count of atoms removed (query `knowledge_atoms` count before deletion for the report).
3. Verify `delete_source()` in `src/store/mod.rs` (lines ~719-751) also cleans `merge_candidates` rows where `atom_a` or `atom_b` matches any deleted atom ID. If not, add a `DELETE FROM merge_candidates WHERE atom_a IN (...) OR atom_b IN (...)` step within the existing transaction.
4. Run `kurultai delete --source titanic` to remove all 891 atoms, their FTS entries, vector entries, and merge candidate references.
5. Verify the store is clean: `sqlite3 store.db "SELECT COUNT(*) FROM knowledge_atoms WHERE source = 'titanic'"` returns 0.

**Patterns to follow:** Existing CLI subcommand structure in `src/cli.rs` (e.g., `Index`, `Search`). The `delete_source()` transaction pattern in `src/store/mod.rs`.

**Test scenarios:**
- Happy path: `kurultai delete --source titanic` removes all 891 atoms and prints a confirmation count.
- Edge case: `kurultai delete --source nonexistent` prints "0 atoms removed" and exits cleanly.
- Edge case: `merge_candidates` rows referencing titanic atoms are removed; rows referencing only non-titanic atoms remain.
- Edge case: FTS search for "titanic" returns zero results after deletion.
- Edge case: `atoms_vec` entries for titanic atom IDs are removed (no orphaned vectors).

**Verification:** Run `kurultai delete --source titanic`, then `kurultai search titanic` returns no results, and `sqlite3 ~/.local/share/kurultai/dev/store.db "SELECT COUNT(*) FROM knowledge_atoms WHERE source = 'titanic'"` returns 0.

---

## Scope Boundaries

### In scope
- `website/src/brain/BrainView.ts` particle and edge rendering
- `src/cli.rs` new `delete` subcommand
- `src/store/mod.rs` `delete_source()` merge_candidates cleanup
- Vite build to `ui/` and daemon embedding

### Out of scope
- Node positioning or new layout modes (AGENTS.md: "do not add circle or brain-shape layout modes")
- Dashboard panel, overlay, or tooltip changes
- Performance optimization for large graphs (tiered loading)
- Changes to `ui/index.html` (landing page) or `web/` (team app)
- Color palette changes (AGENTS.md: black/white/purple only)
- Ghost preview feature (the old `brain.js` ghost preview is not in the new Vite codebase)

### Deferred to follow-up work
- Adding an HTTP API endpoint for source deletion (CLI is sufficient for now)
- Migrating other stale test sources (`test-vault` has 16 atoms; separate cleanup)
- Rebuilding the daemon binary with `cargo build --release` after UI changes (implementation will handle this)

## Verification Contract

1. **U1**: `npm run build` succeeds in `website/`; brain surface shows no box outlines at `http://127.0.0.1:8421/ui/`
2. **U2**: Edges wrap along brain surface curvature; hover highlight still works; solar mode edges still render
3. **U3**: `kurultai delete --source titanic` exits 0; `sqlite3 store.db "SELECT COUNT(*) FROM knowledge_atoms WHERE source='titanic'"` returns 0; `kurultai search titanic` returns no results
4. **Build**: `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all -- --check` pass
5. **TypeScript**: `npx tsc --noEmit` passes in `website/`

## Definition of Done

All three implementation units are complete, the Vite build succeeds, the daemon serves the updated brain UI with smooth particles and surface-conformant edges, and the titanic source is fully removed from the knowledge store with zero residual atoms, FTS entries, vectors, or merge candidates.
