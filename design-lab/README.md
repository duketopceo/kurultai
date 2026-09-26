# design-lab

Scratch space for the UI/UX rebuild (`docs/plans/2026-09-25-001`, unit U3).

- `prompts/` — per-surface generation prompts, seeded from `docs/design/ui-definitions.md` + `docs/design/research-bases.md`. **Committed** — they are the spec-to-model contract.
- `out/` — generated variants. **Gitignored** — candidates land here, get scored against the definitions doc, and winners get stitched by hand into `website/ui-next.html` + `website/src/next/`.

## Run

```bash
OPENROUTER_API_KEY=... node scripts/ui-batch.mjs --model moonshotai/kimi-k2 --n 3
OPENROUTER_API_KEY=... node scripts/ui-batch.mjs --model openai/gpt-5 --n 3 --only tokens,strip
```

`--model` is any OpenRouter model id (Kimi or GPT per the plan — no Fable). `--n` = variants per surface. `--only` limits to comma-separated prompt basenames.

Scoring rubric (from the definitions doc): single dark purple palette · hairline glass panels · mono micro-labels · honest empty/error states · no fake data · no extra Brain chrome · `prefers-reduced-motion` respected.
