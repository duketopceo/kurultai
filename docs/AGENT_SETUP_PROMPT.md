# Agent setup prompt (copy-paste)

Paste the fenced block below into **any** coding agent (Cursor, Claude Code, Codex, Hermes, ChatGPT with tools, etc.). It tells the agent where Kurultai lives, how to install and wire it, how to use it, and how to dump memory into it.

Also see the portable skill: [`skills/kurultai-brain/SKILL.md`](../skills/kurultai-brain/SKILL.md).

---

````markdown
# Task: Set up Kurultai (local knowledge brain) for this user

You are helping install and wire **Kurultai** — a local-first knowledge brain (SQLite + FTS5 + sqlite-vec) that agents and humans search/ask against. Do not invent a parallel system. Prefer the official repo and CLI.

## Where to find it

- **GitHub (source of truth):** https://github.com/duketopceo/kurultai
- **License:** MIT (public open source)
- **Docs in-repo:** README.md · CONCEPTS.md · docs/multi-user-kurultai.md · skills/kurultai-brain/SKILL.md
- **This prompt lives at:** docs/AGENT_SETUP_PROMPT.md (or AGENT_SETUP_PROMPT.md at repo root)

Clone if needed:

```bash
git clone https://github.com/duketopceo/kurultai.git
cd kurultai
```

## Install the CLI

Prefer one of:

```bash
# Install script (Linux/macOS)
curl -fsSL https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.sh | bash

# Or from source / git tag
cargo install --git https://github.com/duketopceo/kurultai --tag v0.4.0 --locked

# Or from a local clone
cargo install --path . --locked
```

Optional:

```bash
export KURULTAI_ENV=dev          # stores under …/kurultai/dev/
export RUST_LOG=kurultai=debug
```

API keys (embeddings / LLM ask) via **environment only** — never write secrets into config.toml:

```bash
export OPENROUTER_API_KEY=…      # or KURULTAI_API_KEY
```

FTS search works **without** a key. Vectors / cloud ask stay off until a key (or local ONNX embed) is configured.

## Store docs on this device (solo)

One command provisions a markdown folder, enables `[sources.notes]`, and writes a **tagged** starter note (untagged markdown is quarantined):

```bash
kurultai init --docs                 # ~/Documents/kurultai (or ~/kurultai)
# kurultai init --docs /ABSOLUTE/PATH/TO/notes
kurultai init --docs --agent none    # CLI / Brain UI only — skip MCP
# kurultai init --docs --agent none --index   # also index now
```

Do **not** overwrite existing files in that folder. Re-running `--docs` refreshes `root_path` / `enabled` only.

## Wire agents (MCP)

```bash
kurultai init --agent all        # cursor + claude + codex + hermes
# or one of: cursor | claude | codex | hermes | none
```

That writes host MCP configs (`kurultai mcp` stdio). **Restart the agent** so tools reload. `--agent none` skips MCP.

MCP tools: `search`, `cite`, `ask`, `who_knows`, `remember`, `promote`, `ontology_get`, `ontology_promote`.

Portable skill to teach agents tool discipline:

- https://github.com/duketopceo/kurultai/tree/main/skills/kurultai-brain

## Configure extra sources (optional)

`init --docs` already enables markdown. To add JSON / code / etc., edit `~/.config/kurultai/config.toml`. Example extra vault:

```toml
environment = "dev"

[sources.notes]
enabled = true
kind = "markdown"
root_path = "/ABSOLUTE/PATH/TO/notes"
```

Markdown atoms need **≥1 tag** in YAML frontmatter (`tags:`). Untagged files land in quarantine and are skipped by default search; promote after fixing: `kurultai promote <atom_id>`.

JSON / NDJSON dumps:

```toml
[sources.data]
enabled = true
kind = "json"                 # .json arrays or .jsonl / .ndjson
root_path = "/ABSOLUTE/PATH/TO/dumps"
# optional: id_field = "url"  # stable source_id (default: id)
```

Other kinds: `dayflow`, `pond`, `github` (local checkout). See README.

## Index and verify

```bash
kurultai index --full
kurultai status
kurultai search "a phrase you know exists" --limit 10
kurultai ask "what do we know about X?"
kurultai who-knows "X"
```

Optional daemon + Brain UI:

```bash
kurultai daemon --port 8421
# open http://127.0.0.1:8421/ui/   (trailing slash matters)
```

## How to use it (day to day)

| Need | Do this |
|------|---------|
| Ground an answer | MCP `search` first; `cite` to pin one source; `ask` only when you need synthesis |
| See which source knows a topic | MCP `who_knows` or `kurultai who-knows` |
| Keep working across devices | `kurultai export -o brain.kurultai` then `kurultai import` / `--combine` on the other machine (pack is **unencrypted** — trusted transfer only; delete after import) |
| Browse spatially | Daemon Brain UI at `/ui/` |

**Guardrails for agents:** return excerpts + citations, not whole-vault dumps. Prefer `search` over `ask`. Never put API keys in config files.

## How to dump memory into Kurultai

Choose the path that matches the material:

### A) Distilled facts from an agent session (preferred for chat lore)

Use MCP **`remember(title, summary, tags)`** — distill first. Good remember = short title + one-line summary + 1–3 tags. **Do not** paste raw transcripts or whole files into `remember`.

### B) Notes / markdown vault

1. Put `.md` files under a folder with YAML `tags: […]`.
2. Point `[sources.*.kind = "markdown"]` `root_path` at that folder.
3. `kurultai index --full` (or run `daemon` with poll/watch).

### C) Bulk JSON / chat / export dumps

1. Drop `.json` / `.jsonl` / `.ndjson` under a folder (stable `id` field when possible).
2. Enable `kind = "json"` source as above.
3. `kurultai index --full`.

### D) Code checkout

Enable `kind = "github"` with `root_path` to a local clone; then `kurultai index --full`.

### E) Bring an existing brain from another machine

```bash
# device A
kurultai export -o brain.kurultai

# device B — empty
kurultai import brain.kurultai
# device B — already has atoms
kurultai import brain.kurultai --combine
```

Then remap `[sources.*.root_path]`, set API keys in env, re-run `kurultai init --agent …`.

### F) Quarantine → trusted

If something was ingested without tags / failed the quality gate:

```bash
kurultai promote <atom_id>
```

(or MCP `promote`)

## Success checklist

- [ ] `kurultai` binary on PATH
- [ ] `~/.config/kurultai/config.toml` exists with ≥1 enabled source and real absolute `root_path` (`kurultai init --docs` does this)
- [ ] `kurultai index --full` succeeds
- [ ] `kurultai search` finds a known phrase
- [ ] Agent MCP restarted and can call `search` / `remember`
- [ ] (Optional) `http://127.0.0.1:8421/ui/` loads with daemon running

If anything fails, read the CLI error, check `root_path` exists, and run `kurultai status`. Do not invent alternate storage layouts.
````
