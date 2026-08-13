---
title: "feat: Config-not-code adapters (inbox, loopback ingest, folder format parity)"
date: 2026-08-12
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: session
execution: code
authority: "Dump adapters · inbox tray · loopback webhook · format parity"
depth: standard
origin: "Session-settled product contract for config-not-code dump edges"
status: draft
---

# feat: Config-not-code adapters (inbox, loopback ingest, folder format parity)

**Target repo:** `duketopceo/kurultai`  
**Audience:** developer → operators dumping notes/JSON into the brain  
**Base:** `main`  
**Process:** PR-only · branch `feat/config-not-code-adapters`

## Goal Capsule

Ship three dump edges together so operators add knowledge via **config and files**, not custom connector code: (1) **inbox tray** folder that moves successes to `processed/` and failures to `failed/` with a reason sidecar; (2) **loopback webhook** `POST /ingest` with required shared secret; (3) **folder markdown + JSON format parity** so both source kinds accept the same dump formats (markdown, JSON/NDJSON, plain text) through one atomizer. Untagged atoms quarantine until explicit promote; soft labels never satisfy tags; hard quality heuristics run on ingest (no LLM judge); light `quality_score` boost on retrieval only.

**Stop when:** shared dump atomizer exists; inbox + webhook + folder parity wired; gate heuristics + quality boost + skip-embed-on-quarantine; watch roots include Json/Inbox; docs + tests green (fmt, clippy `-D warnings`, nextest/cargo test).

**Do not:** hub/public webhook; hot-reload; Custom connector protocol; globs; LLM quality judge.

**Product Contract preservation:** Session-settled — no LLM judge v1, tag gate stays, webhook loopback + secret required.

---

## Product Contract

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | Shared dump atomizer used by inbox, webhook, and folder markdown/json edges |
| R2 | Dump formats: markdown, JSON/NDJSON, plain text |
| R3 | Stable `source_id` from relative path (+ JSON record index); use `atom_id_from_hash` — not remember timestamp ids |
| R4 | `SourceKind::Inbox`; keep markdown/json kinds; Custom stays rejected |
| R5 | Inbox walks tray excluding `processed/` and `failed/`; after gate+upsert trusted → `processed/`; quarantine or parse fail → `failed/` + `{name}.reason.txt`; still STORE quarantined atoms |
| R6 | Offline inbox works via `kurultai index` without daemon |
| R7 | `POST /ingest` outside `/api/`; env `KURULTAI_INGEST_SECRET` required or route disabled; constant-time secret compare; peer must be loopback even if process binds `0.0.0.0` |
| R8 | Ingest response `{ ok, atom_ids?, lane, quarantine_reason? }` — no brain dump |
| R9 | After untagged + exact trusted dupe in `quality::gate::evaluate`, add heuristics: trim length < 80 → `low_quality:too_short`; thin/boilerplate → `low_quality:thin`; promote re-runs evaluate |
| R10 | Soft labels never satisfy the tag gate |
| R11 | Skip/clear embed on quarantine writes |
| R12 | Store `quality_score` 0–1 in atom metadata; small post-RRF boost in hybrid next to soft-label boost; trusted only; `include_quarantine` unchanged |
| R13 | `watch_roots_from_sources` includes Json + Inbox roots |
| R14 | Document one source per mixed folder |

### Actors / Flows

| ID | Actor / flow |
|----|--------------|
| A1 | Operator drops dump files into an inbox tray |
| A2 | Cron/script POSTs dump body to loopback `/ingest` |
| A3 | Operator points a markdown or json folder source at a dump directory |
| A4 | Offline `kurultai index` drains inbox without daemon |
| A5 | Operator promotes a quarantined dump atom after fixing tags |

### Acceptance Examples

| ID | Example |
|----|---------|
| AE1 | Dump `.md` with frontmatter tags → trusted atom; searchable |
| AE2 | Dump untagged plain text → quarantine `untagged`; file (inbox) → `failed/` + reason sidecar; atom still stored |
| AE3 | Dump body trim length < 80 with tags → quarantine `low_quality:too_short` |
| AE4 | Thin/boilerplate tagged dump → quarantine `low_quality:thin` |
| AE5 | Soft labels alone do not satisfy tag gate (regression) |
| AE6 | `POST /ingest` without secret or from non-loopback → rejected; with secret + loopback → atoms stored; response has no full brain dump |
| AE7 | Inbox trusted dump → `processed/`; `kurultai index` drains inbox offline; markdown and json folder sources both accept md/json/txt dumps with path-stable ids |

### Scope boundaries

**In:** R1–R14; `src/ingest/`; inbox connector; HTTP ingest; gate heuristics; hybrid quality boost; pipeline skip-embed-on-quarantine; watch/status; docs; tests.

**Out (explicit)**

- Hub / public webhook
- Hot-reload of config or tray
- Custom connector protocol
- Glob patterns for folder selection
- LLM quality judge

---

## Planning Contract

### Key Technical Decisions

| KTD | Decision | Why |
|-----|----------|-----|
| KTD1 | Shared dump atomizer in `src/ingest/dump.rs` used by all three edges | One mapping, three surfaces |
| KTD2 | Stable `source_id` from relative path (+ JSON record index); `atom_id_from_hash` | Re-index converges; do not copy remember timestamp ids |
| KTD3 | `SourceKind::Inbox`; keep markdown/json; Custom rejected | Config-not-code without opening Custom protocol |
| KTD4 | Inbox connector owns tray; walk exclude processed/failed; move after gate+upsert | Operator-visible success/fail folders |
| KTD5 | `POST /ingest` outside `/api/`; `KURULTAI_INGEST_SECRET` required; constant-time compare (`secrets_equal`); loopback peer check | Local push without exposing hub |
| KTD6 | Gate heuristics after untagged + exact dupe: `<80` → `too_short`; thin/boilerplate → `thin` | Hard quality without LLM |
| KTD7 | Skip/clear embed on quarantine | Don't pay embed or pollute `atoms_vec` |
| KTD8 | `quality_score` in metadata; small post-RRF boost next to soft-label boost; trusted only | Light retrieval signal |
| KTD9 | `watch_roots_from_sources` includes Json + Inbox | Daemon picks up folder dumps |
| KTD10 | Document one source per mixed folder | Avoid double-index of same dumps |

### Implementation Units

| Unit | Deliverable |
|------|-------------|
| U1 | `src/ingest/dump.rs` (+ mod) + unit tests |
| U2 | `markdown.rs` + `json.rs` format parity via dump; extend `tests/json_ingestion_test.rs` |
| U3 | Gate heuristics in `quality/gate.rs`; update `docs/solutions/architecture-patterns/trust-lanes-quality-gate.md` |
| U4 | Inbox connector; types/config/loader/registry; `tests/inbox_adapter_test.rs`; `config.example.toml` |
| U5 | `src/http/ingest.rs` + daemon `ServeOptions`; tests |
| U6 | Daemon watch Json+Inbox; `/api/status` + CLI status inbox pending/failed counts |
| U7 | Hybrid quality boost tests |
| U8 | README + config.example.toml + trust-lanes doc |

### Test plan

Match existing style: `#[cfg(test)]` + `tests/` fixtures, `NullEmbedder`, tempdir. Map AE1–AE7 into tests. Must pass: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo nextest` (or `cargo test --locked`).

### Session-settled notes

- No LLM judge in v1
- Tag gate stays (soft labels never count)
- Webhook is loopback + shared secret only
