import assert from 'node:assert/strict';
import { test } from 'node:test';
import { boardEdges, entityToAtom, layoutOntology } from './ontoBoard.ts';
import type { Atom, OntologyEntity, OntologyLink } from '../../types.ts';

const cls = (id: string, name = id): OntologyEntity => ({ id, kind: 'class', name });
const inst = (id: string, name = id, atom_id?: string): OntologyEntity => ({
  id, kind: 'instance', name, atom_id: atom_id ?? null,
});
const link = (from: string, to: string, rel: string): OntologyLink => ({
  id: `link:${from}:${rel}:${to}`, from_id: from, to_id: to, rel,
});
const atom = (id: string): Atom => ({
  id, title: id, summary: '', source: 'markdown', source_id: '', tags: [],
  file: '', indexed_at: '', last_accessed_at: '', score: 0, tier: 'warm', lean: true,
});

test('classes layer by is_a depth; instances hidden until expanded', () => {
  const entities = [cls('class:memory'), cls('class:note'), inst('ent:a', 'A', 'atom-1')];
  const links = [link('class:note', 'class:memory', 'is_a'), link('ent:a', 'class:note', 'instance_of')];
  const nodes = layoutOntology(entities, links, new Set(), {}, [atom('atom-1')]);
  const mem = nodes.find((n) => n.id === 'class:memory')!;
  const note = nodes.find((n) => n.id === 'class:note')!;
  assert.equal(mem.position.y, 0);
  assert.equal(note.position.y, 180);
  assert.equal(note.data.hiddenCount, 1);
  assert.ok(!nodes.some((n) => n.id === 'ent:a'), 'instance hidden before expand');

  const expanded = layoutOntology(entities, links, new Set(['class:note']), {}, [atom('atom-1')]);
  const a = expanded.find((n) => n.id === 'ent:a')!;
  assert.equal(a.data.backed, true);
  assert.equal(expanded.find((n) => n.id === 'class:note')!.data.hiddenCount, 0);
});

test('saved positions win; orphan instances go to the bottom row', () => {
  const entities = [cls('class:memory'), inst('ent:loose')];
  const saved = { 'class:memory': { x: 42, y: 7 } };
  const nodes = layoutOntology(entities, [], new Set(), saved, []);
  assert.deepEqual(nodes.find((n) => n.id === 'class:memory')!.position, { x: 42, y: 7 });
  const loose = nodes.find((n) => n.id === 'ent:loose')!;
  assert.ok(loose.position.y > 0);
});

test('boardEdges keeps only edges between visible nodes', () => {
  const links = [
    link('class:note', 'class:memory', 'is_a'),
    link('ent:a', 'class:note', 'instance_of'),
  ];
  const edges = boardEdges(links, new Set(['class:note', 'class:memory']));
  assert.equal(edges.length, 1);
  assert.equal(edges[0].rel, 'is_a');
});

test('entityToAtom returns backing atom or a stub', () => {
  const atoms = [atom('atom-1')];
  const backed = entityToAtom(inst('ent:a', 'A', 'atom-1'), atoms);
  assert.equal(backed.id, 'atom-1');
  const stub = entityToAtom(cls('class:x', 'X'), []);
  assert.equal(stub.source, 'ontology');
  assert.equal(stub.title, 'X');
});
