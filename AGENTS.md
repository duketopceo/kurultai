# AGENTS.md

## Learned User Preferences

- Do not commit `.cursor/` contents.
- Before changing Kurultai Brain/dashboard visuals, ask first — the current design is largely liked.
- Keep the dashboard layout with the brain as the main focal point; do not replace it with a full-viewport-only redesign unless asked.
- Brain synaptic visualization: deep black background; black/white plus slight purple only (three colors); neurons/synapses with electric zap/shimmer (not plain white orbs); hovering a node highlights its connections; avoid extra chrome (control buttons and live/suggested/showcase MCP callouts).
- Brain camera should start with the whole graph in view (no opening live zoom-in); search needs a clear/reset control.
- Intended Brain layouts: volumetric **brain-shape** (constrained FDG inside the cortex hull) and **algorithmic ontology** (typed hierarchy). Galaxy/solar is deprecated. Do not ship an ontology layout until O1 primitives exist. Research: `docs/brainstorms/2026-08-13---brain-shape-algorithmic-ontology.md`. Plan: `docs/plans/2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md`.
- Version/tag before risky Brain UI experiments so rollback is easy.
- Hosted Brain human Access should be password-manager-friendly (e.g. 1Password save/autofill login), not a bare API-token paste gate as the primary UX.
- Below-brain chrome uses a secondary playful design language (ThreeUI-inspired CSS/motion only); do not vendor ThreeUI into BrainStage; literal ontology stays in cortex + inspector only (“UI fun, not literal”).
- Sequester pond/session/transcript noise out of hot retrieval tiers (medium/cold) so dogfood search stays usable.
- Prefer Kurultai personal knowledge MCP (`knowledge.shippedit.dev`) as the knowledge backend under test; Dayflow is a local plugin, not a Pinecone substitute.
- Agent identity is two-layer: **codename** = product family (one key per product per lane); **instance_id** = concurrent seat. Never register `devin-2` / `cursor-2` as separate Codenames — use `instance_id` on Hey posts. Canonical doctrine: `duketopceo/luke-agents` AGENTS.md §0a.

## Learned Workspace Facts

- Start at [`INDEX.md`](INDEX.md) (folder map). Schema and update ritual: [`docs/agent-index.md`](docs/agent-index.md). When you change a file, update that folder's `INDEX.md` row (does/needs/touches/stamp/version/3-line changelog) and prepend Recent up to the root index. `python3 scripts/audit-agent-index.py` must stay green.
- Brain UI assets live in `ui/` and are embedded in the daemon binary; with the daemon running, open `http://127.0.0.1:8421/ui/` — do not maintain a parallel brain dashboard under `website/` or `web/`.
- Local Vite preview commonly runs from `website/` on `http://127.0.0.1:5174` (`npm run dev -- --host 127.0.0.1 --port 5174`) and can serve live `ui/` files; embedded daemon UI changes need a rebuild.
- Default local daemon: `./target/debug/kurultai daemon --port 8421` from the repo root.
- Large brain graphs should use tiered loading (hot/warm/cold, timestamped) rather than shipping all nodes to the browser at once.
- `docs/solutions/` — documented solutions to past problems (bugs, architecture, workflow), organized by category with YAML frontmatter (`module`, `tags`, `problem_type`). Relevant when implementing or debugging in documented areas.
- `CONCEPTS.md` — shared domain vocabulary. Read when orienting to the codebase or before discussing domain concepts.
- Hosted solo instances: `knowledge.shippedit.dev` (personal, instance name **Ulaanbaatar**) and `work.shippedit.dev` (work — not dogfooded yet); deploy recipes live in `duketopceo/kurultai-private` under `deploy/server-001/` (Docker solo + Cloudflare tunnel), not the public repo’s Railway hub path.
- Cluster2’s old Ulaanbaatar daemon/store is wiped; **Ulaanbaatar** now names the personal knowledge *instance*, not an agent codename. Do not redeploy kurultai on c2.
- Brain Repos strip reflects local `kind=github` checkouts; product intent includes `duketopceo/repos` and deploy→reindex of those repos. Hey `repo` + `instance_id` claims surface under repo cards via `GET /api/hey/presence`.
- Agent-to-agent messaging board is the Hey surface (MCP `hey_*` tools and dashboard Hey panel). Hot/medium/cold are retrieval tiers — not the board.
- Personal lane agents to keep registered: `cursor`, `claude`, `codex`, `antigravity`, `hermes` (plus `devin` when wired). **Ulaanbaatar** is the instance nickname for knowledge.shippedit.dev — not an agent codename.

## Cursor Cloud specific instructions

These notes are for future cloud agents. The VM startup **update script** already
refreshes dependencies (`cargo fetch` + `npm install` for `web/` and `website/`),
so this section only covers non-obvious run/test caveats. Standard commands live in
`CONTRIBUTING.md`, `README.md`, and each `package.json`.

### CodeRabbit / reviews

- Repo `.coderabbit.yaml` keeps **auto-review off** — do not ping `@coderabbitai review` unless the user asks.
- Prefer CI green + existing approvals; **do not wait on CodeRabbit** to merge.
- Keep `@coderabbitai ignore` in PR descriptions (PR template already includes it).

### Surfaces

| Surface | Path | Run (dev) | Notes |
|---------|------|-----------|-------|
| Core CLI + daemon (main product) | `src/` (Rust) | `cargo run -- <cmd>` or `./target/debug/kurultai <cmd>` | SQLite knowledge brain: `init`, `index`, `search`, `ask`, `who-knows`, `status`, `mcp`, `daemon` |
| Brain UI (embedded in daemon) | `ui/` (source in `website/`) | served at `GET /ui/` when daemon runs; `website/` = `npm run dev` (Vite) | Single dashboard surface — do not add a parallel one |
| Team web app | `web/` (Next.js + Clerk) | `npm run dev` → http://localhost:3000 | Auth flow needs Clerk keys (see below) |

### Core CLI / daemon caveats

- Set `KURULTAI_ENV=dev` for dev work (storage under `~/.local/share/kurultai/dev/store.db`, debug logging).
- `kurultai` needs a config first: `kurultai init --docs` writes `~/.config/kurultai/config.toml`, creates `Documents/kurultai` (or `~/kurultai`), and enables `[sources.notes]`. `--agent none` skips MCP; `--index` indexes immediately. Plain `kurultai init` still writes config only.
- **FTS-only mode is normal here**: without `OPENROUTER_API_KEY` (or `KURULTAI_API_KEY`) the app uses `NullEmbedder` — FTS5 search / `who-knows` / extractive `ask` all work; vector recall, reranking, and LLM `ask` are disabled. This is expected, not an error.
- Markdown atoms need **≥1 tag** (YAML frontmatter `tags:` **or** a dedicated hashtag line such as `#vpn #snipe-it`) or they land in quarantine and are excluded from default search. YAML tags win when present. Headings and inline `#mentions` in prose are not tags.
- Run the daemon: `kurultai daemon --port 8421`. Open the Brain UI at **`http://127.0.0.1:8421/ui/`** — the trailing slash matters (bare `/ui` returns a 308 redirect). API lives under `/api/*` (`/api/status`, `/api/atoms`, `/api/search`, ...).

## Wiring Kurultai (MCP + Data Flow)

Kurultai is the knowledge brain. All autonomous agents (Cursor, Claude, Codex, Hermes) must wire into it via MCP, and all cron-generated data must flow upstream into it.

### 1. The Upstream Data Rule
**For any cron that stores data, its data must flow upstream to Kurultai** (labeled as needed).
- Downstream domain stores (Luke-Vault, NN data, Notion, SQLite databases) are allowed and expected.
- However, all data must match the downstream store and ultimately land in Kurultai so it is globally searchable.
- Build connectors in `src/connectors/` (e.g., `tech_tracker`, `json`) to bridge isolated stores into the brain.

### 2. MCP Agent Wiring
Agents discover and connect to the Kurultai knowledge graph using the Model Context Protocol (MCP). Kurultai provides 8 tools: `search`, `cite`, `remember`, `ask`, `who_knows`, `promote`, `ontology_get`, and `ontology_promote`.

To wire an agent on a new machine or environment:
1. Build/install the binary (`cargo build --release` → `~/.cargo/bin/kurultai`)
2. Run **`kurultai init --agent <name>`** (valid targets: `cursor`, `claude`, `codex`, `hermes`, or `all`).
3. This automatically updates `~/.cursor/mcp.json`, `~/.claude.json`, `~/.codex/config.toml`, or `~/.hermes/config.yaml`. `--agent none` skips MCP. Combine with `--docs` to provision a local markdown folder.
4. Restart the agent(s) to load the MCP server.

### Egress caveat (Brain UI 3D graph)

The Brain UI loads `three.js` + `3d-force-graph` from `unpkg.com`. In the restricted
cloud network `unpkg.com` is blocked, so the "3D Synaptic Network" canvas stays blank.
The rest of the UI (stat cards, Atom Explorer, document detail) works — it reads from
the local `/api` endpoints. Allowlist `unpkg.com` if the 3D graph is needed.

### web/ (Clerk) blocker

`web/` requires Clerk secrets: `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` and `CLERK_SECRET_KEY`
(see `web/.env.example`). The dev server boots and the homepage renders, but protected
routes (e.g. `/dashboard`) return **500 "Missing publishableKey"** until the keys are set.

### Lint / test / build (Rust)

See `CONTRIBUTING.md`. Quick reference: `cargo fmt --all -- --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test --locked`,
`cargo build --release --locked`. CI (`.github/workflows/ci.yml`) uses `cargo nextest`
plus `cargo audit`; plain `cargo test` is fine locally. Note CI sets `RUSTFLAGS=-Dwarnings`.
