# AGENTS.md

## Learned User Preferences

- Do not commit `.cursor/` contents.
- Before changing Kurultai Brain/dashboard visuals, ask first — the current design is largely liked.
- Keep the dashboard layout with the brain as the main focal point; do not replace it with a full-viewport-only redesign unless asked.
- Brain synaptic visualization: deep black background; black/white plus slight purple only (three colors); neurons/synapses with electric zap/shimmer (not plain white orbs); hovering a node highlights its connections; avoid extra chrome (control buttons and live/suggested/showcase MCP callouts).
- Brain camera should start with the whole graph in view (no opening live zoom-in); search needs a clear/reset control.
- Intended Brain layouts: volumetric **brain-shape** (constrained FDG inside the cortex hull) and **algorithmic ontology** (typed hierarchy). Galaxy/solar is deprecated. Do not ship an ontology layout until O1 primitives exist. Research: `docs/brainstorms/2026-08-13---brain-shape-algorithmic-ontology.md`. Plan: `docs/plans/2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md`.
- Version/tag before risky Brain UI experiments so rollback is easy.

## Learned Workspace Facts

- Start at [`INDEX.md`](INDEX.md) (folder map). Schema and update ritual: [`docs/agent-index.md`](docs/agent-index.md). When you change a file, update that folder's `INDEX.md` row (does/needs/touches/stamp/version/3-line changelog) and prepend Recent up to the root index. `python3 scripts/audit-agent-index.py` must stay green.
- Brain UI assets live in `ui/` and are embedded in the daemon binary; with the daemon running, open `http://127.0.0.1:8421/ui/` — do not maintain a parallel brain dashboard under `website/` or `web/`.
- Local Vite preview commonly runs from `website/` on `http://127.0.0.1:5174` (`npm run dev -- --host 127.0.0.1 --port 5174`) and can serve live `ui/` files; embedded daemon UI changes need a rebuild.
- Default local daemon: `./target/debug/kurultai daemon --port 8421` from the repo root.
- Large brain graphs should use tiered loading (hot/warm/cold, timestamped) rather than shipping all nodes to the browser at once.
- `docs/solutions/` — documented solutions to past problems (bugs, architecture, workflow), organized by category with YAML frontmatter (`module`, `tags`, `problem_type`). Relevant when implementing or debugging in documented areas.
- `CONCEPTS.md` — shared domain vocabulary. Read when orienting to the codebase or before discussing domain concepts.

## Shared crew instance on node3 — where this actually runs day-to-day

This repo's own [`docs/crew-instance-node3.md`](docs/crew-instance-node3.md)
is the maintained, code-verified runbook for the real deployment pattern
here: N Claude Code sessions, one unix user, one box, sharing one SQLite
store as a company knowledge brain. It's more current and more thorough than
any summary that could be pasted here, so read it directly rather than a
copy — key points worth knowing before you open it:

- **No daemon runs for this use case.** `kurultai mcp` is pure stdio
  JSON-RPC, opens the store directly, zero network. `kurultai daemon` always
  binds an HTTP port (`--no-poll`/`--no-watch` only disable background
  loops, not the listener) — so the runbook's recommendation is a systemd
  *timer* running one-shot `kurultai index`, not a long-lived daemon.
- **One SQLite file, WAL-mode**, `~/.local/share/kurultai/store.db`,
  configured for multi-process use by `SqliteVecStore::open`
  (`journal_mode=WAL`, `busy_timeout`, `synchronous=NORMAL` —
  `src/store/mod.rs`'s `configure_multiprocess`). Never put the store or a
  build dir on `/tmp` (tmpfs on this box — RAM, not disk).
- **What's actually configured on this box right now differs from the
  runbook's own recommendation** — verified directly while writing this
  pass, not assumed: `~/.config/kurultai/config.toml` has `environment =
  "dev"` (runbook recommends `"prod"` for the crew deployment), storage
  under `~/.local/share/kurultai/dev/store.db`, and there is **no**
  `kurultai-index.timer` systemd unit installed (`systemctl --user
  list-timers` shows none) and **no** `kurultai` entry in `~/.claude.json`'s
  `mcpServers` (only `railway` is wired there). In practice the shared
  instance is driven by direct CLI (`kurultai search "..."`, `kurultai index
  --full`, `kurultai daemon --port 8421` run ad hoc for the Brain UI per
  `~/dev/CLAUDE.md`'s crew doc), not the MCP-wired, timer-driven setup this
  repo's own runbook describes. Confirmed live and populated: `kurultai
  status` reports 2079 atoms (1989 trusted / 90 quarantine), FTS-only (no
  `OPENROUTER_API_KEY` set), schema v10 — matches this repo's `main` exactly.
- Project scoping (`KURULTAI_PROJECT` / the `recall` MCP tool) and the
  `shared_write` containment flag — both real, both covered under "Wiring
  Kurultai" below — are the two mechanisms this runbook's own "Not yet
  done" section originally called for. Both landed in code the same day the
  runbook was last touched; that section of the runbook is now stale on
  that point specifically (confirmed by commit date comparison), current on
  everything else it says.

## Architecture — from-scratch orientation

Verified against `src/lib.rs` and the modules themselves (current `main` HEAD,
crate `0.4.1`, schema `v10`). If you only read one thing before touching code,
read this section, then jump into the specific module.

### Data model (see `CONCEPTS.md` for the glossary; this is the storage shape)

Everything is a `KnowledgeAtom` in the `knowledge_atoms` SQLite table
(`src/store/migrations.rs`, migration 001 + later `ALTER TABLE`s):
`id, source, source_id, title, summary, content, question, resolution,
tags_json, source_updated_at, indexed_at, content_hash, metadata_json,
trust_lane, quarantine_reason, last_accessed_at, visibility, corpus_tier,
visibility_labels_json`. `project_id` is **not** a column — it lives inside
`metadata_json` (see Project scoping below).

- **Trust lane** (`trusted` / `quarantine`): default retrieval (`search`,
  `ask`, `who_knows`) only sees `trusted` atoms. An atom quarantines at
  ingest if it's untagged, too short, or looks like boilerplate
  (`src/quality/gate.rs`). Only `kurultai promote` (or the `promote` /
  `ontology_promote` MCP tools, gated by write-policy — see below) moves an
  atom to `trusted`.
- **Visibility scope** (`personal` / `team` / `company`): default `personal`.
  Only matters once a shared hub is configured (`hub` feature, off by
  default — see `src/features.rs` / the Roadmap section below); solo
  installs never see `team`/`company` atoms filtered differently.
- **Corpus tier** (`public` / `private`) and **visibility labels** (free-text
  tags like `it`/`finance`): a second, orthogonal access dimension, set at
  ingest via `SourceConfig::default_corpus_tier` /
  `default_visibility_labels`. Unknown/missing values fail closed to
  `private`.
- **Ontology** (`ontology_entities` / `ontology_links` tables, migration
  009): a small labeled-property-graph layer *beside* atoms — classes
  (`Memory`, `Note`, `Code`, `Decision`, `Person`, `System`, seeded on
  migration), instances (`ent:{atom_id}`), and typed links (`is_a`,
  `instance_of`, `associates_with`, `triggered_by`, `contradicts`; only
  `approved` links are read). `ontology_promote` creates an instance entity
  from an atom — it does **not** touch the atom's trust lane.
- Other real tables: `store_meta` (schema version + book-keeping),
  `quality_audit`, `merge_candidates` (near-dupe merge queue),
  `ingestion_jobs` (per-file staging status for connector runs),
  `label_vocab` / `atom_soft_labels` (soft, scored labels distinct from
  hard `tags_json`).

### Module map (`src/`, 32 top-level modules per `src/lib.rs`)

`CONTRIBUTING.md`'s "Module Boundaries" list (`brain`, `connectors`, `mcp`,
`query`, `app`) is real but only names 5 of 32 — treat it as a starting
sketch, not the full map. The rest, grouped by what they do:

| Group | Modules | What lives there |
|---|---|---|
| Entry points | `main.rs` (clap `Commands` enum — see CLI below), `lib.rs` | CLI parsing, subcommand dispatch |
| App wiring | `app/` (`context.rs` builds embedder/reranker/store from config; `mod.rs` orchestrates a run) | Turns `Config` into live components |
| Storage | `store/` (`mod.rs` = `Store` trait + `SqliteVecStore`; `migrations.rs` = versioned schema, `CURRENT_SCHEMA_VERSION = 10`; `postgres.rs` = HUB-2 `PostgresStore`, behind `--features postgres` + `KURULTAI_FEATURE_HUB=1`) | FTS5 + `sqlite-vec` kNN + all persistence |
| Ingestion | `connectors/` (`markdown`, `json`, `github`, `dayflow`, `inbox`, `appflowy`, `pond`, plus `registry.rs` factory + the `Connector` trait), `ingest/dump.rs` (format detection: `.md`/`.markdown`/`.json`/`.jsonl`/`.ndjson`/`.txt`), `pipeline/` (ties a connector's atoms through embed → quality gate → store) | Getting external data into atoms |
| Retrieval | `query/` (`hybrid.rs` = FTS+vector fusion, `rrf.rs` = the RRF math, `context.rs` = neighbor-chunk expansion for markdown), `embed/` (`mod.rs` = OpenRouter cloud embedder + `NullEmbedder`; `local.rs` = optional ONNX via `fastembed`, cargo feature `local-embed`), `rerank/` (cloud reranker + `NullReranker`), `synthesize/` (extractive `ask`/`who_knows`, LLM synthesis when a key is present) | Turning a query into ranked, citeable results |
| Quality | `quality/` (`gate.rs` = trust-lane admission, `promote.rs`, `merge.rs` + `near_dupe.rs` = dedup, project-aware since `project_id`) | What gets trusted, merged, or quarantined |
| Ontology | `ontology/` | O1 entities/links CRUD backing `ontology_get`/`ontology_promote` |
| Memory tiering | `memory/` (`tier.rs` = hot/warm/cold classification for the Brain UI graph, `GraphNode`, `MemoryTier`, `TierPolicy`) | Backs the "tiered loading" workspace fact above |
| Interfaces | `mcp/` (`server.rs` = stdio JSON-RPC tool dispatch, `brain.rs` = `BrainService` the tools call into, `init.rs` = `kurultai init --agent`, `interface.rs`), `http/` (`mod.rs` = axum router, `auth.rs` = hub API-key gate, `ingest.rs` = `POST /ingest` loopback adapter, `ui.rs` = embedded `ui/` serving, `mcp.rs` = the daemon's HTTP/SSE MCP surface, read-only) | How agents and the Brain UI actually talk to the store |
| Daemon | `daemon/` | `kurultai daemon`'s poll/watch loop + HTTP server lifecycle |
| Export/import | `export/` | `.kurultai` pack format (backup/restore, multi-device handoff) |
| Cross-cutting | `security/` (`paths.rs` = path traversal guards on connector roots, `redact.rs`, `secrets.rs`, `admin_keys.rs` = HUB API-token minting), `write_policy.rs` (shared-write containment, see Crew instance below), `project.rs` (project-namespace resolution/normalization), `config/`, `features.rs` (versioned flag catalog), `types.rs`, `error.rs`, `activity.rs`, `metrics.rs`, `logging.rs`, `hashutil.rs` (content-hash for incremental skip), `doctor.rs`, `art.rs` (the Yurt banner), `environment.rs` | Config, safety, observability, plumbing |

### CLI commands (`src/main.rs`, `Commands` enum — verified exhaustively)

`init` (`--agent`, `--docs`, `--index`) · `index` (`--full`) · `search
<query>` · `ask <question>` · `who-knows`/`who_knows <topic>` · `status`
(`--metrics`) · `promote <atom_id>` (`--reason`) · `mcp` (stdio server) ·
`daemon` (`--port`, `--no-poll`, `--poll-interval`, `--no-watch`) · `export`
(`-o`) · `import <pack>` (`--force`, `--combine`, `--write-config`) · `prune`
(`--generated`, Next.js/webpack path-segment cleanup) · `admin key
issue/revoke/list` (mints/revokes scoped HUB API tokens, `src/security/admin_keys.rs`,
new since the last AGENTS.md pass — landed with #223).

### End-to-end flow

`index`/`daemon` poll → `ConnectorRegistry` (`connectors/registry.rs`) builds
one connector per enabled `[sources.*]` entry, `init()`s it, calls
`poll()`/`full_sync()` → atoms flow through `pipeline/` (content-hash
skip if unchanged and a vector already exists — "hash-skip" in
`CONCEPTS.md` — then embed if a key is configured, else `NullEmbedder`) →
`quality/gate.rs` decides `trusted` vs `quarantine` → `Store::upsert_batch`.
Reading: `search`/`ask`/`who_knows`/`recall` → `query/hybrid.rs` runs FTS5
and (if embeddings exist) vector kNN in parallel, fuses with RRF
(`query/rrf.rs`), reranks if a reranker is configured, else keeps RRF order
→ `synthesize/` turns hits into an `AgentAtomView` (excerpt-first, never
full content unless asked) for `search`/`cite`, or an extractive/LLM answer
for `ask`.

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

Kurultai is the knowledge brain. The aspiration (this section, historically)
is that all autonomous agents wire into it via MCP and all cron-generated
data flows upstream into it. What's verified true today, split from what's
still aspirational:

### 1. The Upstream Data Rule — aspirational, not enforced anywhere in code
**For any cron that stores data, its data must flow upstream to Kurultai** (labeled as needed).
- Downstream domain stores (Luke-Vault, NN data, Notion, SQLite databases) are allowed and expected.
- However, all data must match the downstream store and ultimately land in Kurultai so it is globally searchable.
- Build connectors in `src/connectors/` (e.g., `json`, `github`) to bridge isolated stores into the brain. Note: a `tech_tracker` connector does **not** exist in `src/connectors/` — the real list, verified against `registry.rs`, is `markdown`, `json`, `github`, `dayflow`, `inbox`, `appflowy`, `pond`.
- This is a policy statement for humans/agents deciding what to build next, not something the code checks or enforces.

### 2. MCP Agent Wiring — the `init --agent` machinery is real and fully implemented
Agents discover and connect to the Kurultai knowledge graph using the Model Context Protocol (MCP). Kurultai provides **9** tools (verified against `src/mcp/server.rs`'s `TOOL_*` constants and its own tool-list test): `search`, `cite`, `remember`, `ask`, `who_knows`, `promote`, `ontology_get`, `ontology_promote`, and **`recall`** (added since the last pass on this doc — project-scoped read, see Project scoping below; the earlier "8 tools" count here was stale).

To wire an agent on a new machine or environment:
1. Build/install the binary (`cargo build --release` → `~/.cargo/bin/kurultai`)
2. Run **`kurultai init --agent <name>`** (valid targets: `cursor`, `claude`, `codex`, `hermes`, or `all`).
3. This automatically updates `~/.cursor/mcp.json`, `~/.claude.json`, `~/.codex/config.toml`, or `~/.hermes/config.yaml`. `--agent none` skips MCP. Combine with `--docs` to provision a local markdown folder.
4. Restart the agent(s) to load the MCP server.

**Verified in `src/mcp/init.rs`: all four targets (`cursor`/`claude`/`codex`/`hermes`) plus `all` are actually implemented**, not stubbed — each has a real `wire_*_at` function that writes a real config file (JSON for cursor/claude, TOML for codex, YAML for hermes), with tests covering each. This part of the doc was previously unverified and is confirmed accurate.

**Not verified as actually used on the crew's own shared box (node3)**: `~/.claude.json` on this machine has no `kurultai` entry in `mcpServers` — only a `railway` MCP server is wired. The crew's day-to-day use of the shared instance (per `~/dev/CLAUDE.md`) is direct CLI (`kurultai search "..."`, `kurultai daemon --port 8421` for the Brain UI), not MCP tool-calling from inside a session. So while `init --agent claude` works, it has not actually been run for the crew's shared instance — a real gap between "wired" and "documented as wireable."

### 3. Project scoping — real namespacing for N sessions sharing one store

`docs/PROJECT_SCOPING.md` (repo-native, more current than this section used to be) documents the actual mechanism for several Claude Code sessions sharing one Kurultai brain on one box without polluting each other's recall: `project_id` in `KnowledgeAtom.metadata`, set via a `project` arg on `remember`/`recall` or the `KURULTAI_PROJECT` env var, normalized (trimmed/lowercased/64-char cap) on both write and read. **Only `recall` is project-aware** — `search`, `ask`, `cite`, `who_knows` all still span every project regardless of `KURULTAI_PROJECT`. This is namespacing (accidental cross-pollution), explicitly **not** isolation/security (any local process can read the whole SQLite file directly) — the doc says so in bold, don't describe it otherwise. `kurultai init` does not wire `KURULTAI_PROJECT` per session; that's still manual per MCP client config today.

### 4. Shared-write containment — real, but off by default

`src/write_policy.rs` (feature `shared_write`, since v0.5.1, **default off** — check with `KURULTAI_FEATURE_SHARED_WRITE=0|1`, see `src/features.rs`) forces agent-reachable writes (`remember`, `POST /ingest`, daemon HTTP) into quarantine regardless of what the quality gate would otherwise decide, and restricts `promote`/`ontology_promote` crossing quarantine→trusted to the CLI actor only when the flag is on. With the flag off (the default, and what's running on this box today — confirmed via `env | grep KURULTAI` returning nothing), any MCP session can still `remember` then immediately `promote` its own write with no containment. `docs/crew-instance-node3.md`'s "Not yet done" list (self-promotion, single namespace) predates this flag and `project_id`/`recall` (both landed the same day, right after that doc was written) — treat that doc's own gap list as **stale**; the mechanisms it asked for now exist in code, just not turned on by default on this box.

### Egress caveat (Brain UI 3D graph)

The Brain UI loads `three.js` + `3d-force-graph` from `unpkg.com`. In the restricted
cloud network `unpkg.com` is blocked, so the "3D Synaptic Network" canvas stays blank.
The rest of the UI (stat cards, Atom Explorer, document detail) works — it reads from
the local `/api` endpoints. Allowlist `unpkg.com` if the 3D graph is needed.

### web/ (Clerk) blocker

`web/` requires Clerk secrets: `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` and `CLERK_SECRET_KEY`
(see `web/.env.example`). The dev server boots and the homepage renders, but protected
routes (e.g. `/dashboard`) return **500 "Missing publishableKey"** until the keys are set.

### Lint / test / build (Rust) — verified locally, not assumed

See `CONTRIBUTING.md`. Quick reference: `cargo fmt --all -- --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test --locked`,
`cargo build --release --locked`. Note CI sets `RUSTFLAGS=-Dwarnings`.

Actually run on a bare-cargo environment (no `rustup`, `cargo` 1.97.1 present
with no `fmt`/`clippy` subcommands installed) while writing this pass:

- `cargo build` (debug): **clean**, no warnings.
- `cargo test --locked`: **clean, 229 unit tests + every integration test
  file passing** (`acceptance_*`, `retrieval_hybrid`, `phase3_ask_test`,
  `phase4_connectors_test`, `phase5_daemon_test`, `json_ingestion_test`,
  `install_script_test`, etc. — 0 failures anywhere).
- `cargo fmt --all -- --check` / `cargo clippy --all-targets -- -D warnings`:
  **could not run** — `error: no such command: fmt`/`clippy`. This
  environment has no `rustup` and the toolchain's `rustfmt`/`clippy`
  components (declared in `rust-toolchain.toml`) were never installed
  alongside plain `cargo`/`rustc`. This is an environment gap, not a repo
  problem — CI installs these via `dtolnay/rust-toolchain@stable` with
  `components: rustfmt, clippy`. If you land on a similarly bare box, expect
  the same and say so rather than assuming these checks ran.

CI (`.github/workflows/ci.yml`) actually has **four** jobs, not just the
lint+test one this doc previously implied:

1. **Lint & Test** — fmt check, clippy (`-D warnings`), `cargo nextest run
   --locked`, `cargo llvm-cov` (coverage report, uploaded as an artifact,
   **informational only, no fail gate**), `cargo build --release --locked`.
2. **Brain UI build** — runs `scripts/build-ui.sh` (Node, `.nvmrc`-pinned)
   and fails if the embedded `ui/` is stale relative to `website/` source.
3. **Dependency audit** — `cargo audit` as its own job (the previous version
   of this doc implied it ran inside the lint/test job; it's separate).
4. **Postgres store** — spins up a `pgvector/pgvector:pg16` service
   container to exercise the HUB-2 Postgres backend (`--features postgres`).

### Roadmap / live work queue — do not invent a new roadmap doc, these already exist

- **Live, current queue**: [`docs/plans/phase-6-next-work-orders.md`](docs/plans/phase-6-next-work-orders.md) — Wave G (Tiered Access + Hosted Hub) is next, HUB-3 (hub-mode daemon, Tailscale/API-key bind) is the current head-of-queue item as of the repo's own root `INDEX.md` Recent log. Version flags (`fts`, `brain_ui`, `mcp_http`, `local_embed`, `shared_write`, `hub`) and their default state are cataloged in `src/features.rs` / `kurultai status`.
- **[`docs/plans/YEAR-1-MILESTONES.md`](docs/plans/YEAR-1-MILESTONES.md) is explicitly marked stale** by the repo's own root `INDEX.md` ("stale (~July 2026)") — it's a July-2026 cashflow/milestone plan (v0.3.1 → v1.0.0, Q3 2026–Q2 2027) that has already been superseded in sequencing by the Wave G work-orders doc above. Useful for the long-arc business framing (pricing tiers, deferred-to-Year-2 list, risk register) but do not treat its dates or its "current milestone" framing as current.
- No standalone `ROADMAP.md` was created for this pass — the durable phased content already lives in the two docs above, actively maintained, and a third file would just fork the source of truth.
