/**
 * Decoded cortex model (baked from brain.glb by scripts/bake-cortex.mjs): low-poly mesh for the wireframe
 * and fresnel shell, an interior voxel mask for atom placement, and lobe seeds (k-means over the interior)
 * that parcellate the volume into contiguous regions. Pure; no three.js import so tests run under node.
 */
import { CORTEX_GRID, CORTEX_LOBE_SEEDS, CORTEX_MASK_BITS, CORTEX_MESH_IDX, CORTEX_MESH_POS } from './cortexData.ts';

const bytes = (b64: string): Uint8Array => {
  const s = atob(b64);
  const u = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) u[i] = s.charCodeAt(i);
  return u;
};

const posQ = new Int16Array(bytes(CORTEX_MESH_POS).buffer);
export const MESH_POS = Float32Array.from(posQ, (v) => v / 32767);
export const MESH_IDX = new Uint16Array(bytes(CORTEX_MESH_IDX).buffer);
export const LOBE_SEEDS = CORTEX_LOBE_SEEDS;
export const LOBES = LOBE_SEEDS.length;

const RES = CORTEX_GRID.res;
const CELL = CORTEX_GRID.cell;
const [OX, OY, OZ] = CORTEX_GRID.origin;
const bits = bytes(CORTEX_MASK_BITS);
const voxelOn = (i: number) => (bits[i >> 3] >> (i & 7)) & 1;

/** True when the point lies in an interior voxel of the cortex model. */
export function insideCortex(x: number, y: number, z: number): boolean {
  const ix = Math.round((x - OX) / CELL);
  const iy = Math.round((y - OY) / CELL);
  const iz = Math.round((z - OZ) / CELL);
  if (ix < 0 || iy < 0 || iz < 0 || ix >= RES || iy >= RES || iz >= RES) return false;
  return voxelOn(ix + RES * (iy + RES * iz)) === 1;
}

export function nearestLobe(x: number, y: number, z: number): number {
  let best = 0;
  let bd = Infinity;
  for (let i = 0; i < LOBES; i++) {
    const s = LOBE_SEEDS[i];
    const d = (x - s[0]) ** 2 + (y - s[1]) ** 2 + (z - s[2]) ** 2;
    if (d < bd) {
      bd = d;
      best = i;
    }
  }
  return best;
}

/** Interior voxel centres grouped by lobe (xyz triples). */
export const LOBE_VOXELS: Float32Array[] = (() => {
  const lists: number[][] = Array.from({ length: LOBES }, () => []);
  for (let z = 0; z < RES; z++) {
    for (let y = 0; y < RES; y++) {
      for (let x = 0; x < RES; x++) {
        if (!voxelOn(x + RES * (y + RES * z))) continue;
        const px = OX + x * CELL;
        const py = OY + y * CELL;
        const pz = OZ + z * CELL;
        lists[nearestLobe(px, py, pz)].push(px, py, pz);
      }
    }
  }
  return lists.map((l) => Float32Array.from(l));
})();

export const VOXEL_CELL = CELL;

/** Lobe of each mesh vertex (for per-region shell colouring + boundary outlines). */
export const MESH_VERTEX_LOBE: Uint8Array = (() => {
  const n = MESH_POS.length / 3;
  const out = new Uint8Array(n);
  for (let i = 0; i < n; i++) out[i] = nearestLobe(MESH_POS[i * 3], MESH_POS[i * 3 + 1], MESH_POS[i * 3 + 2]);
  return out;
})();
