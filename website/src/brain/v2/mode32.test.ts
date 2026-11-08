import test from 'node:test';
import assert from 'node:assert/strict';
import { parseMode32, readMode32, MODE32_KEY } from './mode32.ts';

test('?mode=32bit forces on, ?mode=normal forces off, else null', () => {
  assert.equal(parseMode32('?mode=32bit'), true);
  assert.equal(parseMode32('?brain=v2&mode=normal'), false);
  assert.equal(parseMode32('?brain=v2'), null);
});

test('readMode32 survives a throwing/absent storage', () => {
  const g = globalThis as unknown as { window?: unknown };
  g.window = { location: { search: '' }, get localStorage() { throw new Error('blocked'); } };
  assert.equal(readMode32(), false);
  g.window = { location: { search: '' }, localStorage: { getItem: (k: string) => (k === MODE32_KEY ? '1' : null) } };
  assert.equal(readMode32(), true);
  g.window = { location: { search: '?mode=normal' }, localStorage: { getItem: () => '1' } };
  assert.equal(readMode32(), false);
  delete g.window;
});
