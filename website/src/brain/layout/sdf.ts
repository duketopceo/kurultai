import type { SignedDistanceField } from './types.ts';

const HEADER_FLOATS = 7;

export function packSdf(sdf: SignedDistanceField): ArrayBuffer {
  const buf = new ArrayBuffer((HEADER_FLOATS + sdf.values.length) * 4);
  const f = new Float32Array(buf);
  f[0] = sdf.nx;
  f[1] = sdf.ny;
  f[2] = sdf.nz;
  f[3] = sdf.originX;
  f[4] = sdf.originY;
  f[5] = sdf.originZ;
  f[6] = sdf.cell;
  f.set(sdf.values, HEADER_FLOATS);
  return buf;
}

export function unpackSdf(buf: ArrayBuffer): SignedDistanceField | null {
  if (buf.byteLength < HEADER_FLOATS * 4) return null;
  const f = new Float32Array(buf);
  const nx = f[0] | 0;
  const ny = f[1] | 0;
  const nz = f[2] | 0;
  if (nx < 2 || ny < 2 || nz < 2) return null;
  const values = f.subarray(HEADER_FLOATS);
  if (values.length !== nx * ny * nz) return null;
  return {
    nx,
    ny,
    nz,
    originX: f[3],
    originY: f[4],
    originZ: f[5],
    cell: f[6],
    values: new Float32Array(values),
  };
}

export function sampleSdf(sdf: SignedDistanceField, x: number, y: number, z: number): number {
  const gx = (x - sdf.originX) / sdf.cell;
  const gy = (y - sdf.originY) / sdf.cell;
  const gz = (z - sdf.originZ) / sdf.cell;
  const x0 = Math.floor(gx);
  const y0 = Math.floor(gy);
  const z0 = Math.floor(gz);
  const x1 = x0 + 1;
  const y1 = y0 + 1;
  const z1 = z0 + 1;
  const tx = gx - x0;
  const ty = gy - y0;
  const tz = gz - z0;
  const c000 = at(sdf, x0, y0, z0);
  const c100 = at(sdf, x1, y0, z0);
  const c010 = at(sdf, x0, y1, z0);
  const c110 = at(sdf, x1, y1, z0);
  const c001 = at(sdf, x0, y0, z1);
  const c101 = at(sdf, x1, y0, z1);
  const c011 = at(sdf, x0, y1, z1);
  const c111 = at(sdf, x1, y1, z1);
  const c00 = c000 + (c100 - c000) * tx;
  const c10 = c010 + (c110 - c010) * tx;
  const c01 = c001 + (c101 - c001) * tx;
  const c11 = c011 + (c111 - c011) * tx;
  const c0 = c00 + (c10 - c00) * ty;
  const c1 = c01 + (c11 - c01) * ty;
  return c0 + (c1 - c0) * tz;
}

export function sdfGradient(
  sdf: SignedDistanceField,
  x: number,
  y: number,
  z: number,
): { x: number; y: number; z: number } {
  const h = sdf.cell;
  return {
    x: (sampleSdf(sdf, x + h, y, z) - sampleSdf(sdf, x - h, y, z)) / (2 * h),
    y: (sampleSdf(sdf, x, y + h, z) - sampleSdf(sdf, x, y - h, z)) / (2 * h),
    z: (sampleSdf(sdf, x, y, z + h) - sampleSdf(sdf, x, y, z - h)) / (2 * h),
  };
}

export function makeSphereSdf(radius: number, resolution = 24): SignedDistanceField {
  const pad = radius * 2.5;
  const origin = -pad;
  const cell = (pad * 2) / (resolution - 1);
  const values = new Float32Array(resolution * resolution * resolution);
  let i = 0;
  for (let z = 0; z < resolution; z++) {
    const wz = origin + z * cell;
    for (let y = 0; y < resolution; y++) {
      const wy = origin + y * cell;
      for (let x = 0; x < resolution; x++) {
        const wx = origin + x * cell;
        values[i++] = Math.hypot(wx, wy, wz) - radius;
      }
    }
  }
  return {
    nx: resolution,
    ny: resolution,
    nz: resolution,
    originX: origin,
    originY: origin,
    originZ: origin,
    cell,
    values,
  };
}

/** Bake an SDF from a triangle mesh. `indices` may be null for a packed triangle list. */
export function bakeSdfFromPositions(
  positions: Float32Array,
  indices: ArrayLike<number> | null,
  resolution = 32,
): SignedDistanceField | null {
  const triCount = triangleCount(positions, indices);
  if (triCount === 0) return null;

  let minX = Infinity;
  let minY = Infinity;
  let minZ = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  let maxZ = -Infinity;
  for (let i = 0; i < positions.length; i += 3) {
    const x = positions[i];
    const y = positions[i + 1];
    const z = positions[i + 2];
    if (x < minX) minX = x;
    if (y < minY) minY = y;
    if (z < minZ) minZ = z;
    if (x > maxX) maxX = x;
    if (y > maxY) maxY = y;
    if (z > maxZ) maxZ = z;
  }
  const dx = maxX - minX;
  const dy = maxY - minY;
  const dz = maxZ - minZ;
  const padX = dx * 0.05 + 1e-5;
  const padY = dy * 0.05 + 1e-5;
  const padZ = dz * 0.05 + 1e-5;
  minX -= padX;
  minY -= padY;
  minZ -= padZ;
  maxX += padX;
  maxY += padY;
  maxZ += padZ;
  const nx = resolution;
  const ny = resolution;
  const nz = resolution;
  const cellX = (maxX - minX) / (nx - 1);
  const cellY = (maxY - minY) / (ny - 1);
  const cellZ = (maxZ - minZ) / (nz - 1);
  const cell = Math.max(cellX, cellY, cellZ);

  const binsY = ny;
  const binsZ = nz;
  const buckets: number[][] = Array.from({ length: binsY * binsZ }, () => []);
  const tris: number[] = [];
  visitTriangles(positions, indices, (ax, ay, az, bx, by, bz, cx, cy, cz) => {
    const t = tris.length / 9;
    tris.push(ax, ay, az, bx, by, bz, cx, cy, cz);
    const tMinY = Math.min(ay, by, cy);
    const tMaxY = Math.max(ay, by, cy);
    const tMinZ = Math.min(az, bz, cz);
    const tMaxZ = Math.max(az, bz, cz);
    const y0 = clamp(Math.floor((tMinY - minY) / cell) - 1, 0, binsY - 1);
    const y1 = clamp(Math.floor((tMaxY - minY) / cell) + 1, 0, binsY - 1);
    const z0 = clamp(Math.floor((tMinZ - minZ) / cell) - 1, 0, binsZ - 1);
    const z1 = clamp(Math.floor((tMaxZ - minZ) / cell) + 1, 0, binsZ - 1);
    for (let y = y0; y <= y1; y++) {
      for (let z = z0; z <= z1; z++) buckets[y * binsZ + z].push(t);
    }
  });

  const occupancy = new Int8Array(nx * ny * nz);
  for (let z = 0; z < nz; z++) {
    const wz = minZ + z * cell + cell * 0.013;
    for (let y = 0; y < ny; y++) {
      const wy = minY + y * cell + cell * 0.02;
      const by = clamp(Math.floor((wy - minY) / cell), 0, binsY - 1);
      const bz = clamp(Math.floor((wz - minZ) / cell), 0, binsZ - 1);
      const bucket = buckets[by * binsZ + bz] ?? [];
      for (let x = 0; x < nx; x++) {
        const wx = minX + x * cell + cell * 0.007;
        let hits = 0;
        for (const t of bucket) {
          const o = t * 9;
          if (
            rayHitPlusX(
              wx,
              wy,
              wz,
              tris[o],
              tris[o + 1],
              tris[o + 2],
              tris[o + 3],
              tris[o + 4],
              tris[o + 5],
              tris[o + 6],
              tris[o + 7],
              tris[o + 8],
            )
          ) {
            hits++;
          }
        }
        occupancy[x + nx * (y + ny * z)] = hits % 2 === 1 ? 1 : 0;
      }
    }
  }

  const values = distanceTransform(occupancy, nx, ny, nz, cell);
  return {
    nx,
    ny,
    nz,
    originX: minX,
    originY: minY,
    originZ: minZ,
    cell,
    values,
  };
}

function triangleCount(positions: Float32Array, indices: ArrayLike<number> | null): number {
  if (indices) return Math.floor(indices.length / 3);
  return Math.floor(positions.length / 9);
}

function visitTriangles(
  positions: Float32Array,
  indices: ArrayLike<number> | null,
  fn: (
    ax: number,
    ay: number,
    az: number,
    bx: number,
    by: number,
    bz: number,
    cx: number,
    cy: number,
    cz: number,
  ) => void,
): void {
  if (indices) {
    const n = Math.floor(indices.length / 3) * 3;
    for (let i = 0; i < n; i += 3) {
      const ia = indices[i] * 3;
      const ib = indices[i + 1] * 3;
      const ic = indices[i + 2] * 3;
      fn(
        positions[ia],
        positions[ia + 1],
        positions[ia + 2],
        positions[ib],
        positions[ib + 1],
        positions[ib + 2],
        positions[ic],
        positions[ic + 1],
        positions[ic + 2],
      );
    }
    return;
  }
  for (let i = 0; i + 8 < positions.length; i += 9) {
    fn(
      positions[i],
      positions[i + 1],
      positions[i + 2],
      positions[i + 3],
      positions[i + 4],
      positions[i + 5],
      positions[i + 6],
      positions[i + 7],
      positions[i + 8],
    );
  }
}

function rayHitPlusX(
  ox: number,
  oy: number,
  oz: number,
  ax: number,
  ay: number,
  az: number,
  bx: number,
  by: number,
  bz: number,
  cx: number,
  cy: number,
  cz: number,
): boolean {
  const t = intersectRayTriangle(ox, oy, oz, 1, 0, 0, ax, ay, az, bx, by, bz, cx, cy, cz);
  return t !== null && t > 1e-8;
}

function intersectRayTriangle(
  ox: number,
  oy: number,
  oz: number,
  dx: number,
  dy: number,
  dz: number,
  ax: number,
  ay: number,
  az: number,
  bx: number,
  by: number,
  bz: number,
  cx: number,
  cy: number,
  cz: number,
): number | null {
  const eps = 1e-8;
  const e1x = bx - ax;
  const e1y = by - ay;
  const e1z = bz - az;
  const e2x = cx - ax;
  const e2y = cy - ay;
  const e2z = cz - az;
  const px = dy * e2z - dz * e2y;
  const py = dz * e2x - dx * e2z;
  const pz = dx * e2y - dy * e2x;
  const det = e1x * px + e1y * py + e1z * pz;
  if (det > -eps && det < eps) return null;
  const inv = 1 / det;
  const tx = ox - ax;
  const ty = oy - ay;
  const tz = oz - az;
  const u = (tx * px + ty * py + tz * pz) * inv;
  if (u < 0 || u > 1) return null;
  const qx = ty * e1z - tz * e1y;
  const qy = tz * e1x - tx * e1z;
  const qz = tx * e1y - ty * e1x;
  const v = (dx * qx + dy * qy + dz * qz) * inv;
  if (v < 0 || u + v > 1) return null;
  const t = (e2x * qx + e2y * qy + e2z * qz) * inv;
  return t > eps ? t : null;
}

function distanceTransform(
  occ: Int8Array,
  nx: number,
  ny: number,
  nz: number,
  cell: number,
): Float32Array {
  const n = nx * ny * nz;
  const dist = new Float32Array(n);
  const INF = nx + ny + nz;
  for (let i = 0; i < n; i++) dist[i] = occ[i] ? 0 : INF;

  const idx = (x: number, y: number, z: number) => x + nx * (y + ny * z);
  for (let z = 0; z < nz; z++) {
    for (let y = 0; y < ny; y++) {
      for (let x = 0; x < nx; x++) {
        const i = idx(x, y, z);
        let d = dist[i];
        if (x > 0) d = Math.min(d, dist[idx(x - 1, y, z)] + 1);
        if (y > 0) d = Math.min(d, dist[idx(x, y - 1, z)] + 1);
        if (z > 0) d = Math.min(d, dist[idx(x, y, z - 1)] + 1);
        dist[i] = d;
      }
    }
  }
  for (let z = nz - 1; z >= 0; z--) {
    for (let y = ny - 1; y >= 0; y--) {
      for (let x = nx - 1; x >= 0; x--) {
        const i = idx(x, y, z);
        let d = dist[i];
        if (x + 1 < nx) d = Math.min(d, dist[idx(x + 1, y, z)] + 1);
        if (y + 1 < ny) d = Math.min(d, dist[idx(x, y + 1, z)] + 1);
        if (z + 1 < nz) d = Math.min(d, dist[idx(x, y, z + 1)] + 1);
        dist[i] = d;
      }
    }
  }

  const out = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const d = dist[i] * cell;
    out[i] = occ[i] ? -Math.max(d, cell * 0.5) : d;
  }
  return out;
}

function at(sdf: SignedDistanceField, x: number, y: number, z: number): number {
  const xi = clamp(x, 0, sdf.nx - 1);
  const yi = clamp(y, 0, sdf.ny - 1);
  const zi = clamp(z, 0, sdf.nz - 1);
  return sdf.values[xi + sdf.nx * (yi + sdf.ny * zi)];
}

function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v;
}
