import { test } from 'node:test';
import assert from 'node:assert/strict';
import { identityLabel, mapMessages, mapThreads } from './chatboard-mapping.ts';

const NOW = 1_700_000_000_000;
const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

test('identityLabel joins codename and instance, with ordered fallbacks', () => {
  assert.equal(identityLabel({ codename: 'buzz', instance_id: 'node3' }), 'buzz@node3');
  assert.equal(identityLabel({ codename: 'buzz' }), 'buzz');
  assert.equal(identityLabel({ name: 'Atlas' }), 'Atlas');
  assert.equal(identityLabel({ instance_id: 'x1' }), 'x1');
  assert.equal(identityLabel({}), 'agent');
  assert.equal(identityLabel({ kind: 'human' }), 'Human');
  assert.equal(identityLabel({ kind: 'system' }), 'System');
});

test('mapThreads maps fields, peer labels, unread counts, and sorts by recency', () => {
  const raw = [
    {
      id: 't1',
      name: 'Alpha thread',
      participants: [{ codename: 'buzz', instance_id: 'n3' }, { kind: 'human' }],
      unread_count: 4,
      updated_at: new Date(NOW - 5 * MINUTE).toISOString(),
    },
    {
      id: 't2',
      title: 'Older',
      created_at: new Date(NOW - 2 * DAY).toISOString(),
    },
  ];
  const threads = mapThreads(raw, NOW);
  assert.equal(threads.length, 2);
  assert.equal(threads[0].id, 't1');
  assert.equal(threads[0].peerLabel, 'buzz@n3');
  assert.equal(threads[0].unread, 4);
  assert.equal(threads[0].updatedLabel, '5m');
  assert.equal(threads[1].peerLabel, 'Older');
  assert.equal(threads[1].unread, 0);
  assert.equal(threads[1].updatedLabel, '2d');
  assert.ok(threads[0].updatedAtMs > threads[1].updatedAtMs, 'sorted newest first');
});

test('relative labels bucket into now/m/h/d and ISO date beyond a week', () => {
  const raw = [
    { id: 'a', updated_at: NOW - 30_000 },
    { id: 'b', updated_at: NOW - 3 * HOUR },
    { id: 'c', updated_at: NOW - 9 * DAY },
  ];
  const labels = mapThreads(raw, NOW).map((t) => t.updatedLabel);
  assert.deepEqual(labels, ['now', '3h', new Date(NOW - 9 * DAY).toISOString().split('T')[0]]);
});

test('mapMessages maps agents, humans, and collapses reactions', () => {
  const raw = [
    {
      id: 'm1',
      kind: 'agent',
      codename: 'c1',
      instance_id: 'i2',
      content: 'hi',
      created_at: NOW - 5 * MINUTE,
      reactions: [{ emoji: '👍', count: 2 }, { emoji: '👍', count: 1 }, { emoji: '🔥' }],
    },
    { id: 'm2', kind: 'human', content: 'hello', created_at: NOW },
    {},
  ];
  const msgs = mapMessages(raw, NOW);
  assert.equal(msgs.length, 3);
  assert.equal(msgs[0].senderLabel, 'c1@i2');
  assert.equal(msgs[0].senderKind, 'agent');
  assert.equal(msgs[0].body, 'hi');
  assert.equal(msgs[0].timeLabel, '5m');
  assert.deepEqual(msgs[0].reactions, [
    { emoji: '👍', count: 3 },
    { emoji: '🔥', count: 1 },
  ]);
  assert.equal(msgs[1].senderLabel, 'Human');
  assert.equal(msgs[1].senderKind, 'human');
  assert.equal(msgs[2].id, 'message-2');
  assert.equal(msgs[2].senderKind, 'agent');
  assert.equal(msgs[2].body, '');
});

test('mapping is defensive: malformed and empty inputs never throw or mutate', () => {
  assert.deepEqual(mapThreads(null as unknown as unknown[], NOW), []);
  assert.deepEqual(mapMessages(undefined as unknown as unknown[], NOW), []);
  assert.deepEqual(mapThreads([], NOW), []);
  assert.deepEqual(mapMessages([], NOW), []);
  const raw = [{ id: 't1', participants: 'not-an-array', updated_at: 'garbage' }];
  const before = JSON.stringify(raw);
  const threads = mapThreads(raw, NOW);
  assert.equal(JSON.stringify(raw), before);
  assert.equal(threads[0].id, 't1');
  assert.equal(threads[0].updatedAtMs, 0);
});
