import test from 'node:test';
import assert from 'node:assert/strict';
import { laneDefaultTheme, parseThemePref, resolveTheme } from './theme.ts';

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

test('lane defaults: work light, knowledge dark, others unchanged, stored choice wins', () => {
  assert.equal(parseThemePref(null, 'work.shippedit.dev'), 'light');
  assert.equal(parseThemePref(null, 'knowledge.shippedit.dev'), 'dark');
  assert.equal(parseThemePref(null, 'localhost'), 'light');
  assert.equal(parseThemePref(null, '127.0.0.1'), 'light');
  assert.equal(parseThemePref('dark', 'work.shippedit.dev'), 'dark');
  assert.equal(parseThemePref('light', 'knowledge.shippedit.dev'), 'light');
  assert.equal(parseThemePref('system', 'knowledge.shippedit.dev'), 'system');
  assert.equal(parseThemePref('purple', 'knowledge.shippedit.dev'), 'dark');
  assert.equal(laneDefaultTheme('KNOWLEDGE.shippedit.dev'), 'dark');
  assert.equal(laneDefaultTheme('knowledge.evil.example.work.dev'), 'dark');
});
