---
index: kurultai/v1
folder: .
updated: 2026-09-01
version: 5
---

# `.`

**Does:** Kurultai repo root — start here
**Up:** — · **Protocol:** [`docs/agent-index.md`](docs/agent-index.md)

## Search recipe

1. Pick a **child folder** below (surfaces).
2. Open that folder's `INDEX.md` — file rows say what it does, needs, and touches.
3. Open the file. Use [`CONCEPTS.md`](CONCEPTS.md) for domain words only.
4. After you edit a file: bump its row (ver / stamp / 3-line changelog) and prepend **Recent** here and on every parent. Then `python3 scripts/audit-agent-index.py`.

Live product queue: [`docs/plans/phase-6-next-work-orders.md`](docs/plans/phase-6-next-work-orders.md) — HUB-5 ✅ (#250) · HUB-6 ✅; next LFG: final stretch to v0.5.0. [`docs/plans/YEAR-1-MILESTONES.md`](docs/plans/YEAR-1-MILESTONES.md) updated 2026-09-01.

## Children

- [`.github/`](.github/INDEX.md) — CI, templates, CODEOWNERS
- [`docs/`](docs/INDEX.md) — Product + agent docs
- [`plans/`](plans/INDEX.md) — Legacy root-level plans (prefer docs/plans/)
- [`plugin/`](plugin/INDEX.md) — Agent Zero plugin (tools, daemon proxy, embedded Brain UI)
- [`scripts/`](scripts/INDEX.md) — Install, UI build, closeout, index audit
- [`skills/`](skills/INDEX.md) — Repo-shipped agent skills
- [`src/`](src/INDEX.md) — Rust CLI + daemon (main product)
- [`tests/`](tests/INDEX.md) — Rust integration + acceptance tests
- [`ui/`](ui/INDEX.md) — Built assets rust-embed serves at GET /ui/
- [`web/`](web/INDEX.md) — Next.js + Clerk team app (optional)
- [`website/`](website/INDEX.md) — Brain UI source (Vite) — do not add a second dashboard

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`.coderabbit.yaml`](.coderabbit.yaml) | CodeRabbit: auto-review off | — | — | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`.compound-engineering/config.local.yaml`](.compound-engineering/config.local.yaml) | Compound Engineering per-checkout local config (now tracked) | — | — | 2026-09-01 | 1 | 2026-09-01 local config no longer ignored |
| [`.dockerignore`](.dockerignore) | Docker build context excludes for the hub image | — | — | 2026-08-29 | 1 | 2026-08-29 HUB-3 hub image context |
| [`.env.example`](.env.example) | Example env vars (API keys, hub bind) | — | — | 2026-08-29 | 2 | 2026-08-29 hub env block (HUB-3) · 2026-08-16 indexed (v1 seed) |
| [`.gitignore`](.gitignore) | Ignored build, env, and agent workspace paths | — | — | 2026-09-01 | 3 | 2026-09-01 add .devcontainer ignore · 2026-08-16 ignore Python __pycache__ · 2026-08-16 indexed (v1 seed) |
| [`.nvmrc`](.nvmrc) | Node 22 pin for website/ui build | — | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`ACCEPTANCE_REPORT.md`](ACCEPTANCE_REPORT.md) | Acceptance Report — KHAN-251 | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`AGENTS.md`](AGENTS.md) | Agent start-here: preferences, daemon/UI facts, MCP wiring | `INDEX.md` · `docs/agent-index.md` | — | 2026-08-16 | 2 | 2026-08-16 point agents at INDEX.md · 2026-08-16 indexed (v1 seed) |
| [`AGENT_SETUP_PROMPT.md`](AGENT_SETUP_PROMPT.md) | Prompt snippet for wiring agents to Kurultai | — | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`CHANGELOG.md`](CHANGELOG.md) | Shipped crate versions and unreleased hub notes | — | — | 2026-09-01 | 3 | 2026-09-01 v0.5.0 release notes · 2026-08-29 HUB-3 unreleased notes · 2026-08-16 indexed (v1 seed) |
| [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) | Contributor covenant | — | — | 2026-07-22 | 1 | 2026-08-16 indexed (v1 seed) |
| [`CONCEPTS.md`](CONCEPTS.md) | Shared domain vocabulary (atoms, hub, FTS-first, ontology) | — | — | 2026-08-29 | 2 | 2026-08-29 Hub paragraph points at railway-hub.md · 2026-08-16 indexed (v1 seed) |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Dev setup, tests, PR and architecture guidelines | `INDEX.md` · `docs/agent-index.md` | — | 2026-08-16 | 2 | 2026-08-16 agent-index subsection · 2026-08-16 indexed (v1 seed) |
| [`Cargo.lock`](Cargo.lock) | Locked Rust dependency graph for reproducible CI | — | — | 2026-08-13 | 1 | 2026-08-16 indexed (v1 seed) |
| [`Cargo.toml`](Cargo.toml) | Rust crate manifest (v0.5.0) and optional features | — | — | 2026-09-01 | 4 | 2026-09-01 bump to v0.5.0 · 2026-08-31 release profile: thin LTO + strip symbols · 2026-08-16 indexed (v1 seed) |
| [`Dockerfile`](Dockerfile) | Multi-stage hub image (`--features postgres`) | `docs/deploy/railway-hub.md` | `docker-compose.hub.yml` | 2026-08-29 | 1 | 2026-08-29 HUB-3 Railway/compose image |
| [`FEATURE_MATRIX.md`](FEATURE_MATRIX.md) | Kurultai Feature Matrix (KHAN-251) | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`INSTALL_GUIDE.md`](INSTALL_GUIDE.md) | Kurultai Install Guide (macOS) | — | — | 2026-08-13 | 1 | 2026-08-16 indexed (v1 seed) |
| [`INSTALL_REPORT.md`](INSTALL_REPORT.md) | Kurultai Install Verification Report | — | — | 2026-08-13 | 1 | 2026-08-16 indexed (v1 seed) |
| [`LICENSE`](LICENSE) | MIT license | — | — | 2026-07-18 | 1 | 2026-08-16 indexed (v1 seed) |
| [`README.md`](README.md) | Recruiter-clean product README: what/why/run/architecture/status | — | — | 2026-09-01 | 3 | 2026-09-01 v0.5.0 release stack facts · 2026-08-19 rewrite for v0.4.1 stack facts · 2026-08-16 indexed (v1 seed) |
| [`SECURITY.md`](SECURITY.md) | Vulnerability reporting | — | — | 2026-07-22 | 1 | 2026-08-16 indexed (v1 seed) |
| [`config.example.toml`](config.example.toml) | Example config.toml for sources and apps | — | — | 2026-09-01 | 2 | 2026-09-01 HUB-5 default_visibility_scope examples · 2026-08-16 indexed (v1 seed) |
| [`docker-compose.hub.yml`](docker-compose.hub.yml) | Local pgvector + hub daemon proof of Railway recipe | `Dockerfile` · `docs/deploy/railway-hub.md` | — | 2026-08-29 | 1 | 2026-08-29 HUB-3 compose proof |
| [`hey.md`](hey.md) | Informal notes / scratch | — | — | 2026-08-08 | 1 | 2026-08-16 indexed (v1 seed) |
| [`rust-toolchain.toml`](rust-toolchain.toml) | Rust toolchain pin | — | — | 2026-07-18 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-01 — track `.compound-engineering/config.local.yaml`, add `.devcontainer/` to `.gitignore`
- 2026-09-01 — v0.5.0 Team released: GitHub tag + release created; queue + Year-1 milestones marked shipped
- 2026-09-01 — v0.5.0 release prep: crate/Cargo.toml → 0.5.0, CHANGELOG, README, index
- 2026-09-01 — HUB-5 ✅ (#250) · HUB-6 ✅; final stretch to v0.5.0 plan + queue cleanup
- 2026-09-01 — HUB-5: source-level `default_visibility_scope` tagged at ingest (`src/types.rs` · `src/pipeline/mod.rs` · `config.example.toml`)
- 2026-08-31 — `Cargo.toml`: release profile with thin LTO + strip symbols (18M → 15M)
- 2026-08-31 — `main.rs`: `init --doctor` reuses `doctor` diagnostic spine
- 2026-09-01 — live queue: HUB-4 ✅ (#247); **next LFG = HUB-5** (#180)
- 2026-08-31 — added `tests/stress_http.rs` — 1000 request mixed-load HTTP stress test
- 2026-08-31 — review fixes: auth DB 500s, reason length, team atom validation, shared hub DDL
- 2026-08-31 — `.github/CODEOWNERS`: require @duketopceo approval for all repo changes
- 2026-08-31 — HUB-4: ensure hub_activity table migrates alongside hub_api_keys
- 2026-08-29 — CLI smoke unsets ambient hub flag so Postgres CI job stays solo
- 2026-08-29 — HUB-3 Railway transport: Dockerfile, compose, `docs/deploy/railway-hub.md`
- 2026-08-27 — added `plugin/` — Agent Zero plugin (KT-004)
- 2026-08-19 — `README.md` recruiter-clean rewrite (Rust/axum/SQLite/MCP/Brain UI facts)
- 2026-08-16 — seeded hierarchical agent `INDEX.md` tree (v1); protocol in `docs/agent-index.md`
- 2026-08-16 — `AGENTS.md` / `CONTRIBUTING.md` point at the map; CI job **Agent index**
