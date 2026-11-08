import test from 'node:test';
import assert from 'node:assert/strict';
import { layoutFloorplan, entriesFromAtoms, entriesFromAgentIndex } from './floorplan.ts';

const E = (path: string, size: number, activity = 0.5) => ({ path, size, activity });

test('tiles stay inside bounds and do not overlap', () => {
  const entries = [E('src/a.rs', 400), E('src/b.rs', 100), E('src/http/c.rs', 250), E('docs/x.md', 60), E('README.md', 30)];
  const plan = layoutFloorplan(entries, 800, 500);
  assert.equal(plan.tiles.length, entries.length);
  for (const t of plan.tiles) {
    assert.ok(t.x >= -0.01 && t.y >= -0.01 && t.x + t.w <= 800.01 && t.y + t.h <= 500.01, t.path);
  }
  for (let i = 0; i < plan.tiles.length; i++) {
    for (let j = i + 1; j < plan.tiles.length; j++) {
      const a = plan.tiles[i], b = plan.tiles[j];
      const overlap = a.x < b.x + b.w - 0.01 && b.x < a.x + a.w - 0.01 && a.y < b.y + b.h - 0.01 && b.y < a.y + a.h - 0.01;
      assert.ok(!overlap, `${a.path} overlaps ${b.path}`);
    }
  }
});

test('bigger files get bigger tiles; folders become rooms', () => {
  const plan = layoutFloorplan([E('d/big.rs', 900), E('d/small.rs', 100)], 600, 400);
  const big = plan.tiles.find((t) => t.name === 'big.rs')!;
  const small = plan.tiles.find((t) => t.name === 'small.rs')!;
  assert.ok(big.w * big.h > small.w * small.h * 3);
  assert.deepEqual(plan.rooms.map((r) => r.path), ['d']);
});

test('entriesFromAtoms strips the repo prefix, merges duplicates, scores recency', () => {
  const now = Date.parse('2026-11-07T00:00:00Z');
  const atoms = [
    { source_id: '/home/u/kurultai/src/a.rs#fn1', summary: 'x'.repeat(100), tags: ['rs'], indexed_at: '2026-11-06T00:00:00Z', last_accessed_at: '' },
    { source_id: '/home/u/kurultai/src/a.rs#fn2', summary: '', tags: [], indexed_at: '2025-01-01T00:00:00Z', last_accessed_at: '' },
  ];
  const e = entriesFromAtoms(atoms, 'duketopceo/kurultai', now);
  assert.equal(e.length, 1);
  assert.equal(e[0].path, 'src/a.rs');
  assert.ok(e[0].activity > 0.9);
  assert.ok(e[0].size > 160);
});

test('entriesFromAgentIndex parses INDEX.md file rows', () => {
  const md = '| File | Does |\n|---|---|\n| [`a.rs`](a.rs) | does a | x |\n| [`sub/b.ts`](sub/b.ts) | b | x |\n';
  const e = entriesFromAgentIndex(md, 'src');
  assert.deepEqual(e.map((x) => x.path), ['src/a.rs', 'src/sub/b.ts']);
});

test('empty input yields an empty plan', () => {
  assert.deepEqual(layoutFloorplan([], 100, 100), { rooms: [], tiles: [] });
});
