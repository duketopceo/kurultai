import test from 'node:test';
import assert from 'node:assert/strict';
import { LAST_VISIT_KEY, computeWhatsNew, readLastVisit, writeLastVisit } from './whatsNew.ts';

const mem = () => {
  const m = new Map<string, string>();
  return { getItem: (k: string) => m.get(k) ?? null, setItem: (k: string, v: string) => void m.set(k, v) };
};
const A = (id: string, source: string, iso: string) => ({ id, title: id, source, indexed_at: iso });

test('first visit reports nothing; marker round-trips', () => {
  const s = mem();
  assert.equal(readLastVisit(s), null);
  assert.deepEqual(computeWhatsNew([A('a', 'notes', '2026-11-01T00:00:00Z')], null), { items: [], newSources: [] });
  writeLastVisit(s, 1234);
  assert.equal(readLastVisit(s), 1234);
  assert.equal(s.getItem(LAST_VISIT_KEY), '1234');
});

test('storage that throws never breaks the page', () => {
  const bad = { getItem() { throw new Error('blocked'); }, setItem() { throw new Error('blocked'); } };
  assert.equal(readLastVisit(bad), null);
  assert.doesNotThrow(() => writeLastVisit(bad));
  assert.equal(readLastVisit(null), null);
});

test('atoms indexed after the marker are new; fully-new sources are flagged', () => {
  const since = Date.parse('2026-11-05T00:00:00Z');
  const r = computeWhatsNew([
    A('old', 'notes', '2026-11-01T00:00:00Z'),
    A('n1', 'notes', '2026-11-06T00:00:00Z'),
    A('g1', 'github', '2026-11-06T00:00:00Z'),
    A('bad', 'notes', 'not-a-date'),
  ], since);
  assert.deepEqual(r.items.map((i) => i.id), ['n1', 'g1']);
  assert.deepEqual(r.newSources, ['github']);
});
