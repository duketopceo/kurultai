import assert from 'node:assert/strict';
import test from 'node:test';
import { shouldInitGlitchtip } from './glitchtip.ts';

test('shouldInitGlitchtip is false without DSN', () => {
  assert.equal(shouldInitGlitchtip(undefined), false);
  assert.equal(shouldInitGlitchtip(''), false);
  assert.equal(shouldInitGlitchtip('   '), false);
});

test('shouldInitGlitchtip is true with DSN', () => {
  assert.equal(shouldInitGlitchtip('https://key@errors.example/1'), true);
});
