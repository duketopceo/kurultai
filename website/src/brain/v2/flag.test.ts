import test from 'node:test';
import assert from 'node:assert/strict';
import { parseBrainVersion } from './flag.ts';

test('brain flag: v2 default, v1 only on explicit ?brain=v1', () => {
  assert.equal(parseBrainVersion(''), 'v2');
  assert.equal(parseBrainVersion('?brain=v2'), 'v2');
  assert.equal(parseBrainVersion('?brain=v1'), 'v1');
  assert.equal(parseBrainVersion('?brain=zzz'), 'v2');
});
