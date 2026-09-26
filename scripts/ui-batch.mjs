#!/usr/bin/env node
// Batch-generate UI design variants for design-lab/ (plan 2026-09-25-001, U3).
// Usage: OPENROUTER_API_KEY=... node scripts/ui-batch.mjs --model <id> [--n 3] [--only tokens,chrome]
// Writes design-lab/out/<surface>/<model>-<i>.md (gitignored).

import { readFileSync, writeFileSync, mkdirSync, readdirSync } from 'node:fs';
import { join, basename } from 'node:path';

const args = process.argv.slice(2);
const get = (flag, dflt) => {
  const i = args.indexOf(flag);
  return i >= 0 ? args[i + 1] : dflt;
};
const MODEL = get('--model', 'moonshotai/kimi-k2');
const N = Number(get('--n', '3'));
const ONLY = get('--only', '')?.split(',').filter(Boolean);
const KEY = process.env.OPENROUTER_API_KEY;
if (!KEY) { console.error('OPENROUTER_API_KEY required'); process.exit(1); }

const promptsDir = new URL('../design-lab/prompts/', import.meta.url).pathname;
const outRoot = new URL('../design-lab/out/', import.meta.url).pathname;
const prompts = readdirSync(promptsDir).filter((f) => f.endsWith('.md'));
const picked = ONLY.length ? prompts.filter((f) => ONLY.includes(basename(f, '.md'))) : prompts;
if (!picked.length) { console.error('no prompts matched'); process.exit(1); }

const system = `You are a senior product engineer generating production-quality UI code.
Follow the prompt's output contract exactly: fenced code blocks only where asked, no prose padding.
Palette: near-black canvas, electric purple accent (#a855f7/#c084fc/#7c3aed), glass panels,
JetBrains Mono chrome text, honest states, zero fake data.`;

for (const file of picked) {
  const surface = basename(file, '.md');
  const prompt = readFileSync(join(promptsDir, file), 'utf8');
  mkdirSync(join(outRoot, surface), { recursive: true });
  for (let i = 1; i <= N; i++) {
    const t0 = Date.now();
    const res = await fetch('https://openrouter.ai/api/v1/chat/completions', {
      method: 'POST',
      headers: { Authorization: `Bearer ${KEY}`, 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: MODEL,
        messages: [
          { role: 'system', content: system },
          { role: 'user', content: prompt },
        ],
        temperature: 0.8,
      }),
    });
    if (!res.ok) { console.error(`${surface} #${i}: HTTP ${res.status} ${await res.text()}`); continue; }
    const json = await res.json();
    const text = json.choices?.[0]?.message?.content ?? '';
    const safe = MODEL.replaceAll('/', '-');
    writeFileSync(join(outRoot, surface, `${safe}-${i}.md`), text);
    console.log(`${surface} #${i} <- ${MODEL} (${((Date.now() - t0) / 1000).toFixed(1)}s, ${text.length} chars)`);
  }
}
