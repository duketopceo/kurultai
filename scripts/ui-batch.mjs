#!/usr/bin/env node
// Batch-generate UI design variants for design-lab/ (plan 2026-09-25-001, U3).
// Usage: OPENROUTER_API_KEY=... node scripts/ui-batch.mjs --model <id> [--n 3] [--only tokens,chrome]
// Writes design-lab/out/<surface>/<model>-<i>.md (gitignored).

import { readFileSync, writeFileSync, mkdirSync, readdirSync, existsSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join, basename } from 'node:path';

const args = process.argv.slice(2);
const get = (flag, dflt) => {
  const i = args.indexOf(flag);
  return i >= 0 ? (args[i + 1] ?? dflt) : dflt;
};
const MODEL = get('--model', 'moonshotai/kimi-k2');
const N = Number(get('--n', '3'));
if (!Number.isInteger(N) || N < 1) { console.error(`--n must be a positive integer (got ${get('--n', '3')})`); process.exit(1); }
const ONLY = get('--only', '').split(',').filter(Boolean);
const FORCE = process.argv.includes('--force');
const KEY = process.env.OPENROUTER_API_KEY;
if (!KEY) { console.error('OPENROUTER_API_KEY required'); process.exit(1); }

const promptsDir = fileURLToPath(new URL('../design-lab/prompts/', import.meta.url));
const outRoot = fileURLToPath(new URL('../design-lab/out/', import.meta.url));
let prompts;
try {
  prompts = readdirSync(promptsDir).filter((f) => f.endsWith('.md') && f !== 'INDEX.md' && f !== 'README.md');
} catch {
  console.error(`no prompts dir at ${promptsDir} — expected design-lab/prompts/*.md`);
  process.exit(1);
}
const picked = ONLY.length ? prompts.filter((f) => ONLY.includes(basename(f, '.md'))) : prompts;
if (!picked.length) { console.error('no prompts matched'); process.exit(1); }

const system = `You are a senior product engineer generating production-quality UI code.
Follow the prompt's output contract exactly: fenced code blocks only where asked, no prose padding.
Palette: near-black canvas, electric purple accent (#a855f7/#c084fc/#7c3aed), glass panels,
JetBrains Mono chrome text, honest states, zero fake data.`;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

for (const file of picked) {
  const surface = basename(file, '.md');
  const prompt = readFileSync(join(promptsDir, file), 'utf8');
  mkdirSync(join(outRoot, surface), { recursive: true });
  for (let i = 1; i <= N; i++) {
    const safe = MODEL.replaceAll('/', '-');
    const dest = join(outRoot, surface, `${safe}-${i}.md`);
    if (!FORCE && existsSync(dest) && statSync(dest).size > 0) {
      console.log(`${surface} #${i} <- ${MODEL} (cached)`);
      continue;
    }
    const t0 = Date.now();
    let ok = false;
    for (let attempt = 0; attempt < 3 && !ok; attempt++) {
      try {
        const res = await fetch('https://openrouter.ai/api/v1/chat/completions', {
          method: 'POST',
          headers: {
            Authorization: `Bearer ${KEY}`,
            'Content-Type': 'application/json',
            'HTTP-Referer': 'https://github.com/duketopceo/kurultai',
            'X-Title': 'Kurultai',
          },
          signal: AbortSignal.timeout(300_000),
          body: JSON.stringify({
            model: MODEL,
            messages: [
              { role: 'system', content: system },
              { role: 'user', content: prompt },
            ],
            temperature: 0.8,
          }),
        });
        if (!res.ok) {
          const body = await res.text();
          if ((res.status === 429 || res.status >= 500) && attempt < 2) {
            await sleep(2000 * (attempt + 1));
            continue;
          }
          console.error(`${surface} #${i}: HTTP ${res.status} ${body.slice(0, 200)}`);
          break;
        }
        const json = await res.json();
        const text = json.choices?.[0]?.message?.content ?? '';
        if (!text.trim()) {
          console.error(`${surface} #${i}: empty completion (HTTP 200, attempt ${attempt + 1}/3)`);
          if (attempt < 2) { await sleep(2000 * (attempt + 1)); continue; }
          break;
        }
        writeFileSync(dest, text);
        console.log(`${surface} #${i} <- ${MODEL} (${((Date.now() - t0) / 1000).toFixed(1)}s, ${text.length} chars)`);
        ok = true;
      } catch (err) {
        if (attempt < 2) { await sleep(2000 * (attempt + 1)); continue; }
        console.error(`${surface} #${i}: ${err.message}`);
      }
    }
    if (!ok) process.exitCode = 1;
  }
}
