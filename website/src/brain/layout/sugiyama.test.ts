import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import {
  assignLayers,
  bucketsFromAssign,
  hierPositions,
  orderLayers,
} from './sugiyama.ts';

describe('assignLayers', () => {
  it('places chain A→B→C with C superclass at layer 0', () => {
    const nodes = [{ id: 'A' }, { id: 'B' }, { id: 'C' }];
    const edges = [
      { from: 'A', to: 'B' },
      { from: 'B', to: 'C' },
    ];
    const layers = assignLayers(nodes, edges);
    assert.equal(layers.get('C'), 0);
    assert.equal(layers.get('B'), 1);
    assert.equal(layers.get('A'), 2);
  });

  it('puts two subclasses of Memory on the same layer', () => {
    const nodes = [{ id: 'memory' }, { id: 'note' }, { id: 'code' }];
    const edges = [
      { from: 'note', to: 'memory' },
      { from: 'code', to: 'memory' },
    ];
    const layers = assignLayers(nodes, edges);
    assert.equal(layers.get('memory'), 0);
    assert.equal(layers.get('note'), 1);
    assert.equal(layers.get('code'), 1);
  });

  it('terminates on a cycle and assigns finite layers', () => {
    const nodes = [{ id: 'A' }, { id: 'B' }];
    const edges = [
      { from: 'A', to: 'B' },
      { from: 'B', to: 'A' },
    ];
    const layers = assignLayers(nodes, edges);
    assert.equal(layers.size, 2);
    for (const id of ['A', 'B']) {
      const layer = layers.get(id);
      assert.equal(typeof layer, 'number');
      assert.ok(Number.isFinite(layer));
    }
  });
});

describe('hierPositions', () => {
  it('gives two same-layer subclasses distinct x', () => {
    const assign = assignLayers(
      [{ id: 'memory' }, { id: 'note' }, { id: 'code' }],
      [
        { from: 'note', to: 'memory' },
        { from: 'code', to: 'memory' },
      ],
    );
    const ordered = orderLayers(bucketsFromAssign(assign), [
      { from: 'note', to: 'memory' },
      { from: 'code', to: 'memory' },
    ]);
    const pos = hierPositions(ordered, 1, 1);
    assert.equal(pos.get('memory')?.y, 0);
    assert.equal(pos.get('note')?.y, pos.get('code')?.y);
    assert.notEqual(pos.get('note')?.x, pos.get('code')?.x);
  });
});
