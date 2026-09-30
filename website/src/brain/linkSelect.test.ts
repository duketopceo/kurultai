import { test } from 'node:test';
import assert from 'node:assert/strict';
import { selectIntentionalLinks } from './linkSelect.ts';

const clique = (ids: string[], strength = 1) => {
  const links = [];
  for (let i = 0; i < ids.length; i++)
    for (let j = i + 1; j < ids.length; j++)
      links.push({ a: ids[i], b: ids[j], strength });
  return links;
};

test('a clique collapses to a backbone, not a full mesh', () => {
  const ids = Array.from({ length: 25 }, (_, i) => `c${i}`);
  const links = clique(ids);
  const kept = selectIntentionalLinks(links, { nominationsPerNode: 3, maxLinks: 1200 });
  // Union of nominations: ≤ 25×3 undirected edges — not the 300-edge mesh.
  // Union (not mutual-only) guarantees every node keeps >=1 link.
  assert.ok(kept.length <= 75, `kept ${kept.length}`);
  assert.ok(kept.length > 0);
});

test('low-degree bridge edges always survive', () => {
  // leaf --hub-- plus hub embedded in a clique of stronger links.
  const cliqueIds = ['hub', 'x1', 'x2', 'x3', 'x4', 'x5'];
  const links = [
    ...clique(cliqueIds, 5),
    { a: 'leaf', b: 'hub', strength: 1 },
  ];
  const kept = selectIntentionalLinks(links, { nominationsPerNode: 2, maxLinks: 1200 });
  assert.ok(kept.some((l) => l.a === 'leaf' && l.b === 'hub'), 'leaf edge kept');
});

test('maxLinks budget respected and strongest win', () => {
  const ids = Array.from({ length: 60 }, (_, i) => `n${i}`);
  const links = clique(ids, 1);
  // add a few strong links
  links.push({ a: 'n0', b: 'n1', strength: 9 });
  const kept = selectIntentionalLinks(links, { nominationsPerNode: 4, maxLinks: 50 });
  assert.equal(kept.length, 50);
  assert.equal(kept[0].strength, 9);
});

test('bridges survive the cap even when weaker than nominated links', () => {
  // 50-node strength-5 clique + one weak bridge to a leaf. With a cap below
  // the nominated count, the bridge must still be kept — it is load-bearing.
  const ids = Array.from({ length: 50 }, (_, i) => `n${i}`);
  const links = [...clique(ids, 5), { a: 'leaf', b: 'n0', strength: 1 }];
  const kept = selectIntentionalLinks(links, { nominationsPerNode: 2, maxLinks: 40 });
  assert.ok(
    kept.some((l) => l.a === 'leaf' && l.b === 'n0'),
    'weak leaf bridge survived cap',
  );
});

test('deterministic output for identical input', () => {
  const ids = Array.from({ length: 12 }, (_, i) => `d${i}`);
  const links = clique(ids);
  const a = selectIntentionalLinks(links);
  const b = selectIntentionalLinks(links.slice().reverse());
  assert.deepEqual(
    a.map((l) => [l.a, l.b, l.strength]),
    b.map((l) => [l.a, l.b, l.strength]),
  );
});

test('chain topology keeps every link (all nodes low-degree)', () => {
  const links = [
    { a: 'a', b: 'b', strength: 1 },
    { a: 'b', b: 'c', strength: 1 },
    { a: 'c', b: 'd', strength: 1 },
  ];
  const kept = selectIntentionalLinks(links, { lowDegreeKeepAll: 2 });
  assert.equal(kept.length, 3);
});
