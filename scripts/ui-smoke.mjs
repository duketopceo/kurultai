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
  args: ['--disable-gpu'], // DOM/tab checks don't need SwiftShader burning cores
  timeout: NAV_TIMEOUT_MS,
});
try {
  const page = await browser.newPage();
  page.setDefaultTimeout(STEP_TIMEOUT_MS);
  page.setDefaultNavigationTimeout(NAV_TIMEOUT_MS);
  page.on('pageerror', (e) => errors.push(String(e).slice(0, 200)));

  await page.goto(URL_ARG, { waitUntil: 'domcontentloaded' });
  await page.waitForTimeout(2000);

  for (const tab of ['pulse', 'ontology', 'ask', 'store', 'hey']) {
    const el = page.getByRole('tab', { name: tab, exact: true }).first();
    await el.click().catch(() => console.log('click-fail', tab));
    const sel = await el.getAttribute('aria-selected').catch(() => null);
    console.log('tab', tab, 'selected=' + sel);
  }

  if (SHOT) {
    await page.screenshot({ path: SHOT });
    console.log('screenshot=' + SHOT);
  }
  console.log('ERRORS', errors.length ? errors.join(' || ') : 'none');
} finally {
  await browser.close();
}
