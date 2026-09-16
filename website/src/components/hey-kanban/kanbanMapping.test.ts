import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  laneOf,
  laneBody,
  withLane,
  messagesToColumns,
  DEFAULT_LANE,
} from './kanbanMapping.ts';

test('laneOf parses a leading token, defaults to inbox', () => {
  assert.equal(laneOf('[todo] ship the thing'), 'todo');
  assert.equal(laneOf('[DOING] work'), 'doing');
  assert.equal(laneOf('no lane here'), DEFAULT_LANE);
  assert.equal(laneOf('  [review] check'), 'review');
  assert.equal(laneOf('[x-custom] body'), 'x-custom');
  assert.equal(laneOf('[has space] body'), DEFAULT_LANE);
});

test('laneBody strips only the leading token', () => {
  assert.equal(laneBody('[todo] ship it'), 'ship it');
  assert.equal(laneBody('plain body'), 'plain body');
  assert.equal(laneBody('[todo] keep [other] inner'), 'keep [other] inner');
});

test('withLane rewrites or removes the token', () => {
  assert.equal(withLane('[todo] ship it', 'doing'), '[doing] ship it');
  assert.equal(withLane('ship it', 'done'), '[done] ship it');
  assert.equal(withLane('[doing] ship it', 'inbox'), 'ship it');
});

test('messagesToColumns buckets, drops reactions, orders oldest-first', () => {
  const cols = messagesToColumns([
    { id: '1', kind: 'message', content: '[todo] a', created_at: '2026-09-16T02:00:00Z' },
    { id: '2', kind: 'message', content: 'b', created_at: '2026-09-16T01:00:00Z' },
    { id: '3', kind: 'reaction', content: ':ok:', created_at: '2026-09-16T03:00:00Z' },
    { id: '4', kind: 'message', content: '[custom-lane] c', created_at: '2026-09-16T04:00:00Z' },
  ]);
  const ids = Object.fromEntries(cols.map((c) => [c.id, c.messages.map((m) => m.id)]));
  assert.deepEqual(ids.todo, ['1']);
  assert.deepEqual(ids.inbox, ['2']);
  assert.deepEqual(ids['custom-lane'], ['4']);
  assert.deepEqual(ids.done, []);
  assert.ok(cols.every((c) => c.messages.every((m) => m.kind !== 'reaction')));
});
