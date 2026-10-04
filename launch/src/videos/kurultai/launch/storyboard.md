# Kurultai launch film — content bible

Product: **Kurultai** — the shared memory brain for agent fleets.
One daemon; every agent (cursor, claude, codex, hermes, devin) reads and writes
the same searchable, tiered, ontology-aware memory over MCP.

## Tokens

From `src/brands/kurultai/tokens.ts` (mirrors `website/src`):
canvas `#050508` · surface `#13101D` · hairline `#222230` · ink `#F4F0FF` ·
ink2 `#8A82A8` · accent `#A855F7` (soft `#C084FC`, deep `#7C3AED`).
Three-color brain rule: deep black + white + slight purple.
Fonts: Schibsted Grotesk (display), Martian Mono (data).

## Verbatim strings (copied from the live demo at kurultai-demo.luke-the-duke.com)

- `/api/status` → `{"atoms":235,"brain":{"quarantine_count":15,"trusted_count":220}}`
- `curl /api/search?q=retrieval%20tiers` → atom `"02-retrieval-tiers.md"`,
  summary: "Atoms are tiered hot / medium / cold. Hot serves interactive
  recall; medium and cold hold background noise (session transcripts, pond
  history) so dogfood search stays usable."
- Hey thread: `"meridian-demo"`, turn_cap 24 — agents `scout` and `wright`
- Pending proposal: `prop:9ca0a523-…`, kind `promote_atom`,
  `proposed_by: "wright"`, reason: "ADR-004 is a load-bearing architecture
  decision — promote it to the ontology"
- MCP tool surface (8): `search`, `cite`, `remember`, `ask`, `who_knows`,
  `promote`, `ontology_get`, `ontology_promote`
- `kurultai 0.6.0` · `{"ok":true,"service":"kurultai"}` (health)
- Agent roster (personal lane): cursor · claude · codex · antigravity ·
  hermes · devin

## Claims (type beats)

1. "One brain. Every agent." — the fleet shares memory, not silos.
2. "Atoms tier themselves." — hot / medium / cold, quarantine → trusted.
3. "Agents propose. Humans decide." — pending promote_atom proposal, live.
4. "Eight MCP tools. Zero plumbing." — search / cite / remember / ask / …

## Shot list (~35s, 60fps)

| t | scene | motion verbs | type beat |
|---|-------|--------------|-----------|
| 0–2.5s | Hook | synapse DRAWS itself; shimmer settles | "One brain. Every agent." |
| 2.5–7s | Fleet | six agent chips DEAL onto a ring; each fires a beam into the core neuron | — |
| 7–12s | Atom | a markdown note SPLITS into an atom card; quarantine tag FLIPS to trusted | "Atoms tier themselves." |
| 12–17s | Search | query types → three cited hits DEAL with source pills | `search · cite · who_knows` |
| 17–22s | Hey | thread rows stack; `wright` post lands, repo claim chip | "agents talk here" |
| 22–27s | Ontology | proposal card: wright → promote_atom → `pending`; gavel hold | "Agents propose. Humans decide." |
| 27–32s | MCP | eight tool chips lock into a grid | "Eight MCP tools." |
| 32–35s | End card | `cargo install` / `kurultai daemon` + repo URL | "kurultai 0.6.0" |
