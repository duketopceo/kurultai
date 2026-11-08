import test from 'node:test';
import assert from 'node:assert/strict';
import { parseThemePref, resolveTheme } from './theme.ts';

test('theme pref parses stored values, system when absent/garbage', () => {
  assert.equal(parseThemePref('light'), 'light');
  assert.equal(parseThemePref('dark'), 'dark');
  assert.equal(parseThemePref(null), 'system');
  assert.equal(parseThemePref('purple'), 'system');
});

test('system follows the OS, explicit wins', () => {
  assert.equal(resolveTheme('system', true), 'light');
  assert.equal(resolveTheme('system', false), 'dark');
  assert.equal(resolveTheme('dark', true), 'dark');
});
