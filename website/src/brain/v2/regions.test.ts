import test from 'node:test';
import assert from 'node:assert/strict';
import { insideHull, lodTier, parcellate, placeAll, hashStream, MAX_REGIONS } from './regions.ts';
import { buildAdjacency, shortestPath } from './path.ts';
import { buildTagLinks } from './links.ts';

const mk = (n: number, tags: (i: number) => string[]) =>
  Array.from({ length: n }, (_, i) => ({ id: `a${i}`, tags: tags(i) }));

test('every placed atom is inside the hull (no spaghetti)', () => {
  const atoms = mk(2000, (i) => [`t${i % 7}`]);
  const p = parcellate(atoms);
  const xyz = placeAll(atoms, p);
  for (let i = 0; i < atoms.length; i++) assert.ok(insideHull(xyz[i * 3], xyz[i * 3 + 1], xyz[i * 3 + 2]), `atom ${i}`);
});

test('placement is deterministic and region counts add up', () => {
  const atoms = mk(500, (i) => (i % 5 === 0 ? [] : [`t${i % 3}`, 'common']));
  const a = placeAll(atoms, parcellate(atoms));
  const b = placeAll(atoms, parcellate(atoms));
  assert.deepEqual([...a], [...b]);
  const p = parcellate(atoms);
  assert.equal(p.regions.reduce((s, r) => s + r.count, 0), atoms.length);
});

test('regions cap at MAX_REGIONS with an other bucket; fill grows with count', () => {
  const many = mk(3000, (i) => [`tag${i % 60}`]);
  const p = parcellate(many);
  assert.equal(p.regions.length, MAX_REGIONS);
  assert.ok(p.regions.some((r) => r.key === 'other'));
  const small = parcellate(mk(3, () => ['x']));
  const big = parcellate(mk(500, () => ['x']));
  assert.ok(big.regions[0].fill > small.regions[0].fill);
  assert.equal(big.regions[0].fill, 1);
});

test('untagged atoms land in an untagged region', () => {
  const p = parcellate(mk(5, () => []));
  assert.equal(p.regions[0].key, 'untagged');
});

test('lod tiers by distance', () => {
  assert.equal(lodTier(4), 'far');
  assert.equal(lodTier(2), 'mid');
  assert.equal(lodTier(0.8), 'near');
});

test('hashStream stays in [0,1)', () => {
  const r = hashStream('seed');
  for (let i = 0; i < 1000; i++) {
    const v = r();
    assert.ok(v >= 0 && v < 1);
  }
});

test('shortestPath finds the minimal chain, null when disconnected', () => {
  const adj = buildAdjacency([
    { a: 'a', b: 'b' }, { a: 'b', b: 'c' }, { a: 'c', b: 'd' }, { a: 'a', b: 'd' }, { a: 'x', b: 'y' },
  ]);
  assert.deepEqual(shortestPath(adj, 'a', 'd'), ['a', 'd']);
  assert.deepEqual(shortestPath(adj, 'a', 'c'), ['a', 'b', 'c']);
  assert.equal(shortestPath(adj, 'a', 'y'), null);
  assert.deepEqual(shortestPath(adj, 'a', 'a'), ['a']);
});

test('buildTagLinks connects atoms sharing a tag', () => {
  const links = buildTagLinks([
    { id: '1', tags: ['k'] }, { id: '2', tags: ['k'] }, { id: '3', tags: ['z'] },
  ]);
  assert.ok(links.some((l) => (l.a === '1' && l.b === '2') || (l.a === '2' && l.b === '1')));
});

test('cortex lobes: every lobe owns interior volume, atoms stay in their lobe, shell mesh is low-poly', async () => {
  const c = await import('./cortex.ts');
  assert.equal(c.LOBES, MAX_REGIONS);
  assert.ok(c.MESH_IDX.length / 3 >= 1500 && c.MESH_IDX.length / 3 <= 6000, 'low-poly mesh stays within 1.5-6k tris');
  for (let l = 0; l < c.LOBES; l++) assert.ok(c.LOBE_VOXELS[l].length >= 3 * 40, `lobe ${l} has volume`);
  const atoms = mk(400, (i) => [`t${i % 9}`]);
  const p = parcellate(atoms);
  const xyz = placeAll(atoms, p);
  let own = 0; // sub-voxel jitter may straddle a lobe border; contiguity means nearly all stay home
  for (let i = 0; i < atoms.length; i++) {
    if (c.nearestLobe(xyz[i * 3], xyz[i * 3 + 1], xyz[i * 3 + 2]) === p.regions[p.regionOf[i]].lobe) own++;
  }
  assert.ok(own / atoms.length > 0.9, `${own}/${atoms.length} atoms in their own lobe`);
});

test('region fill rises monotonically with atom count (few -> many story)', () => {
  const fills = [3, 30, 150, 600].map((n) => parcellate(mk(n, () => ['x'])).regions[0].fill);
  for (let i = 1; i < fills.length; i++) assert.ok(fills[i] > fills[i - 1] || fills[i] === 1);
  assert.equal(fills[3], 1);
});
