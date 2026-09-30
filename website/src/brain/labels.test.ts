import { test } from 'node:test';
import assert from 'node:assert/strict';
import { computeLabelPlan, type LabelInput } from './labels.ts';

const distances = [0, 1, 2, 3, 4, 5, 6];

function makeNodes(count: number): LabelInput['ids'] {
  return Array.from({ length: count }, (_, i) => ({ id: `node-${i}`, title: `Memory ${i}` }));
}

/** Hub topology: node-0..29 each link to node-100, so ranking is deterministic. */
function hubLinks(): LabelInput['links'] {
  return Array.from({ length: 30 }, (_, i) => ({ a: `node-${i}`, b: 'node-100' }));
}

function plan(
  nodes: LabelInput['ids'],
  overrides: Partial<LabelInput> = {},
): Map<string, number> {
  const input: LabelInput = {
    ids: nodes,
    links: hubLinks(),
    mode: 'brain',
    showLabels: true,
    hoverId: null,
    hoverConnected: [],
    cameraDistance: 2,
    ...overrides,
  };
  return new Map(computeLabelPlan(input).labels.map((l): [string, number] => [l.id, l.opacity]));
}

test('brain keeps the highest-degree 24 labels on at every camera distance', () => {
  const labels = plan(makeNodes(320));
  const top = ['node-100', ...Array.from({ length: 23 }, (_, i) => `node-${i}`)];
  for (const cameraDistance of distances) {
    const at = plan(makeNodes(320), { cameraDistance });
    for (const id of top) {
      assert.equal(at.get(id), 0.9, `top-ranked ${id} should stay at 0.9 at distance ${cameraDistance}`);
    }
  }
  assert.ok(labels.get('node-100'), 'hub labeled in default plan');
});

test('brain never exceeds 200 labels, including a hovered low-ranked node', () => {
  for (const count of [199, 200, 201, 512]) {
    const nodes = makeNodes(count);
    for (const cameraDistance of distances) {
      for (const hoveredId of [null, `node-${count - 1}`]) {
        const labels = plan(nodes, { cameraDistance, hoverId: hoveredId });
        assert.ok(
          labels.size <= Math.min(count, 200),
          `${labels.size} labels for ${count} nodes at distance ${cameraDistance}`,
        );
      }
    }
  }
});

test('camera fading stays bounded and does not brighten labels as distance increases', () => {
  const nodes = makeNodes(64);
  let previous = plan(nodes, { cameraDistance: distances[0] });
  assert.ok(
    nodes.some((node) => node.id !== 'node-100' && !node.id.match(/^node-([0-9]|1[0-9]|2[0-3])$/) && (previous.get(node.id) ?? 0) > 0),
    'the near-camera plan should include labels beyond the always-on 24',
  );
  for (const cameraDistance of distances.slice(1)) {
    const current = plan(nodes, { cameraDistance });
    for (const node of nodes) {
      assert.ok(
        (current.get(node.id) ?? 0) <= (previous.get(node.id) ?? 0) + 1e-12,
        `${node.id} brightened at distance ${cameraDistance}`,
      );
    }
    previous = current;
  }
  assert.ok(
    nodes.some((node) => (previous.get(node.id) ?? 0) < 0.9),
    'distant non-core labels should fade',
  );
});

test('hover overrides camera fading and ranking without displacing the top 24', () => {
  const nodes = makeNodes(320);
  const hoveredId = 'node-319';
  const at = plan(nodes, { hoverId: hoveredId, cameraDistance: 100 });
  assert.equal(at.get(hoveredId), 1, 'hovered node is fully on');
  for (const id of ['node-100', 'node-0', 'node-22']) {
    assert.equal(at.get(id), 0.9, `top-24 member ${id} stays on`);
  }
  assert.equal(at.get('node-300') ?? 0, 0, 'non-top non-hover stays off at far distance');
});

test('ontology mode labels every id at full opacity up to the cap', () => {
  const few = plan(makeNodes(10), { mode: 'ontology' });
  assert.equal(few.size, 10);
  for (const opacity of few.values()) assert.equal(opacity, 1);
  const many = plan(makeNodes(250), { mode: 'ontology' });
  assert.equal(many.size, 200);
});

test('brain mode with showLabels=false is empty; empty input is safe; cap is 200', () => {
  const off = computeLabelPlan({
    ids: makeNodes(50),
    links: hubLinks(),
    mode: 'brain',
    showLabels: false,
    hoverId: null,
    hoverConnected: [],
    cameraDistance: 1,
  });
  assert.deepEqual(off, { labels: [], cap: 200 });
  const empty = computeLabelPlan({
    ids: [],
    links: [],
    mode: 'ontology',
    showLabels: true,
    hoverId: null,
    hoverConnected: [],
    cameraDistance: 1,
  });
  assert.deepEqual(empty, { labels: [], cap: 200 });
});
