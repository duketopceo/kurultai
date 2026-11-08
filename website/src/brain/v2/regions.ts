/**
 * Tag-region parcellation of the cortex hull. Pure + deterministic so it is
 * unit-testable and O(n) at 50k+ atoms (no force layout).
 */
export const HULL = { x: 0.78, y: 0.66, z: 1.0 } as const;
export const MAX_REGIONS = 24;
export const OTHER = 'other';
export const UNTAGGED = 'untagged';

export interface Region {
  key: string;
  count: number;
  center: [number, number, number];
  /** 0..1: how full the region is (drives wire-chunk brightness). */
  fill: number;
}

export interface Parcellation {
  regions: Region[];
  /** atom index -> region index */
  regionOf: Uint16Array;
}

/** True when the point is inside the low-poly brain hull (ellipsoid + longitudinal fissure). */
export function insideHull(x: number, y: number, z: number): boolean {
  const e = (x / HULL.x) ** 2 + (y / HULL.y) ** 2 + (z / HULL.z) ** 2;
  if (e > 1) return false;
  if (Math.abs(x) < 0.03 && y > 0.18) return false;
  return true;
}

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

/** Fibonacci points on a scaled ellipsoid shell (radius factor `r`). */
export function regionCenters(n: number, r = 0.66): [number, number, number][] {
  const out: [number, number, number][] = [];
  const golden = Math.PI * (3 - Math.sqrt(5));
  for (let i = 0; i < n; i++) {
    const y = n === 1 ? 0 : 1 - (i / (n - 1)) * 2;
    const rad = Math.sqrt(Math.max(0, 1 - y * y));
    const th = golden * i;
    out.push([Math.cos(th) * rad * HULL.x * r, y * HULL.y * r, Math.sin(th) * rad * HULL.z * r]);
  }
  return out;
}

export function nearestCenter(x: number, y: number, z: number, centers: [number, number, number][]): number {
  let best = 0;
  let bd = Infinity;
  for (let i = 0; i < centers.length; i++) {
    const c = centers[i];
    const d = (x - c[0]) ** 2 + (y - c[1]) ** 2 + (z - c[2]) ** 2;
    if (d < bd) {
      bd = d;
      best = i;
    }
  }
  return best;
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
  const centers = regionCenters(Math.max(1, kept.length));
  const mean = atoms.length / Math.max(1, kept.length);
  const regions: Region[] = kept.map((key, i) => ({
    key,
    count: counts[i],
    center: centers[i],
    // a region "fills" as it approaches an even share; a lone dominant region reaches 1 fast
    fill: Math.min(1, counts[i] / Math.max(8, mean)),
  }));
  return { regions, regionOf };
}

/** Deterministic position of an atom inside its region (Voronoi cell of the hull). */
export function placeAtom(id: string, region: number, centers: [number, number, number][]): [number, number, number] {
  const rnd = hashStream(id);
  const c = centers[region] ?? [0, 0, 0];
  let fallback: [number, number, number] = [c[0], c[1], c[2]];
  for (let t = 0; t < 48; t++) {
    // sample a box around the region centre, not the whole hull: cells are ~1/24 of the volume
    const x = c[0] + (rnd() * 2 - 1) * 0.42;
    const y = c[1] + (rnd() * 2 - 1) * 0.42;
    const z = c[2] + (rnd() * 2 - 1) * 0.5;
    if (!insideHull(x, y, z)) continue;
    if (nearestCenter(x, y, z, centers) !== region) continue;
    return [x, y, z];
  }
  // fall back to a jittered point near the centre (always inside: centre is at 0.55 radius)
  for (let t = 0; t < 8; t++) {
    const x = c[0] + (rnd() - 0.5) * 0.12;
    const y = c[1] + (rnd() - 0.5) * 0.12;
    const z = c[2] + (rnd() - 0.5) * 0.12;
    if (insideHull(x, y, z)) {
      fallback = [x, y, z];
      break;
    }
  }
  return fallback;
}

export function placeAll(atoms: { id: string }[], p: Parcellation): Float32Array {
  const centers = p.regions.map((r) => r.center);
  const xyz = new Float32Array(atoms.length * 3);
  for (let i = 0; i < atoms.length; i++) {
    const [x, y, z] = placeAtom(atoms[i].id, p.regionOf[i], centers);
    xyz[i * 3] = x;
    xyz[i * 3 + 1] = y;
    xyz[i * 3 + 2] = z;
  }
  return xyz;
}

export type LodTier = 'far' | 'mid' | 'near';
/** Camera distance (hull units) -> LOD tier. Far: region glow + counts; near: atoms. */
export function lodTier(distance: number): LodTier {
  if (distance > 2.6) return 'far';
  if (distance > 1.5) return 'mid';
  return 'near';
}
