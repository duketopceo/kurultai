# Kurultai launch film v2 — content bible

Product: **Kurultai** — shared, cited, governed memory for agent fleets.
One daemon; every agent reads and writes the same searchable, tiered,
ontology-aware memory over MCP (stdio local) or HTTP (remote).

## The honest problem (v2 framing)

NOT "agents forget everything" — false in 2026; every agent tool has its
own memory feature now. The true claim:

- **Memory is siloed** — each agent's memory lives in its own tool's
  store. Claude's memory is invisible to Codex. Five agents = five brains.
- **No provenance** — you can't ask "where did this claim come from."
- **No trust boundary** — ingested web content vs. curated knowledge sit
  in the same pot; memory features are poisoning vectors.
- **No governance** — nothing proposes-vs-decides what becomes canonical.

Hook (verbatim): **"Every agent has its own memory. None of them share it."**

## New features to show (post-v1 beats)

- **Edge extraction at index time** — zero-LLM ontology edges: the brain
  wires itself as atoms land (no LLM cost).
- **Supersede chains + `--as-of`** — atoms correct/obsolete each other;
  time-travel query "what did the brain know yesterday."
- **remote_ingest** — secret-auth non-loopback ingest → server mode is
  real, not theoretical (demo runs it behind CF Access today).

## Onboarding arc (must be in the film — it IS the product answer)

- **Local**: `kurultai daemon` headless → `kurultai mcp` registered in
  each agent config once → that agent gains 8 tools. Web UI optional.
- **Server**: same binary `docker compose up` → agents connect over
  tailscale/`https://` — the live demo is literally this.
- ⚠️ BLOCKER: local `daemon` maxes all cores (user-observed). Profile +
  fix BEFORE ship — video shows the local command, must be true.

## Verbatim strings (live demo, kurultai-demo.luke-the-duke.com)

- `/api/status` → `{"atoms":235,"brain":{"quarantine_count":15,"trusted_count":220}}`
- Hey thread `"meridian-demo"` turn_cap 24; agents `scout`/`wright` —
  post bodies verbatim from `demo/seed.sh`
- Pending `prop:9ca0a523-9352-…` kind `promote_atom` → `class:decision`,
  `proposed_by:"wright"`, reason "ADR-004 is a load-bearing architecture
  decision — promote it to the ontology"
- MCP surface (8): `search cite remember ask who_knows promote ontology_get ontology_promote`
- `kurultai 0.6.0` · `{"ok":true,"service":"kurultai"}`
- Roster (personal lane): cursor · claude · codex · antigravity · hermes · devin

## Shot list v2 (~40s, 60fps)

| t | scene | motion | beat |
|---|-------|--------|------|
| 0–4s | **Silo hook** | five agent chips, each w/ its own memory bubble, walled off | "Every agent has its own memory. None of them share it." |
| 4–9s | **Wiring** | `kurultai mcp` types into one config line; bubbles collapse into ONE core | "One daemon. Every agent." |
| 9–14s | Atom | note SPLITS → atom card w/ `source:` + tier; quarantine→trusted flip | "Memory with a source. And a trust tier." |
| 14–19s | Search | query types → cited hits deal w/ source pills | "Ask where a claim came from." |
| 19–24s | **Self-wiring** | edges extract at index time; a superseded atom fades, chain advances; `--as-of` scrubber | "It organizes itself. And it remembers what it knew." |
| 24–29s | Hey | meridian-demo rows stack; wright post + repo claim chip | "Agents coordinate here." |
| 29–34s | Ontology | wright's promote_atom card, `pending`, AWAITING HUMAN stamp | "Agents propose. Humans decide." |
| 34–38s | **Onboard split** | left `daemon`+`mcp` (local), right `docker compose`→tailscale/https (server) | "Run it here. Or there." |
| 38–42s | End card | mark + `kurultai daemon --demo` + repo URL | "kurultai v0.6.0" |

## Deliverables

`KurultaiLaunch` (1920×1080) · `KurultaiLaunchSquare` (1:1, X feed) ·
`KurultaiReadmeLoop` (7.5s silent). X post copy drafted separately.
