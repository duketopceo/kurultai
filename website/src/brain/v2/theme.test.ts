import test from 'node:test';
import assert from 'node:assert/strict';
import { parseThemePref, resolveTheme } from './theme.ts';

test('theme pref: light by default, remembered dark/system kept', () => {
  assert.equal(parseThemePref('light'), 'light');
  assert.equal(parseThemePref('dark'), 'dark');
  assert.equal(parseThemePref('system'), 'system');
  assert.equal(parseThemePref(null), 'light');
  assert.equal(parseThemePref('purple'), 'light');
});

test('system follows the OS, explicit wins', () => {
  assert.equal(resolveTheme('system', true), 'light');
  assert.equal(resolveTheme('system', false), 'dark');
  assert.equal(resolveTheme('dark', true), 'dark');
});
