import { test } from 'node:test';
import assert from 'node:assert/strict';
import { deriveGroups, GROUP_TINTS, type GroupInputEntity, type GroupInputLink } from './grouping.ts';

function classEntity(id: string, name?: string): GroupInputEntity {
  return { id, kind: 'class', name };
}

function instanceEntity(id: string): GroupInputEntity {
  return { id, kind: 'instance' };
}

test('instance_of children attach to their class regardless of link order', () => {
  const entities = [classEntity('c1', 'Note'), instanceEntity('i1'), instanceEntity('i2')];
  const forward = deriveGroups(entities, [{ a: 'i1', b: 'c1', rel: 'instance_of' }]);
  const reversed = deriveGroups(entities, [{ a: 'c1', b: 'i2', rel: 'instance_of' }]);
  assert.deepEqual(forward[0].childIds, ['i1']);
  assert.deepEqual(reversed[0].childIds, ['i2']);
});

test('is_a subclasses nest under their root ancestor group only', () => {
  const entities = [classEntity('a', 'Root'), classEntity('b', 'Mid'), classEntity('c', 'Leaf')];
  const links: GroupInputLink[] = [
    { a: 'b', b: 'a', rel: 'is_a' },
    { a: 'c', b: 'b', rel: 'is_a' },
  ];
  const groups = deriveGroups(entities, links);
  const byId = new Map(groups.map((g): [string, typeof g] => [g.classId, g]));
  assert.deepEqual(byId.get('a')?.nestedIds, ['b', 'c']);
  assert.deepEqual(byId.get('b')?.nestedIds, []);
  assert.deepEqual(byId.get('c')?.nestedIds, []);
});

test('tintIndex is the stable sorted-class index modulo the tint count', () => {
  const entities = [
    classEntity('z', 'Zulu'),
    classEntity('a', 'Alpha'),
    classEntity('m', 'Mike'),
    classEntity('b2', 'Bravo'),
    classEntity('q', 'Quebec'),
    classEntity('d', 'Delta'),
    classEntity('e', 'Echo'),
  ];
  const groups = deriveGroups(entities, []);
  const order = ['Alpha', 'Bravo', 'Delta', 'Echo', 'Mike', 'Quebec', 'Zulu'];
  for (const g of groups) {
    assert.equal(g.tintIndex, order.indexOf(g.name) % GROUP_TINTS.length, g.name);
  }
});

test('links referencing unknown entities are ignored', () => {
  const groups = deriveGroups(
    [classEntity('c1', 'Note')],
    [
      { a: 'ghost', b: 'c1', rel: 'instance_of' },
      { a: 'c1', b: 'phantom', rel: 'is_a' },
    ],
  );
  assert.equal(groups.length, 1);
  assert.deepEqual(groups[0].childIds, []);
  assert.deepEqual(groups[0].nestedIds, []);
});

test('empty inputs yield no groups; inputs are never mutated', () => {
  assert.deepEqual(deriveGroups([], []), []);
  const entities = [classEntity('c1', 'Note'), instanceEntity('i1')];
  const links: GroupInputLink[] = [{ a: 'i1', b: 'c1', rel: 'instance_of' }];
  const before = JSON.stringify({ entities, links });
  deriveGroups(entities, links);
  assert.equal(JSON.stringify({ entities, links }), before);
});

test('GROUP_TINTS stays inside the purple family (six shades)', () => {
  assert.equal(GROUP_TINTS.length, 6);
  for (const tint of GROUP_TINTS) {
    const r = (tint >> 16) & 0xff;
    const g = (tint >> 8) & 0xff;
    const b = tint & 0xff;
    assert.ok(b > r && r > g, `tint ${tint.toString(16)} is purple-family (b > r > g)`);
    assert.ok(b >= 0xfd * 0.6, `tint ${tint.toString(16)} is bright enough on black`);
  }
});
