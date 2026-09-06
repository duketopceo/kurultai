import { Octree } from './octree.ts';
import { sampleSdf, sdfGradient } from './sdf.ts';
import type { FdgLink, FdgNode, FdgParams, SignedDistanceField } from './types.ts';
import { DEFAULT_FDG_PARAMS } from './types.ts';

const EPS = 0.01;

/** One Verlet/Euler tick. Mutates `nodes` positions and velocities in place. */
export function tickFdg(
  nodes: FdgNode[],
  links: FdgLink[],
  sdf: SignedDistanceField | null,
  params: FdgParams = DEFAULT_FDG_PARAMS,
): void {
  const n = nodes.length;
  if (n === 0) return;
  if (n === 1) {
    nodes[0].x = 0;
    nodes[0].y = 0;
    nodes[0].z = 0;
    nodes[0].vx = 0;
    nodes[0].vy = 0;
    nodes[0].vz = 0;
    return;
  }

  const tree = new Octree(nodes, params.theta);
  const force = { x: 0, y: 0, z: 0 };
  for (let i = 0; i < n; i++) {
    tree.accumulate(i, force);
    nodes[i].vx += force.x * params.repulsion;
    nodes[i].vy += force.y * params.repulsion;
    nodes[i].vz += force.z * params.repulsion;
  }

  for (const link of links) {
    if (link.a === link.b || link.a < 0 || link.b < 0 || link.a >= n || link.b >= n) continue;
    const a = nodes[link.a];
    const b = nodes[link.b];
    const dx = b.x - a.x;
    const dy = b.y - a.y;
    const dz = b.z - a.z;
    const d = Math.sqrt(dx * dx + dy * dy + dz * dz) || EPS;
    const f = params.springK * (d - params.springRest) * (link.strength || 1);
    const nx = dx / d;
    const ny = dy / d;
    const nz = dz / d;
    a.vx += nx * f;
    a.vy += ny * f;
    a.vz += nz * f;
    b.vx -= nx * f;
    b.vy -= ny * f;
    b.vz -= nz * f;
  }

  if (params.tagK !== 0) applyTagAttract(nodes, params);

  for (let i = 0; i < n; i++) {
    const node = nodes[i];
    node.vx -= node.x * params.centerK;
    node.vy -= node.y * params.centerK;
    node.vz -= node.z * params.centerK;

    if (sdf && params.hullK !== 0) {
      const d = sampleSdf(sdf, node.x, node.y, node.z);
      const span = (sdf.nx - 1) * sdf.cell;
      const inGrid =
        node.x >= sdf.originX && node.x <= sdf.originX + span &&
        node.y >= sdf.originY && node.y <= sdf.originY + span &&
        node.z >= sdf.originZ && node.z <= sdf.originZ + span;
      if (d > 0 && inGrid) {
        const g = sdfGradient(sdf, node.x, node.y, node.z);
        const glen = Math.hypot(g.x, g.y, g.z);
        if (glen > 1e-6) {
          const mag = params.hullK * d;
          node.vx -= (g.x / glen) * mag;
          node.vy -= (g.y / glen) * mag;
          node.vz -= (g.z / glen) * mag;
        } else {
          node.vx -= node.x * params.hullK;
          node.vy -= node.y * params.hullK;
          node.vz -= node.z * params.hullK;
        }
      }
    }

    node.vx *= params.damping;
    node.vy *= params.damping;
    node.vz *= params.damping;
    node.x += node.vx;
    node.y += node.vy;
    node.z += node.vz;

    // Hard containment: the soft hull force above can lose equilibrium to
    // repulsion on dense graphs, letting nodes settle in a shell outside the
    // cortex. Project escapers back onto the surface and damp the outward
    // velocity so the layout stays volumetric, not a ring around the brain.
    if (sdf) {
      const dOut = sampleSdf(sdf, node.x, node.y, node.z);
      const span = (sdf.nx - 1) * sdf.cell;
      // The sampler clamps to boundary cells: outside the grid its "gradient"
      // is tangential garbage, so only trust it inside the baked box.
      const inGrid =
        node.x >= sdf.originX && node.x <= sdf.originX + span &&
        node.y >= sdf.originY && node.y <= sdf.originY + span &&
        node.z >= sdf.originZ && node.z <= sdf.originZ + span;
      if (dOut > 0) {
        const g = sdfGradient(sdf, node.x, node.y, node.z);
        const glen = Math.hypot(g.x, g.y, g.z);
        if (inGrid && glen > 1e-6) {
          node.x -= (g.x / glen) * dOut;
          node.y -= (g.y / glen) * dOut;
          node.z -= (g.z / glen) * dOut;
          const vDot = node.vx * (g.x / glen) + node.vy * (g.y / glen) + node.vz * (g.z / glen);
          if (vDot > 0) {
            node.vx -= (g.x / glen) * vDot;
            node.vy -= (g.y / glen) * vDot;
            node.vz -= (g.z / glen) * vDot;
          }
        } else {
          // No usable gradient (sampler clamps outside the grid). Bisect the
          // ray from the field center to the node for the first inside point —
          // guaranteed containment regardless of how far it escaped.
          const cx = sdf.originX + span / 2;
          const cy = sdf.originY + span / 2;
          const cz = sdf.originZ + span / 2;
          // t=0 is the field center (inside); t=1 is the node (outside).
          let tin = 0;
          let tout = 1;
          for (let it = 0; it < 16; it++) {
            const mid = (tin + tout) / 2;
            const mx = cx + (node.x - cx) * mid;
            const my = cy + (node.y - cy) * mid;
            const mz = cz + (node.z - cz) * mid;
            if (sampleSdf(sdf, mx, my, mz) <= 0) tin = mid;
            else tout = mid;
          }
          node.x = cx + (node.x - cx) * tin;
          node.y = cy + (node.y - cy) * tin;
          node.z = cz + (node.z - cz) * tin;
          node.vx *= 0.5;
          node.vy *= 0.5;
          node.vz *= 0.5;
        }
      }
    }
  }
}

function applyTagAttract(nodes: FdgNode[], params: FdgParams): void {
  const sums = new Map<string, { x: number; y: number; z: number; n: number }>();
  for (const node of nodes) {
    for (const tag of node.tags) {
      let s = sums.get(tag);
      if (!s) {
        s = { x: 0, y: 0, z: 0, n: 0 };
        sums.set(tag, s);
      }
      s.x += node.x;
      s.y += node.y;
      s.z += node.z;
      s.n += 1;
    }
  }
  for (const node of nodes) {
    let ax = 0;
    let ay = 0;
    let az = 0;
    let count = 0;
    for (const tag of node.tags) {
      const s = sums.get(tag);
      if (!s || s.n < params.minTagMembers) continue;
      ax += s.x / s.n;
      ay += s.y / s.n;
      az += s.z / s.n;
      count += 1;
    }
    if (!count) continue;
    node.vx += (ax / count - node.x) * params.tagK;
    node.vy += (ay / count - node.y) * params.tagK;
    node.vz += (az / count - node.z) * params.tagK;
  }
}
