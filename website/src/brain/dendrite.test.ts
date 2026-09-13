import { test } from 'node:test';
import assert from 'node:assert/strict';
import { coronaParams, coronaTexture, somaGeometry } from './dendrite.ts';

test('coronaParams grows monotonically with degree', () => {
  const low = coronaParams(1);
  const mid = coronaParams(10);
  const high = coronaParams(20);
  assert.ok(low.filaments < mid.filaments);
  assert.ok(mid.filaments <= high.filaments);
  assert.ok(low.scale <= mid.scale && mid.scale <= high.scale);
});

test('coronaParams clamps at the floor and saturates at the cap', () => {
  const zero = coronaParams(0);
  const huge = coronaParams(500);
  assert.equal(zero.filaments, 5);
  assert.equal(huge.filaments, 9);
  assert.ok(huge.scale <= 3.0);
});

test('coronaParams never goes below the minimum on negative degree', () => {
  const neg = coronaParams(-7);
  assert.equal(neg.filaments, 5);
});

test('coronaTexture returns null without a DOM', () => {
  assert.equal(coronaTexture(coronaParams(3)), null);
});

test('somaGeometry is a faceted polyhedron, not a smooth sphere', () => {
  const geo = somaGeometry();
  const pos = geo.getAttribute('position');
  // IcosahedronGeometry(1, 1) = 80 faces × 3 verts; a sphere of the same
  // budget would carry hundreds. Facets are the point.
  assert.ok(pos.count <= 240);
  geo.dispose();
});
