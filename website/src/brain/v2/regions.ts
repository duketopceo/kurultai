/**
 * Tag-region parcellation of the cortex hull. Pure + deterministic so it is
 * unit-testable and O(n) at 50k+ atoms (no force layout).
 */
import { insideCortex, LOBES, LOBE_SEEDS, LOBE_VOXELS, nearestLobe, VOXEL_CELL } from './cortex.ts';

export const MAX_REGIONS = LOBES;
export const OTHER = 'other';
export const UNTAGGED = 'untagged';

export interface Region {
  key: string;
  count: number;
  /** Index of the cortex lobe (contiguous volume chunk) this tag owns. */
  lobe: number;
  center: [number, number, number];
  /** 0..1: how full the region is (drives wire-chunk brightness). */
  fill: number;
}

export interface Parcellation {
  regions: Region[];
  /** atom index -> region index */
  regionOf: Uint16Array;
}

/** True when the point is inside the cortex model (voxel mask baked from brain.glb). */
export const insideHull = insideCortex;

/** FNV-1a -> [0,1) stream, seeded by string. */
export function hashStream(seed: string): () => number {
  let h = 2166136261;
  for (let i = 0; i < seed.length; i++) {
    h ^= seed.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return () => {
    h ^= h << 13;
    h ^= h >>> 17;
    h ^= h << 5;
    return ((h >>> 0) % 1_000_003) / 1_000_003;
  };
}

/** The tag that names an atom's region: its globally most frequent tag (ties lexical). */
export function topTagOf(tags: string[], freq: Map<string, number>): string {
  let best = '';
  let bestN = -1;
  for (const t of tags) {
    const n = freq.get(t) ?? 0;
    if (n > bestN || (n === bestN && t < best)) {
      best = t;
      bestN = n;
    }
  }
  return best || UNTAGGED;
}

export function parcellate(atoms: { id: string; tags: string[] }[], maxRegions = MAX_REGIONS): Parcellation {
  const freq = new Map<string, number>();
  for (const a of atoms) for (const t of a.tags) freq.set(t, (freq.get(t) ?? 0) + 1);
  const keyOf = atoms.map((a) => topTagOf(a.tags, freq));
  const sizes = new Map<string, number>();
  for (const k of keyOf) sizes.set(k, (sizes.get(k) ?? 0) + 1);
  const ranked = [...sizes.entries()].sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : 1));
  const kept = ranked.slice(0, maxRegions).map((e) => e[0]);
  const hasOther = ranked.length > maxRegions;
  if (hasOther && !kept.includes(OTHER)) kept[kept.length - 1] = OTHER;
  const index = new Map(kept.map((k, i) => [k, i]));
  const otherIdx = index.get(OTHER) ?? kept.length - 1;
  const regionOf = new Uint16Array(atoms.length);
  const counts = new Array<number>(kept.length).fill(0);
  keyOf.forEach((k, i) => {
    const r = index.get(k) ?? otherIdx;
    regionOf[i] = Math.max(0, r);
    counts[Math.max(0, r)]++;
  });
  const regions: Region[] = kept.map((key, i) => ({
    key,
    count: counts[i],
    lobe: i,
    center: LOBE_SEEDS[i],
    // log-scaled saturation: a handful of atoms barely lights a lobe, ~150 fills it
    fill: Math.min(1, Math.log2(1 + counts[i]) / Math.log2(151)),
  }));
  return { regions, regionOf };
}

/** Deterministic position of an atom: a random interior voxel of its lobe plus sub-voxel jitter. */
export function placeAtom(id: string, lobe: number): [number, number, number] {
  const rnd = hashStream(id);
  const v = LOBE_VOXELS[lobe];
  if (!v.length) {
    const s = LOBE_SEEDS[lobe] ?? [0, 0, 0];
    return [s[0], s[1], s[2]];
  }
  const n = v.length / 3;
  const k = Math.floor(rnd() * n) * 3;
  const j = VOXEL_CELL * 0.95;
  return [v[k] + (rnd() - 0.5) * j, v[k + 1] + (rnd() - 0.5) * j, v[k + 2] + (rnd() - 0.5) * j];
}

export function placeAll(atoms: { id: string }[], p: Parcellation): Float32Array {
  const xyz = new Float32Array(atoms.length * 3);
  for (let i = 0; i < atoms.length; i++) {
    const [x, y, z] = placeAtom(atoms[i].id, p.regions[p.regionOf[i]]?.lobe ?? 0);
    xyz[i * 3] = x;
    xyz[i * 3 + 1] = y;
    xyz[i * 3 + 2] = z;
  }
  return xyz;
}

export { nearestLobe };

export type LodTier = 'far' | 'mid' | 'near';
/** Camera distance (hull units) -> LOD tier. Far: region glow + counts; near: atoms. */
export function lodTier(distance: number): LodTier {
  if (distance > 2.3) return 'far';
  if (distance > 1.4) return 'mid';
  return 'near';
}
