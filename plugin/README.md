# Kurultai — Agent Zero Plugin

Local integration between Agent Zero and **[Kurultai](https://github.com/AUTH_LOGIN/kurultai)**,
the Rust knowledge-retrieval brain: a SQLite + FTS5 + sqlite-vec index of your notes,
agent chats, JSON dumps, Dayflow/Pond data, and local code checkouts, queried with
token-capped excerpts and citations instead of whole-file dumps.

## What the plugin provides

Five Agent Zero tools:

| Tool | Purpose | Transport |
|------|---------|-----------|
| `kurultai_search` | Hybrid/FTS search — ranked excerpts with source citations (preferred default) | CLI or daemon |
| `kurultai_ask` | Synthesized, cited answer across atoms (higher cost — use sparingly) | CLI or daemon |
| `kurultai_who_knows` | Which sources know about a topic (scope before diving in) | CLI or daemon |
| `kurultai_remember` | Store a distilled fact (title + summary + 1–3 tags) | Daemon only (`POST /api/recall`) |
| `kurultai_status` | Binary/daemon/store diagnostics + kurultai's own `status` | CLI or daemon |

## Integration design (why CLI + HTTP, not MCP)

- **CLI mode (primary)** — shells out to `kurultai search|ask|who-knows|status` with
  `--plain`. Works with no daemon running and **no API key** (FTS-only mode uses
  `NullEmbedder`; extractive `ask` still works). No background process management,
  no extra packages (stdlib only).
- **HTTP mode (optional upgrade)** — when `server_url` points at a running
  `kurultai daemon` and it answers `/health`, tools switch to the JSON API
  (`/api/search`, `/api/ask`, `/api/status`, `/api/recall`) for structured results,
  falling back to the CLI automatically on failure. `kurultai_remember` requires
  this mode because `remember` is an MCP/daemon write tool — the CLI has no write
  subcommand.
- **MCP (not duplicated here)** — Agent Zero already supports MCP servers natively.
  If you want the full MCP tool surface (`search`, `cite`, `remember`, `ask`,
  `who_knows`, `promote`, `ontology_get`, `ontology_promote`), register
  `kurultai mcp` in **Settings → MCP servers** instead of this plugin's tools:

  ```json
  {
    "mcpServers": {
      "kurultai": {
        "command": "kurultai",
        "args": ["mcp"]
      }
    }
  }
  ```

  Use either the plugin tools or the MCP registration to avoid duplicated tool names.

## Prerequisites

1. **Install the kurultai binary** (macOS/Linux):
   ```bash
   curl -fsSL https://raw.githubusercontent.com/agent0ai/kurultai/main/scripts/install.sh | bash
   # or from source:
   cargo install --git https://github.com/AUTH_LOGIN/kurultai --locked
   ```
2. **Initialize a brain** (once):
   ```bash
   kurultai init --docs     # creates ~/Documents/kurultai + config.toml
   kurultai index --full
   kurultai search "welcome" --limit 10
   ```
3. **(Optional) run the daemon** for JSON responses, the Brain UI at
   `http://127.0.0.1:8421/ui/`, and the `kurultai_remember` write path:
   ```bash
   kurultai daemon --port 8421
   ```
4. **(Optional) OpenRouter key** for vector recall, reranking, and LLM `ask`:
   keep `OPENROUTER_API_KEY` in **Agent Zero secrets / the process environment**.
   This plugin never reads, stores, or echoes it; the kurultai binary picks it up
   from its own inherited environment. FTS search works without any key.

## Activation

1. The plugin lives at `/a0/usr/plugins/kurultai/` — it is discovered automatically
   (manifest: `plugin.yaml`, name `kurultai`). Toggle it in **Settings → Plugins** if needed.
2. Open **Settings → External → Kurultai Knowledge Brain** and set:
   - *Kurultai binary path* — only if `kurultai` is not on `PATH` (e.g. `~/.cargo/bin/kurultai`)
   - *Daemon server URL* — e.g. `http://127.0.0.1:8421` (optional; enables HTTP + remember)
   - *Kurultai config file / environment* — only for non-default `KURULTAI_CONFIG` / `KURULTAI_ENV`
   - *Store database path* — informational presence check for `kurultai_status`
3. Ask the agent: "search my brain for …" / "what do my notes know about …" —
   or call `kurultai_status` first to verify wiring.

## Guardrails (inherited from Kurultai)

- **Excerpts, not files.** The brain is a retrieval layer; your files remain the
  system of record. Edit files, not the brain.
- **Default to `search`; escalate to `ask`** only when synthesis across atoms is needed.
- **`remember` is for distilled facts** — a title, a one-line summary, 1–3 tags.
  Never dump raw transcripts or whole files. At least one tag is required
  (untagged atoms land in quarantine).

## Auto-memory (passive)

When **auto_memory** is enabled in settings and a daemon is running, the plugin
passively stores distilled summaries of each AI monologue response into the brain.

- **Fire-and-forget**: runs as a background task, never blocks the agent loop
- **Sanitized**: strips secret-like substrings (API keys, tokens, Bearer headers)
- **Deduplicated**: identical summaries within 5 minutes are skipped
- **Minimum length**: responses shorter than `auto_memory_min_length` chars are ignored
- **No raw transcripts**: only the last AI response text is stored, truncated to 500 chars

Toggle it in **Settings → External → Kurultai → Passive auto-memory**.

## Brain panel (WebUI)

A **Brain** button (🧠 psychology icon) appears in the sidebar. Click it to open
the Kurultai brain panel as a right-canvas surface:

- **Search** — hybrid/FTS search with ranked excerpts and citations
- **Ask** — synthesized cited answer across atoms
- **Status badge** — green (online) / red (offline) daemon health indicator
- **Optimized for speed** — health-check cached for 5s, no lag on panel open

The panel calls `/api/plugins/kurultai/brain` (GET for status, POST for search/ask).

## Security

- **Daemon URL validation**: the plugin rejects server URLs pointing to public/internet addresses (localhost and private network only)
- **API key isolation**: `OPENROUTER_API_KEY` is never read, stored, or echoed by the plugin — it flows through the inherited environment to the kurultai binary
- **Content sanitization**: auto-memory strips secret-like patterns before storing

## Layout

```
plugin.yaml           manifest (name: kurultai, settings section: external)
default_config.yaml   default settings + security note about the API key
helpers/config.py     settings resolution + subprocess env (no key handling)
helpers/client.py     CLI subprocess + HTTP daemon transport (stdlib only)
tools/                kurultai_search / _ask / _who_knows / _remember / _status
extensions/python/monologue_end/  passive auto-memory hook
extensions/webui/              sidebar button + right-canvas brain panel
webui/brain-store.js  Alpine store for brain panel (search/ask/status)
webui/brain-panel.html  brain viewer UI
webui/config.html     settings modal fields (Settings → External)
api/brain.py          brain panel API handler (search/ask/status)
helpers/security.py   localhost-only URL validation + content sanitization
README.md             this file
```

License: MIT (same as kurultai).
