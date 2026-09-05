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
      if (d > 0) {
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

    // Hard project after integrate: soft hullK is only a bias and loses to
    // repulsion/springs. Exterior nodes (d > 0) snap to/into the surface
    // along −∇. A few iterations cover coarse SDF residual.
    if (sdf) {
      for (let proj = 0; proj < 3; proj++) {
        const d = sampleSdf(sdf, node.x, node.y, node.z);
        if (d <= 0) break;
        const g = sdfGradient(sdf, node.x, node.y, node.z);
        const glen = Math.hypot(g.x, g.y, g.z);
        if (glen > 1e-6) {
          const nx = g.x / glen;
          const ny = g.y / glen;
          const nz = g.z / glen;
          const step = d + sdf.cell * 0.05;
          node.x -= nx * step;
          node.y -= ny * step;
          node.z -= nz * step;
          if (proj === 0) {
            const vn = node.vx * nx + node.vy * ny + node.vz * nz;
            if (vn > 0) {
              node.vx -= nx * vn;
              node.vy -= ny * vn;
              node.vz -= nz * vn;
            }
          }
        } else {
          const r = Math.hypot(node.x, node.y, node.z) || EPS;
          const scale = Math.max(0, (r - d) / r);
          node.x *= scale;
          node.y *= scale;
          node.z *= scale;
          break;
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
