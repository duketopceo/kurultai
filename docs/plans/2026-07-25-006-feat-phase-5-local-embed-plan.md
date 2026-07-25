---
title: "feat: Phase 5 local embeddings (OpenAI-compatible HTTP)"
date: 2026-07-25
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
authority: "Milestone 5 · #9 local embeddings; README sequence after poll (#65) + notify (#66)"
depth: standard
origin: "User /lfg next phase — next Phase 5 product slice after daemon poll/watch"
---

# feat: Phase 5 local embeddings (OpenAI-compatible HTTP)

**Target repo:** `duketopceo/kurultai`  
**Audience:** team / solo offline (no OpenRouter cloud key)  
**Base:** `main` after #65/#66  
**Process:** PR-only

## Goal Capsule

**Objective:** Ship the next Phase 5 product slice: **local embeddings** so index/search can produce real vectors without `OPENROUTER_API_KEY`, by calling a **local OpenAI-compatible embeddings HTTP server** (Ollama / TEI / llama.cpp server).

**Authority:** This plan > #9 / Milestone 5 > layer0 OpenAI-compatible HTTP inspire > Stratum “no API keys for search path”.

**Stop when:** `embed.backend = "local"` (or env) wires a live local HTTP embedder; OpenRouter/auto/null unchanged; tests cover selection + mocked local HTTP (no cloud); README Phase 5 notes poll+watch ✅ and this slice; CI green.

**Do not:** Pull `fastembed`/`ort` (CDN blocked in agent env); ARC/#20; GlitchTip/#35; auth/bind; raise coverage to 75%; close Milestone 5.

**Assumption (LFG headless):** “/lfg next phase” = next **Phase 5** product slice (not Phase 6). Evidence during planning: `ort-sys` prebuilt download from `cdn.pyke.io` fails here → prefer HTTP-local over ONNX-in-process for this PR; in-process ONNX/llama.cpp remain follow-ups.

**Product Contract preservation:** new bootstrap.  
**Changed from first draft:** R1/KTD1 switched from optional `fastembed` to OpenAI-compatible local HTTP — reason: ort CDN unavailable.

---

## Product Contract

### Summary

After daemon poll+watch, the offline path still uses `NullEmbedder`. Operators often already run Ollama/TEI locally — wire that as `backend = "local"`.

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | `HttpEmbedder` (or equivalent) posts to OpenAI-compatible `/v1/embeddings` (reuse OpenRouter response shape). |
| R2 | Config `[embed] backend = "auto" \| "openrouter" \| "local" \| "null"` (default `auto`). |
| R3 | Local URL: `embed.local_url` or `KURULTAI_LOCAL_EMBED_URL` (default `http://127.0.0.1:11434/v1/embeddings`). |
| R4 | Local model: `embed.model` (or dedicated field) — default `nomic-embed-text` when backend=local if model still looks like OpenRouter cloud default. |
| R5 | Env `KURULTAI_EMBED_BACKEND` overrides file backend when set. |
| R6 | Selection: null → Null; openrouter → key required; local → HttpEmbedder to local URL (optional `OPENROUTER_API_KEY`/`KURULTAI_API_KEY` as Bearer if present); auto → key→OpenRouter else Null (**local not implicit in auto** — must opt in). |
| R7 | `is_live()==true` for local HTTP embedder. |
| R8 | Unit tests: selection; mock HTTP server returns fixed vector; empty text rejected. |
| R9 | README Phase 5 row: poll (#65) + notify (#66) ✅; local HTTP embed slice; llama.cpp/ONNX follow-up noted. |

### Actors / flows

- A1 Operator with Ollama · F1 set backend=local · F2 index writes vectors · F3 CI

### Scope boundaries

**In:** R1–R9.  
**Out:** In-process ONNX/llama.cpp; changing store schema; Milestone close.

### Acceptance examples

- AE1. No key + auto → NullEmbedder.  
- AE2. backend=local + mock server → live embedder returns dim-sized vector.  
- AE3. backend=openrouter without key → error.  
- AE4. Key + auto → OpenRouter URL (not local).

---

## Planning Contract

### Assumptions

| Decision | Class | Rejected | Why |
|----------|-------|----------|-----|
| Next = Phase 5 local embed, not Phase 6 | inferred | Launch #10 | Milestone 5 open |
| Local HTTP over in-process ONNX this PR | LFG + evidence | fastembed/ort | CDN reset; Ollama already common |
| Local not auto-selected without opt-in | inferred | auto→local when no key | Avoid surprising failed HTTP to :11434 |

### Key Technical Decisions

| KTD | Decision | Why |
|-----|----------|-----|
| KTD1 | OpenAI-compatible HTTP local embedder | No native CDN; shares decode path with OpenRouter |
| KTD2 | Refactor shared HTTP embed client used by OpenRouter + local | DRY |
| KTD3 | Opt-in `backend=local` | Preserve FTS-first default |
| KTD4 | Default local URL Ollama OpenAI compat port | Zero-surprise for Mac solo |
| KTD5 | Mock HTTP in unit tests (no live Ollama in CI) | Reliable CI |

### Sequencing

U1 plan → U2 HttpEmbedder refactor → U3 config/wiring → U4 tests + README.

---

## Implementation Units

### U1. Plan (this file)

### U2. Shared HTTP embedder

**Files:** `src/embed/mod.rs`

**Tests:** empty reject; mock `/v1/embeddings` success path.

### U3. Config + App wiring

**Files:** `src/config/file.rs`, `src/config/loader.rs`, `src/types.rs`, `src/app/context.rs`

**Tests:** AE1–AE4 selection.

### U4. README

**Files:** `README.md`

---

## Verification Contract

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```

---

## Definition of Done

- [ ] Local HTTP embed path + config  
- [ ] Selection + mock tests  
- [ ] README Phase 5 updated  
- [ ] Green PR; Milestone 5 remains open  
