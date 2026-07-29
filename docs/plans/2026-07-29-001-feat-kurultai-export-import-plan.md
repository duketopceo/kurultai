---
title: Kurultai Export Import Pack - Plan
type: feat
date: 2026-07-29
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
origin: "User /lfg — export Kurultai setup for multi-device ingest/combine; beginning of #80"
tags:
  - multi-device
  - export
  - import
  - plan
  - feat
---

# Kurultai Export Import Pack - Plan

## Goal Capsule

**Objective:** Ship `kurultai export` / `kurultai import` so a user can quickly pack a Kurultai setup (config + store) and ingest it into a **new** or **existing** Kurultai on another device — the thin first slice of multi-device use ([#80](https://github.com/duketopceo/kurultai/issues/80)).

**Authority:** This plan > [docs/multi-user-kurultai.md](../multi-user-kurultai.md) > README multi-device notes > issue #80.

**Stop when:** CLI export writes a `.kurultai` pack; import can replace into empty (or `--force` overwrite) and `--combine` into an existing store; smoke tests cover export→import→search; README + multi-user doc mention the flow; no secrets in the pack.

**Do not:** Cloud sync, Dropbox live sync, MCP agent config portability, SaaS connectors, encrypted upload to a hosted service.

**Assumption:** Pack is an offline file the user moves (AirDrop, USB, scp). Destination re-runs `kurultai init` for MCP and may remap `root_path`s.

## Product Contract

### Summary

Today multi-device means “copy `store.db` or re-index.” Users need a first-class **custom pack** (`*.kurultai`) with manifest + sanitized config + SQLite brain, plus import into empty or combine into existing.

### Requirements

- R1. `kurultai export [-o PATH]` writes a `.kurultai` archive (tar.gz) containing `manifest.json`, `config.toml`, `store.db`.
- R2. Export uses SQLite backup (not a live file copy) from the resolved storage path.
- R3. Pack never includes API keys or MCP agent configs.
- R4. `kurultai import <PATH>` into empty storage installs the store (+ optional config write if missing).
- R5. `kurultai import <PATH> --replace --force` overwrites an existing store.db.
- R6. `kurultai import <PATH> --combine` upserts atoms from the pack into the current store (FTS via upsert; vectors when embed dims match).
- R7. Manifest records format version, schema version, embed_dim, atom_count, kurultai crate version, created_at.
- R8. CLI smoke: index fixture → export → import into new tmp → search works; combine into second store preserves both.

### Actors

- A1. Solo user moving between devices
- A2. CLI / local store
- A3. CI smoke tests

### Scope boundaries

**In:** export/import CLI, pack format, docs, tests.  
**Out:** hosted upload, encryption-at-rest beyond file permissions, incremental sync, live WAL sync via cloud disks.

## Planning Contract

### Key Technical Decisions

- KTD1. **Format = gzip tar named `*.kurultai`.**  
  Provenance: `user-directed` (“kurultai custom”).  
  Rejected: raw `store.db` only — loses config/manifest.  
  Reason: one file to move; inspectable with `tar tzf`.

- KTD2. **Replace vs combine are explicit flags.**  
  Default import: replace only when target DB missing/empty; refuse non-empty without `--force` or `--combine`.  
  Provenance: `user-directed` (new or existing).  
  Rejected: silent overwrite.  
  Reason: fail loud.

- KTD3. **Combine via Store upsert, not file merge.**  
  Provenance: `user-approved` (safe FTS/vec path exists).  
  Rejected: ATTACH+SQL copy of virtual tables — fragile with fts5/vec0.  
  Reason: reuse `upsert_sync` invariants.

- KTD4. **Vectors: full fidelity on replace; best-effort on combine.**  
  If embed_dim mismatches, combine atoms without embeddings and warn.  
  Rejected: fail combine entirely on dim mismatch.  
  Reason: FTS still useful across devices.

### Sequencing

U1 pack module → U2 CLI → U3 tests/docs.

## Implementation Units

### U1. `src/export` pack/unpack

**Files:** `src/export/mod.rs`, `src/lib.rs`, `Cargo.toml` (`tar`, `flate2`)

**Approach:** manifest serde; SQLite `backup` to temp; write tar.gz; unpack validate; combine helper pages `list_atoms` + optional vec load + `upsert_batch`.

**Scenarios:** unit tests for manifest round-trip; empty store export; refuse bad magic/version.

### U2. CLI `export` / `import`

**Files:** `src/main.rs`, touch README + `docs/multi-user-kurultai.md`

**Approach:** clap variants; resolve paths via same config/env as `status`; print next steps (init, remap roots).

### U3. CLI smoke

**Files:** `tests/cli_smoke.rs`

**Scenarios:** export→import replace search; export→combine into second indexed store finds atoms from both.

## Verification Contract

```bash
cargo test --locked --lib export::
cargo test --locked --test cli_smoke export_import
cargo clippy --all-targets -- -D warnings
```

## Definition of Done

- [ ] `.kurultai` export/import shipped
- [ ] replace + combine paths work
- [ ] smoke tests green
- [ ] docs updated; #80 thin slice noted
- [ ] no secrets in pack
