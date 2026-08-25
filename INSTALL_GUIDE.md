# Kurultai Install Guide (macOS)

**Version:** v0.4.1
**Platform:** macOS (Apple Silicon arm64 / Intel x86_64)

Kurultai is a local knowledge brain. It indexes notes, chats, JSON dumps, and code into one SQLite store, then exposes search/ask/MCP with excerpts and citations. The "desktop app" is a CLI binary + local HTTP daemon serving a browser-based Brain UI.

---

## Option A — One-line install (recommended)

Prefers a prebuilt GitHub Release binary; falls back to `cargo install`.

```bash
curl -fsSL https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.sh | bash
```

This installs `kurultai` to `~/.local/bin/kurultai`. Add it to your PATH (add to `~/.zshrc`):

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
kurultai --version    # → kurultai 0.4.1
```

## Option B — Install via cargo

```bash
# Install Rust if you don't have it:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

cargo install --git https://github.com/duketopceo/kurultai --tag v0.4.1 --locked
# Binary lands at ~/.cargo/bin/kurultai (already on PATH after rustup)
kurultai --version
```

## Option C — Build from source

```bash
git clone https://github.com/duketopceo/kurultai.git
cd kurultai
cargo build --release          # ~4-5 min clean build, 0 warnings
./target/release/kurultai --version    # → kurultai 0.4.1
```

Optional: copy the binary somewhere on your PATH:

```bash
cp target/release/kurultai /usr/local/bin/   # or ~/.local/bin/
```

---

## Setup (one time)

### 1. OpenRouter (recommended — full brain)

Kurultai's normal setup uses **one OpenRouter API key** for the full retrieval stack:

```bash
export OPENROUTER_API_KEY=sk-or-...    # or KURULTAI_API_KEY
```

| Role | OpenRouter model | Where |
|------|------------------|-------|
| **Embeddings** | `openai/text-embedding-3-large` (3072-d) | `[embed]` in config (default) |
| **Rerank** | `openai/gpt-4o-mini` | `[runtime] reranker_model` (uncomment in config) |
| **LLM `ask`** | `openai/gpt-4o-mini` | automatic when the key is set |

Add the export to `~/.zshrc` (or your shell profile) so index, search, daemon, and MCP all see it. Keys live in the environment only — never in `config.toml`.

> **FTS-only fallback:** without a key, Kurultai stays in FTS-only mode — FTS5 search, `who-knows`, and extractive `ask` all work. Vector recall, reranking, and LLM `ask` need the key above (or local ONNX below).

### 2. Initialize config + docs folder + MCP wiring

```bash
# Dev environment (storage under ~/.local/share/kurultai/dev/):
export KURULTAI_ENV=dev

# Create config + a markdown docs folder + wire MCP for all agents:
kurultai init --docs --agent all
```

What this does:
- Writes `~/.config/kurultai/config.toml`
- Creates `~/Documents/kurultai/` with a tagged `welcome.md` starter note
- Adds a `[sources.notes]` section pointing at that folder
- Writes MCP configs for Cursor (`~/.cursor/mcp.json`), Claude (`~/.claude.json`), Codex (`~/.codex/config.toml`), and Hermes (`~/.hermes/config.yaml`)

> **Note:** `init` does NOT create the database directory — that happens lazily on first `index`/`daemon`/`mcp` run. This is expected.

Variants:
```bash
kurultai init                         # config only, wire Cursor MCP (default)
kurultai init --docs                  # config + docs folder + Cursor MCP
kurultai init --docs --agent none     # config + docs, skip MCP
kurultai init --docs --agent cursor   # wire only Cursor
kurultai init --docs ~/Notes          # use a custom docs folder
kurultai init --docs --index          # also run a full index immediately
```

### 3. (Optional) Local ONNX embeddings — offline alternative

Skip OpenRouter for embeddings only (rerank / LLM `ask` still need the key unless you stay extractive):

```bash
# Build with the local-embed feature:
cargo build --release --features local-embed
# Then in ~/.config/kurultai/config.toml [embed] section, set:
#   backend = "local"
#   model = "AllMiniLML6V2"
#   dimension = 384
```

---

## Daily use

### Index your notes

```bash
kurultai index --full      # full re-index (FTS works with no API key)
kurultai index             # incremental (only changed files)
```

> Markdown atoms need ≥1 tag (YAML frontmatter `tags: [...]` or a hashtag line like `#vpn #snipe-it`). Untagged or low-quality atoms land in quarantine and are skipped by default search. Promote after fixing: `kurultai promote <atom_id>`.

### Search and ask

```bash
kurultai search "welcome" --limit 10
kurultai ask "what is in my notes?"
kurultai who-knows "kubernetes"        # which sources know about a topic
kurultai status                        # env, sources, atom counts, feature flags
```

### Run the daemon (HTTP API + Brain UI)

```bash
kurultai daemon --port 8421
```

Then open the Brain UI in your browser:

```
http://127.0.0.1:8421/ui/      ← trailing slash matters
```

The Brain UI is a React + Three.js dashboard embedded in the binary. It shows atom stats, an Atom Explorer, and a 3D synaptic network graph (loads three.js + 3d-force-graph from `unpkg.com` — allowlist that domain if the graph canvas is blank).

Useful daemon flags:
```bash
kurultai daemon --port 8421 --no-poll        # disable background incremental indexing
kurultai daemon --port 8421 --no-watch       # disable filesystem watch
kurultai daemon --port 8421 --poll-interval 60   # override poll interval (seconds)
```

### MCP server (for agents)

```bash
kurultai mcp       # stdio MCP server — Cursor/Claude/Codex/Hermes connect to this
```

Kurultai provides 8 MCP tools: `search`, `cite`, `remember`, `ask`, `who_knows`, `promote`, `ontology_get`, `ontology_promote`.

Optional remote MCP (read-only) on the daemon:
```bash
export KURULTAI_MCP_HTTP_SECRET='your-secret'
kurultai daemon --port 8421
# Then: POST http://127.0.0.1:8421/mcp  (Authorization: Bearer your-secret)
#       GET  http://127.0.0.1:8421/mcp/sse
```

### Loopback ingest (push atoms via HTTP)

```bash
export KURULTAI_INGEST_SECRET='your-secret'
kurultai daemon --port 8421

# Push a markdown atom:
curl -X POST http://127.0.0.1:8421/ingest \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-secret" \
  -d '{"title":"My Note","content":"...substantial content...","tags":["notes"]}'
```

### Diagnostics

```bash
kurultai doctor      # checks config, database, embeddings, ontology, MCP, HTTP, connectors
```

### Multi-device packs

```bash
kurultai export -o brain.kurultai                    # pack this brain
# ...copy brain.kurultai to the new device...
kurultai import brain.kurultai                        # restore (empty store)
kurultai import brain.kurultai --combine              # merge into existing store
```

---

## Rebuilding the Brain UI (developers)

The Brain UI source lives in `website/` (React + Three.js, Vite). Built assets go to `ui/` and are embedded into the daemon binary at compile time.

```bash
bash scripts/build-ui.sh      # runs npm ci + tsc --noEmit + vite build → ui/
cargo build --release         # re-embed the new ui/ assets into the binary
```

Requirements: Node 20+ (`.nvmrc` says 22). The build outputs `ui/assets/brain-*.js`, `brain-*.css`, and `brain-*.glb`.

---

## Troubleshooting

### `kurultai: command not found`
- One-line installer: add `~/.local/bin` to PATH: `echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc`
- cargo install: ensure `~/.cargo/bin` is on PATH (rustup adds it automatically)

### `config file not found, using defaults`
- Run `kurultai init` first. Or set `KURULTAI_CONFIG=/path/to/config.toml`.

### Daemon won't start / port already in use
- Check for a stale process: `lsof -i :8421` then `kill <pid>`
- Use a different port: `kurultai daemon --port 8422`

### Brain UI is blank / 3D graph doesn't render
- The 3D synaptic network loads `three.js` + `3d-force-graph` from `unpkg.com`. If that domain is blocked (firewall, corporate network), the graph canvas stays blank but the rest of the UI works.
- Fix: allowlist `unpkg.com`, or run a local Vite preview: `cd website && npm install && npm run dev` (serves on http://127.0.0.1:5174 with `/api` proxied to the daemon).

### `/ingest` returns 404
- Loopback ingest is disabled by default. Set `KURULTAI_INGEST_SECRET` env var before starting the daemon, and send `Authorization: Bearer <secret>` header.

### Atoms go to quarantine
- Atoms need ≥1 tag (YAML frontmatter `tags: [...]` or a hashtag line). Short/thin atoms also quarantine (`low_quality:too_short` / `low_quality:thin`).
- Promote after fixing: `kurultai promote <atom_id>`
- Check status: `kurultai status` shows `quarantine_count`.

### `no OPENROUTER_API_KEY — FTS-only mode`
- This is **expected** without an API key. FTS5 search, `who-knows`, and extractive `ask` all work. Vector recall, reranking, and LLM `ask` need `OPENROUTER_API_KEY` (or `KURULTAI_API_KEY`), or local ONNX (`--features local-embed`).

### Slow first daemon startup
- First startup runs DB migrations (schema 0→9). On a fresh DB this takes a few seconds; subsequent starts are instant. If it's very slow, check disk I/O.

### Reset everything
```bash
rm -rf ~/.local/share/kurultai/        # delete the database
rm -f ~/.config/kurultai/config.toml   # delete config (re-run init)
```

---

## File locations (macOS)

| What | Path |
|------|------|
| Binary (cargo install) | `~/.cargo/bin/kurultai` |
| Binary (one-line installer) | `~/.local/bin/kurultai` |
| Config | `~/.config/kurultai/config.toml` |
| Database | `~/.local/share/kurultai/dev/store.db` (dev) / `.../prod/store.db` (prod) |
| Docs folder | `~/Documents/kurultai/` |
| Cursor MCP | `~/.cursor/mcp.json` |
| Claude MCP | `~/.claude.json` |
| Codex MCP | `~/.codex/config.toml` |
| Hermes MCP | `~/.hermes/config.yaml` |

---

## Quick reference

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.sh | bash
export PATH="$HOME/.local/bin:$PATH"

# Setup
export KURULTAI_ENV=dev
export OPENROUTER_API_KEY=sk-or-...   # or KURULTAI_API_KEY — full brain
kurultai init --docs --agent all
kurultai index --full

# Use
kurultai search "welcome"
kurultai daemon --port 8421        # → open http://127.0.0.1:8421/ui/
kurultai mcp                       # stdio MCP for agents
kurultai doctor                     # diagnostics
```
