import { test } from 'node:test';
import assert from 'node:assert/strict';
import { describeProposal, type OntologyProposal } from './proposals.ts';

const base = {
  id: 'prop:1',
  status: 'pending',
  proposed_by: 'claude',
  created_at: '2026-09-06 00:00:00',
};

const prop = (
  kind: string,
  payload: Record<string, unknown>,
): OntologyProposal => ({ ...base, kind, payload });

test('describeProposal renders each kind as a one-line summary', () => {
  assert.equal(
    describeProposal(prop('promote_atom', { atom_id: 'abc', class_id: 'class:note' })),
    'abc → class:note',
  );
  assert.equal(
    describeProposal(
      prop('new_link', { from_id: 'ent:a', to_id: 'class:decision', rel: 'is_a' }),
    ),
    'ent:a —is_a→ class:decision',
  );
  assert.equal(
    describeProposal(prop('new_entity', { entity_kind: 'class', name: 'Vector DB' })),
    'class “Vector DB”',
  );
});

test('describeProposal survives missing payload fields and unknown kinds', () => {
  assert.equal(describeProposal(prop('promote_atom', {})), '? → ?');
  const unknown = describeProposal(prop('merge_entities', { a: 1 }));
  assert.ok(unknown.length > 0 && unknown.length <= 80);
});
