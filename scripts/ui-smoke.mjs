#!/usr/bin/env node
// Bounded headless-UI smoke check for the daemon UI.
//
// Why this exists: ad-hoc Playwright heredocs have no timeout and have
// orphaned a Chromium GPU process at 700% CPU for hours (SwiftShader renders
// in software on this box). Every UI probe must go through this script or
// carry the same three guards: hard process timeout, --disable-gpu (DOM
// checks don't need a rasterizer), and bounded waits.
//
// Usage:
//   timeout -k 5s 180s node scripts/ui-smoke.mjs [url] [--shot /tmp/x.png]
//
// Exits 0 on success, 2 on timeout, 1 on failure. Playwright resolves from
// hermes-agent's node_modules (this repo has no playwright dep).

import { createRequire } from 'node:module';

const URL_ARG = process.argv[2] && !process.argv[2].startsWith('--')
  ? process.argv[2]
  : 'http://127.0.0.1:8421/ui/ui-next.html';
const SHOT = process.argv.includes('--shot')
  ? process.argv[process.argv.indexOf('--shot') + 1]
  : null;
// Brain v2 options: `--brain v2` loads ?brain=v2 (software WebGL via SwiftShader,
// still bounded by the hard timeout); `--synthetic N` injects N synthetic atoms
// and reports fps / node count / draw calls from `__kurultaiBrain.metrics()`.
const flagVal = (name) => (process.argv.includes(name) ? process.argv[process.argv.indexOf(name) + 1] : null);
const BRAIN = flagVal('--brain') || 'v2'; // v2 is the default renderer; `--brain v1` checks the legacy page
const SYNTH = Number(flagVal('--synthetic') || 0);
const HARD_LIMIT_MS = 120_000;
const NAV_TIMEOUT_MS = 20_000;
const STEP_TIMEOUT_MS = 10_000;

setTimeout(() => {
  console.error(`ui-smoke: hard timeout after ${HARD_LIMIT_MS / 1000}s`);
  process.exit(2);
}, HARD_LIMIT_MS).unref();

const require = createRequire(import.meta.url);
const PW_PATHS = [
  '/home/lukekimball/.hermes/hermes-agent/node_modules',
  `${process.env.HOME}/.hermes/hermes-agent/node_modules`,
];
const { chromium } = require(require.resolve('playwright', { paths: PW_PATHS }));

const errors = [];
const browser = await chromium.launch({
  // DOM/tab checks don't need SwiftShader burning cores; the v2 canvas does.
  args: BRAIN === 'v2' ? ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] : ['--disable-gpu'],
  timeout: NAV_TIMEOUT_MS,
});
try {
  const page = await browser.newPage();
  page.setDefaultTimeout(STEP_TIMEOUT_MS);
  page.setDefaultNavigationTimeout(NAV_TIMEOUT_MS);
  page.on('console', (m) => { if (m.type() === 'error' && BRAIN) errors.push('console: ' + m.text().slice(0, 1200)); });
  page.on('pageerror', (e) => errors.push(String(e).slice(0, 200)));

  if (BRAIN === 'v2') await page.addInitScript(() => { try { localStorage.setItem('kurultai.brain.lastVisit', '1000'); } catch { /* ignore */ } });
  const target = BRAIN ? `${URL_ARG}${URL_ARG.includes('?') ? '&' : '?'}brain=${BRAIN}` : URL_ARG;
  await page.goto(target, { waitUntil: 'domcontentloaded' });
  await page.waitForTimeout(2000);

  if (BRAIN && SYNTH > 0) {
    await page.waitForFunction(() => !!window.__kurultaiBrain, null, { timeout: STEP_TIMEOUT_MS }).catch(() => {});
    const out = await page.evaluate(async (n) => {
      const b = window.__kurultaiBrain;
      if (!b) return { error: 'no __kurultaiBrain' };
      const tags = Array.from({ length: 40 }, (_, i) => `topic-${i}`);
      const atoms = Array.from({ length: n }, (_, i) => ({
        id: `s${i}`, title: `Synthetic ${i}`, summary: '', source: 'synthetic', source_id: `s${i}`,
        tags: [tags[Math.floor(Math.pow(Math.random(), 2) * tags.length)]],
        file: '', indexed_at: '', last_accessed_at: '', score: 0, tier: 'hot', lean: false,
      }));
      const links = [];
      for (let i = 1; i < n; i += 1) if (i % 3 === 0) links.push({ a: `s${i - 1}`, b: `s${i}`, strength: 1 });
      const t0 = performance.now();
      b.setData(atoms, links);
      const setDataMs = Math.round(performance.now() - t0);
      await new Promise((r) => setTimeout(r, 4000));
      return { ...b.metrics(), setDataMs };
    }, SYNTH);
    console.log('v2-metrics', JSON.stringify(out));
    if (SHOT) {
      await page.screenshot({ path: SHOT });
      console.log('screenshot=' + SHOT);
    }
  }

  if (BRAIN === 'v2') {
    // v2 chrome: search + status dot + menu; surfaces behind the palette/drawer.
    const dot = await page.locator('.v2-dot').count();
    console.log('v2 status-dot', dot);
    const badge = await page.getByRole('button', { name: /\d+ new/ }).count();
    console.log('v2 new-badge', badge);
    if (badge) { await page.getByRole('button', { name: /\d+ new/ }).first().click(); await page.waitForTimeout(500); if (SHOT) await page.screenshot({ path: SHOT.replace(/\.png$/, '-new.png') }); await page.getByRole('button', { name: /\d+ new/ }).first().click(); }
    await page.keyboard.press('Control+k');
    const palette = await page.getByRole('dialog', { name: 'Command palette' }).count();
    console.log('v2 palette-open', palette);
    if (SHOT) await page.screenshot({ path: SHOT.replace(/\.png$/, '-palette.png') });
    await page.keyboard.type('pulse');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);
    const drawer = await page.locator('.v2-drawer').count();
    console.log('v2 drawer-open', drawer);
    if (SHOT) await page.screenshot({ path: SHOT.replace(/\.png$/, '-drawer.png') });
    await page.keyboard.press('Escape');
    await page.fill('input[aria-label="Search memory"]', 'x');
    const clear = await page.getByRole('button', { name: /Clear search/ }).count();
    console.log('v2 clear-control', clear);
    await page.keyboard.press('Control+k');
    await page.keyboard.type('repos');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(2500);
    console.log('v2 repos-view', await page.locator('.v2-repos').count());
    if (SHOT) await page.screenshot({ path: SHOT.replace(/\.png$/, '-repos.png') });
  } else {
    for (const tab of ['pulse', 'ontology', 'ask', 'store', 'hey']) {
      const el = page.getByRole('tab', { name: tab, exact: true }).first();
      await el.click().catch(() => console.log('click-fail', tab));
      const sel = await el.getAttribute('aria-selected').catch(() => null);
      console.log('tab', tab, 'selected=' + sel);
    }
  }

  if (SHOT && !(BRAIN && SYNTH > 0)) {
    await page.screenshot({ path: SHOT });
    console.log('screenshot=' + SHOT);
  }
  console.log('ERRORS', errors.length ? errors.join(' || ') : 'none');
} finally {
  await browser.close();
}
