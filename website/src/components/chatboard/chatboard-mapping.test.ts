import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  buildReactionIndex,
  identityLabel,
  mapMessages,
  mapThreads,
  presenceMap,
} from './chatboard-mapping.ts';

const NOW = 1_700_000_000_000;
const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const iso = (ms: number) => new Date(ms).toISOString();

test('identityLabel joins codename and instance, with ordered fallbacks', () => {
  assert.equal(identityLabel({ codename: 'buzz', instance_id: 'node3' }), 'buzz@node3');
  assert.equal(identityLabel({ codename: 'buzz' }), 'buzz');
  assert.equal(identityLabel({ name: 'Atlas' }), 'Atlas');
  assert.equal(identityLabel({ instance_id: 'x1' }), 'x1');
  assert.equal(identityLabel({}), 'agent');
  assert.equal(identityLabel({ kind: 'human' }), 'Human');
  assert.equal(identityLabel({ kind: 'system' }), 'System');
});

test('mapThreads uses real HeyThread fields with unread fed from the unread map', () => {
  const raw = [
    { id: 't-uuid-1', name: 'hey.md', updated_at: iso(NOW - 5 * MINUTE) },
    { id: 't-uuid-2', name: 'ops', created_at: iso(NOW - 2 * DAY) },
  ];
  const unread = new Map([['t-uuid-1', 4]]);
  const threads = mapThreads(raw, { nowMs: NOW, unreadByThread: unread });
  assert.equal(threads.length, 2);
  assert.equal(threads[0].id, 't-uuid-1');
  assert.equal(threads[0].title, 'hey.md');
  assert.equal(threads[0].unread, 4);
  assert.equal(threads[1].unread, 0);
  assert.equal(threads[0].updatedLabel, '5m');
  assert.ok(threads[0].updatedAtMs > threads[1].updatedAtMs, 'sorted newest first');
});

test('mapThreads never reads phantom participants/unread_count fields', () => {
  const raw = [{ id: 't1', name: 'n', participants: [{ codename: 'x' }], unread_count: 9 }];
  const threads = mapThreads(raw, { nowMs: NOW });
  assert.equal(threads[0].unread, 0);
});

test('relative labels bucket into now/m/h/d and ISO date beyond a week', () => {
  const raw = [
    { id: 'a', updated_at: NOW - 30_000 },
    { id: 'b', updated_at: iso(NOW - 3 * HOUR) },
    { id: 'c', updated_at: iso(NOW - 9 * DAY) },
  ];
  const labels = mapThreads(raw, { nowMs: NOW }).map((t) => t.updatedLabel);
  assert.deepEqual(labels, ['now', '3h', new Date(NOW - 9 * DAY).toISOString().split('T')[0]]);
});

test('mapMessages maps real HeyMessage fields; senders use codename@instance', () => {
  const raw = [
    {
      id: 'm1',
      thread_id: 't1',
      agent_id: 'a1',
      agent_codename: 'c1',
      instance_id: 'i2',
      kind: 'message',
      content: 'hi',
      created_at: iso(NOW - 5 * MINUTE),
    },
    { id: 'm2', thread_id: 't1', agent_id: 'a2', kind: 'message', content: 'hello', created_at: iso(NOW) },
  ];
  const msgs = mapMessages(raw, { nowMs: NOW });
  assert.equal(msgs.length, 2);
  assert.equal(msgs[0].senderLabel, 'c1@i2');
  assert.equal(msgs[0].agentKey, 'a1');
  assert.equal(msgs[0].body, 'hi');
  assert.equal(msgs[0].timeLabel, '5m');
  assert.equal(msgs[0].createdAtMs, NOW - 5 * MINUTE);
  assert.equal(msgs[1].senderLabel, 'a2');
  assert.equal(msgs[1].reactions.length, 0);
});

test('buildReactionIndex aggregates reaction rows onto their parent message', () => {
  const raw = [
    { id: 'm1', kind: 'message', content: 'hi' },
    { id: 'r1', kind: 'reaction', parent_id: 'm1', agent_id: 'a1', content: '👍' },
    { id: 'r2', kind: 'reaction', parent_id: 'm1', agent_id: 'a2', content: '👍' },
    { id: 'r3', kind: 'reaction', parent_id: 'm1', agent_id: 'a3', content: '🔥' },
    { id: 'r4', kind: 'reaction', parent_id: '', agent_id: 'a4', content: '👀' },
  ];
  const index = buildReactionIndex(raw);
  assert.deepEqual(index.get('m1'), [
    { emoji: '👍', count: 2 },
    { emoji: '🔥', count: 1 },
  ]);
  assert.equal(index.size, 1, 'orphan reactions without a parent are skipped');
});

test('mapMessages threads reactions from the index, not from message rows', () => {
  const raw = [
    { id: 'm1', kind: 'message', agent_id: 'a1', content: 'hi', created_at: iso(NOW) },
    { id: 'r1', kind: 'reaction', parent_id: 'm1', agent_id: 'a2', content: '👍' },
  ];
  const msgs = mapMessages(raw, { nowMs: NOW, reactions: buildReactionIndex(raw) });
  assert.deepEqual(msgs[0].reactions, [{ emoji: '👍', count: 1 }]);
});

test('presenceMap buckets by recency into online/away/offline keyed by agent id', () => {
  const raw = [
    { agent_id: 'a1', created_at: iso(NOW - 2 * MINUTE) },
    { agent_id: 'a2', created_at: iso(NOW - 40 * MINUTE) },
    { agent_id: 'a3', created_at: iso(NOW - 5 * HOUR) },
    { agent_id: 'a4' },
  ];
  const map = presenceMap(raw, NOW);
  assert.equal(map.a1, 'online');
  assert.equal(map.a2, 'away');
  assert.equal(map.a3, 'offline');
  assert.equal(map.a4, 'offline');
});

test('mapping is defensive: malformed and empty inputs never throw or mutate', () => {
  assert.deepEqual(mapThreads(null as unknown as unknown[]), []);
  assert.deepEqual(mapMessages(undefined as unknown as unknown[]), []);
  assert.deepEqual(mapThreads([]), []);
  assert.deepEqual(mapMessages([]), []);
  assert.deepEqual(buildReactionIndex([null, 'x', { kind: 'reaction' }]), new Map());
  assert.deepEqual(presenceMap('nope' as unknown), {});
  const raw = [{ id: 't1', updated_at: 'garbage' }];
  const before = JSON.stringify(raw);
  const threads = mapThreads(raw, { nowMs: NOW });
  assert.equal(JSON.stringify(raw), before);
  assert.equal(threads[0].updatedAtMs, 0);
});
