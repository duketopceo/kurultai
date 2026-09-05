import assert from 'node:assert/strict';
import { test } from 'node:test';
import { normalizeLayout } from './mode.ts';
import { Octree } from './octree.ts';
import { tickFdg } from './fdg.ts';
import { bakeSdfFromPositions, makeSphereSdf, sampleSdf } from './sdf.ts';
import { DEFAULT_FDG_PARAMS, type FdgNode, type FdgParams } from './types.ts';

test('normalizeLayout maps galaxy and junk to brain', () => {
  assert.equal(normalizeLayout('ontology'), 'ontology');
  assert.equal(normalizeLayout('galaxy'), 'brain');
  assert.equal(normalizeLayout('lattice'), 'brain');
  assert.equal(normalizeLayout(null), 'brain');
});

test('octree far-cluster force matches naive COM within 20%', () => {
  const bodies: { x: number; y: number; z: number }[] = [];
  for (let i = 0; i < 8; i++) {
    bodies.push({ x: -10 + (i % 2) * 0.05, y: (i % 3) * 0.04, z: 0 });
  }
  for (let i = 0; i < 8; i++) {
    bodies.push({ x: 10 + (i % 2) * 0.05, y: (i % 3) * 0.04, z: 0 });
  }
  const tree = new Octree(bodies, 0.8);
  const bh = { x: 0, y: 0, z: 0 };
  tree.accumulate(0, bh);

  const naive = { x: 0, y: 0, z: 0 };
  const p = bodies[0];
  for (let j = 1; j < bodies.length; j++) {
    const q = bodies[j];
    const dx = p.x - q.x;
    const dy = p.y - q.y;
    const dz = p.z - q.z;
    const d2 = dx * dx + dy * dy + dz * dz + 1e-8;
    const inv = 1 / (d2 * Math.sqrt(d2));
    naive.x += dx * inv;
    naive.y += dy * inv;
    naive.z += dz * inv;
  }
  const bhMag = Math.hypot(bh.x, bh.y, bh.z);
  const naiveMag = Math.hypot(naive.x, naive.y, naive.z);
  assert.ok(naiveMag > 0, 'naive force should be nonzero');
  const err = Math.hypot(bh.x - naive.x, bh.y - naive.y, bh.z - naive.z) / naiveMag;
  assert.ok(err < 0.2, `barnes-hut error ${err} (bh=${bhMag} naive=${naiveMag})`);
});

test('tickFdg keeps nodes inside a spherical SDF', () => {
  const sdf = makeSphereSdf(1, 24);
  const nodes = seedNodes(20, 1.8);
  const params: FdgParams = {
    ...DEFAULT_FDG_PARAMS,
    tagK: 0,
    springK: 0,
    centerK: 0.04,
    hullK: 0.4,
    repulsion: 0.004,
    damping: 0.8,
  };
  for (let i = 0; i < 80; i++) tickFdg(nodes, [], sdf, params);
  for (const node of nodes) {
    const r2 = node.x * node.x + node.y * node.y + node.z * node.z;
    assert.ok(r2 <= 1.05 * 1.05, `node ${node.id} r=${Math.sqrt(r2)}`);
  }
});

test('tag attractors separate two unlinked groups vs no-tagK control', () => {
  const withTags = twoTagClouds();
  const without = twoTagClouds();
  const base: FdgParams = { ...DEFAULT_FDG_PARAMS, hullK: 0, springK: 0, centerK: 0.002, repulsion: 0.03 };
  for (let i = 0; i < 80; i++) {
    tickFdg(withTags, [], null, { ...base, tagK: 0.08 });
    tickFdg(without, [], null, { ...base, tagK: 0 });
  }
  const tagged = centroidGap(withTags);
  const control = centroidGap(without);
  assert.ok(tagged > control, `tagged gap ${tagged} should exceed control ${control}`);
});

test('mesh bake: unit cube center is inside, far point is outside', () => {
  const sdf = bakeSdfFromPositions(unitCubeTriangles(), null, 20);
  assert.ok(sdf, 'bake should succeed');
  assert.ok(sampleSdf(sdf!, 0, 0, 0) < 0, 'origin should be inside the cube');
  assert.ok(sampleSdf(sdf!, 2, 0, 0) > 0, 'far point should be outside');
});

function seedNodes(n: number, spread: number): FdgNode[] {
  const nodes: FdgNode[] = [];
  for (let i = 0; i < n; i++) {
    const h = (i * 17 + 3) % 1000;
    nodes.push({
      id: `n${i}`,
      x: ((h % 100) / 100 - 0.5) * 2 * spread,
      y: (((h * 3) % 100) / 100 - 0.5) * 2 * spread,
      z: (((h * 7) % 100) / 100 - 0.5) * 2 * spread,
      vx: 0,
      vy: 0,
      vz: 0,
      tags: [],
    });
  }
  return nodes;
}

function twoTagClouds(): FdgNode[] {
  const nodes: FdgNode[] = [];
  for (let i = 0; i < 8; i++) {
    nodes.push({
      id: `a${i}`,
      x: -0.4 + (i % 4) * 0.05,
      y: (i >> 2) * 0.05,
      z: 0,
      vx: 0,
      vy: 0,
      vz: 0,
      tags: ['a'],
    });
  }
  for (let i = 0; i < 8; i++) {
    nodes.push({
      id: `b${i}`,
      x: 0.4 + (i % 4) * 0.05,
      y: (i >> 2) * 0.05,
      z: 0,
      vx: 0,
      vy: 0,
      vz: 0,
      tags: ['b'],
    });
  }
  return nodes;
}

function centroidGap(nodes: FdgNode[]): number {
  const a = { x: 0, y: 0, z: 0, n: 0 };
  const b = { x: 0, y: 0, z: 0, n: 0 };
  for (const node of nodes) {
    const s = node.tags[0] === 'a' ? a : b;
    s.x += node.x;
    s.y += node.y;
    s.z += node.z;
    s.n += 1;
  }
  const dx = a.x / a.n - b.x / b.n;
  const dy = a.y / a.n - b.y / b.n;
  const dz = a.z / a.n - b.z / b.n;
  return Math.hypot(dx, dy, dz);
}

function unitCubeTriangles(): Float32Array {
  const v = [
    [-0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, 0.5, -0.5],
    [-0.5, 0.5, -0.5],
    [-0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],
    [-0.5, 0.5, 0.5],
  ];
  const faces = [
    [0, 1, 2, 0, 2, 3],
    [4, 6, 5, 4, 7, 6],
    [0, 4, 5, 0, 5, 1],
    [3, 2, 6, 3, 6, 7],
    [0, 3, 7, 0, 7, 4],
    [1, 5, 6, 1, 6, 2],
  ];
  const out: number[] = [];
  for (const face of faces) {
    for (const i of face) out.push(v[i][0], v[i][1], v[i][2]);
  }
  return new Float32Array(out);
}
