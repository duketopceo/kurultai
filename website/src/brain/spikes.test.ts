import { test } from 'node:test';
import assert from 'node:assert/strict';
import * as THREE from 'three';
import {
  emissionRate,
  heatOf,
  nextEmission,
  SpikePool,
  SPIKE_POOL_CAPACITY,
} from './spikes.ts';

const NOW = Date.parse('2026-09-12T00:00:00Z');

test('heatOf orders hot recent high-score atoms above cold stale ones', () => {
  const hot = heatOf(
    { score: 0.9, tier: 'hot', last_accessed_at: '2026-09-11T00:00:00Z' },
    NOW,
    10,
  );
  const cold = heatOf(
    { score: 0.1, tier: 'cold', last_accessed_at: '2026-06-01T00:00:00Z' },
    NOW,
    0,
  );
  assert.ok(hot > cold);
  assert.ok(hot > 0.6);
  assert.ok(cold < 0.25);
});

test('heatOf tolerates missing recency without throwing', () => {
  const heat = heatOf({ score: 0.5, tier: 'warm' }, NOW, 4);
  assert.ok(heat > 0 && heat <= 1);
});

test('emissionRate: floor keeps cold edges alive, hover saturates', () => {
  const floor = emissionRate(0, 0, 'none');
  assert.ok(floor >= 0.04);
  const busy = emissionRate(1, 1, 'none');
  assert.ok(busy > floor);
  assert.equal(emissionRate(0.5, 0.5, 'towardA'), 2.5);
  assert.ok(emissionRate(1, 1, 'unrelated') < floor);
});

test('nextEmission fires deterministically as the accumulator fills', () => {
  let acc = 0;
  // rate 1/s, dt 0.4 → 0.4 (no), 0.8 (no), 1.2 → fires, carry 0.2.
  let r = nextEmission(acc, 1, 0.4);
  assert.equal(r.fired, false);
  acc = r.acc;
  r = nextEmission(acc, 1, 0.4);
  assert.equal(r.fired, false);
  acc = r.acc;
  r = nextEmission(acc, 1, 0.4);
  assert.equal(r.fired, true);
  assert.ok(r.acc < 1 && r.acc > 0);
});

test('nextEmission never fires at zero rate', () => {
  assert.equal(nextEmission(0.99, 0, 1).fired, false);
});

test('pool caps at capacity and releases on arrival', () => {
  const pool = new SpikePool(0xffffff);
  const curve = new THREE.LineCurve3(
    new THREE.Vector3(0, 0, 0),
    new THREE.Vector3(1, 0, 0),
  );
  const origin = new THREE.Vector3();
  // Fill the pool completely — claims beyond capacity must fail, not alloc.
  for (let i = 0; i < SPIKE_POOL_CAPACITY; i++) {
    assert.equal(pool.claim(0, 1, 1, origin), true);
  }
  assert.equal(pool.claim(0, 1, 1, origin), false);
  assert.equal(pool.active, SPIKE_POOL_CAPACITY);

  let arrivals = 0;
  pool.advance(1.1, () => curve, () => arrivals++);
  assert.equal(arrivals, SPIKE_POOL_CAPACITY);
  assert.equal(pool.active, 0);
  // Slots recycle — the pool is usable again after full drain.
  assert.equal(pool.claim(0, -1, 1, origin), true);
});

test('spike travels the curve and alpha fades near arrival', () => {
  const pool = new SpikePool(0xffffff);
  const curve = new THREE.LineCurve3(
    new THREE.Vector3(0, 0, 0),
    new THREE.Vector3(2, 0, 0),
  );
  pool.claim(0, 1, 1, new THREE.Vector3());
  pool.advance(0.5, () => curve, () => {});
  const pos = (pool.points.geometry.getAttribute('position') as THREE.BufferAttribute);
  assert.ok(pos.getX(0) > 0.5 && pos.getX(0) < 1.5);
});

test('missing curve releases the slot silently', () => {
  const pool = new SpikePool(0xffffff);
  pool.claim(3, 1, 1, new THREE.Vector3());
  pool.advance(0.1, () => null, () => {});
  assert.equal(pool.active, 0);
});
