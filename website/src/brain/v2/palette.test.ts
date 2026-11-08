import test from 'node:test';
import assert from 'node:assert/strict';
import { PALETTE_COMMANDS, filterCommands } from './palette.ts';
import { describeChain } from './why.ts';

test('palette: empty query lists all, prefix beats substring', () => {
  assert.equal(filterCommands(PALETTE_COMMANDS, '').length, PALETTE_COMMANDS.length);
  const r = filterCommands(PALETTE_COMMANDS, 'o');
  assert.equal(r[0].id, 'ontology');
  assert.ok(r.some((c) => c.id === 'store'));
});

test('palette: hint match and no match', () => {
  assert.equal(filterCommands(PALETTE_COMMANDS, 'floorplan')[0].id, 'repos');
  assert.deepEqual(filterCommands(PALETTE_COMMANDS, 'zzzz'), []);
});

test('why-connected chain names the shared tags per hop', () => {
  const m = new Map([
    ['a', { id: 'a', title: 'A', tags: ['x', 'y'] }],
    ['b', { id: 'b', title: 'B', tags: ['y', 'z'] }],
    ['c', { id: 'c', title: 'C', tags: ['z'] }],
  ]);
  const c = describeChain(['a', 'b', 'c', 'gone'], m);
  assert.deepEqual(c.map((s) => s.via), [[], ['y'], ['z'], []]);
  assert.equal(c[3].title, 'gone');
});
