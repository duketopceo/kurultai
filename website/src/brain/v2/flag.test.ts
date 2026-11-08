import test from 'node:test';
import assert from 'node:assert/strict';
import { parseBrainVersion } from './flag.ts';

test('brain flag: v2 opt-in, v1 default and explicit', () => {
  assert.equal(parseBrainVersion('?brain=v2'), 'v2');
  assert.equal(parseBrainVersion('?brain=v1'), 'v1');
  assert.equal(parseBrainVersion(''), 'v1');
  assert.equal(parseBrainVersion('?brain=zzz'), 'v1');
});
