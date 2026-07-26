# AGENTS.md

## Learned User Preferences

- Do not commit `.cursor/` contents.
- Before changing Kurultai Brain/dashboard visuals, ask first — the current design is largely liked.
- Brain synaptic visualization: prefer a deep black background with white/electric nodes; hovering a node should highlight its connections; avoid extra chrome (control buttons and live/suggested/showcase MCP callouts).

## Learned Workspace Facts

- Brain UI assets live in `ui/` and are embedded in the daemon binary; with the daemon running, open `http://127.0.0.1:8421/ui` — do not maintain a parallel brain dashboard under `website/` or `web/`.
- Local Vite preview commonly runs from `website/` on `http://127.0.0.1:5174` (`npm run dev -- --host 127.0.0.1 --port 5174`) and can serve live `ui/` files; embedded daemon UI changes need a rebuild.
- Default local daemon: `./target/debug/kurultai daemon --port 8421` from the repo root.

## Cursor Cloud specific instructions

These notes are for future cloud agents. The VM startup **update script** already
refreshes dependencies (`cargo fetch` + `npm install` for `web/` and `website/`),
so this section only covers non-obvious run/test caveats. Standard commands live in
`CONTRIBUTING.md`, `README.md`, and each `package.json`.

### Surfaces

| Surface | Path | Run (dev) | Notes |
|---------|------|-----------|-------|
| Core CLI + daemon (main product) | `src/` (Rust) | `cargo run -- <cmd>` or `./target/debug/kurultai <cmd>` | SQLite knowledge brain: `init`, `index`, `search`, `ask`, `who-knows`, `status`, `mcp`, `daemon` |
| Brain UI (embedded in daemon) | `ui/` (source in `website/`) | served at `GET /ui/` when daemon runs; `website/` = `npm run dev` (Vite) | Single dashboard surface — do not add a parallel one |
| Team web app | `web/` (Next.js + Clerk) | `npm run dev` → http://localhost:3000 | Auth flow needs Clerk keys (see below) |

### Core CLI / daemon caveats

- Set `KURULTAI_ENV=dev` for dev work (storage under `~/.local/share/kurultai/dev/store.db`, debug logging).
- `kurultai` needs a config first: `kurultai init` writes `~/.config/kurultai/config.toml`. Enable a source (e.g. a `[sources.notes]` markdown root) before `index`.
- **FTS-only mode is normal here**: without `OPENROUTER_API_KEY` (or `KURULTAI_API_KEY`) the app uses `NullEmbedder` — FTS5 search / `who-knows` / extractive `ask` all work; vector recall, reranking, and LLM `ask` are disabled. This is expected, not an error.
- Markdown atoms need **≥1 tag** (YAML frontmatter `tags:`) or they land in quarantine and are excluded from default search.
- Run the daemon: `kurultai daemon --port 8421`. Open the Brain UI at **`http://127.0.0.1:8421/ui/`** — the trailing slash matters (bare `/ui` returns a 308 redirect). API lives under `/api/*` (`/api/status`, `/api/atoms`, `/api/search`, ...).

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
