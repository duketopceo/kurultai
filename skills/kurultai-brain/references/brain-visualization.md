# Brain view: lobes, sectors, and why one blob swallows the picture

The WebGL brain at `work.shippedit.dev` is a **client-side drawing of the atom
table**. Nothing about lobes, sectors, or region labels exists in the API, the
server, or the ontology store — do not go looking for a `sector` field there.

## Layout model

- `website/src/assets/brain.glb` is decimated (~2k tris) and baked by
  `scripts/bake-cortex.mjs` into `website/src/brain/v2/cortexData.ts`: mesh,
  a 40^3 interior voxel mask, and **24 k-means lobe seeds** over the interior.
- `website/src/brain/v2/cortex.ts` decodes that and splits interior voxels into
  per-lobe lists (`LOBE_VOXELS`), so each lobe owns a contiguous chunk of volume.
- `website/src/brain/v2/regions.ts` assigns atoms to regions and places them.
- `website/src/brain/v2/BrainV2.ts` draws the shell, glow, atoms, synapses, and
  the top-10 region label pills.

The deployed bundle is `https://api-work.shippedit.dev/ui/assets/ui-next-*.js`
plus `types-*.js`; grep those to confirm what the live build actually does.

## The parcellation rule that produces "everything in one lobe"

`parcellate()` names each atom's region with `topTagOf()`: the atom's own tag
that is **most frequent across the whole corpus**. Regions are then ranked by
size and only the top 24 survive, the last slot becoming `other`.

So any umbrella tag present on nearly every atom (a workspace tag, a repo tag,
a language/extension tag) wins for nearly every atom, and the whole store piles
into lobe 0. The remaining lobes are then labelled with the only atoms that
lacked the umbrella tag — typically two or three stragglers — which is why tiny
pills like `clickonce · 1` and `agent · 1` show up while the rest of the cortex
is empty. In the light theme those thousands of coincident points render as one
solid black blotch.

Fill intensity is `min(1, log2(1 + count) / log2(151))`: about 150 atoms fills a
lobe, a handful barely glows. A lobe count that looks absurd next to a dense
cloud is a grouping bug, not a rendering bug — the label count and the point
count both come from the same array.

## Diagnosis, cheapest first

1. Pull the graph and count tags:
   `curl -s -H "Authorization: Bearer $KURULTAI_WORK_SEAT_KEY" \
    'https://api-work.shippedit.dev/api/graph?tier=hot'`
   then tally `tags` across all nodes.
2. If one tag covers most atoms, that is the whole explanation. Report the
tag histogram with the diagnosis; it is convincing and it points at the fix.
3. Only after that check the renderer (`BrainV2.ts` label placement, themes) for
the separate cosmetic issues — mobile header overlap, mesh clipping at the
viewport edge, label pills sitting over the dense cluster.

## Fix directions, in preference order

- **Local custom sections and tags config (preferred on `hermes-bartlett-omarchy`):**
  do not patch upstream `regions.ts`. Instead, maintain `/opt/data/kurultai-sources/sections-config.json`
  with `ignore_umbrella_tags` (stripping generic tags like `bartlett`, `code`, `md`, `py`)
  and `sections` rules (`match_tags` + `match_keywords` mapping items to domain sections
  such as `servicetitan`, `salestrak`, `middleman`, `data-platform`, `infrastructure`,
  `deployments`, `it-support`, `security`, `engineering-ops`, `hermes-ops`,
  `tech_questions`, `company_wide`, `meeting-notes`, `email`, `slack`).
  `kurultai_sync_sources.py` resolves each item's section and clean tag list before
  pushing via MCP `remember`, so `topTagOf` naturally groups lobes by real domain
  sections without upstream changes.
- Or, if modifying the frontend directly, demote generic tags before ranking or weight
  by inverse document frequency in `website/src/brain/v2/regions.ts`.

Whatever changes, keep `parcellate`/`placeAtom` pure and deterministic: the
node tests assert determinism, hull containment, region cap with an `other`
bucket, monotonic fill, and >90% of atoms staying inside their own lobe.
