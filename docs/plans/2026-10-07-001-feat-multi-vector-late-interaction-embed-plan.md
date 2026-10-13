# Plan: Multi-vector late-interaction embed lane (pplx-embed-v2-late)

## Problem

Perplexity shipped `pplx-embed-v2-late-{0.6b,9b}` (MIT, HF-only, no API endpoint
yet) — ColBERT-style retrievers emitting one 128-dim vector **per token** and
scoring query↔doc with MaxSim. Kurultai's entire vector lane is single-vector
dense (`Embedder` trait → `Vec<f32>` → `sqlite-vec` KNN), so the new model is
not a config flip. This plan adds a **late-interaction rerank lane** on top of
the existing dense+FTS recall, feature-gated, without migrating or rebuilding
the primary vector store.

Tracking: [kurultai#424](https://github.com/duketopceo/kurultai/issues/424).

## Settled decisions (KTDs)

- **KTD-1 — Rerank lane, not a new primary index.** Dense v1 (`pplx-embed-v1-0.6b`,
  1024-dim) + FTS5 stays the first-stage recall; multi-vector vectors only
  rescore the top-K candidate set. This preserves the existing `atoms_vec`
  store, hosted configs, and `embed_dim` invariants with zero migration risk.
  *Rejected alternative: replace dense lane — multi-vector primary retrieval
  needs a full index rewrite and the model has no hosted API yet.*
- **KTD-2 — Storage: per-token sidecar table, not vec0.** A new
  `atoms_multivec` table (`atom_id, token_idx, vec BLOB(512B f32×128)`) plus a
  `atom_id → COUNT` cache. sqlite-vec is single-vector-per-row and MaxSim isn't
  expressible in vec0 anyway; we score in Rust against a candidate set, so ANN
  indexing is unnecessary at our scale (thousands of atoms, top-K ≤ ~200).
  *Rejected alternative: external vector DB — overkill for a rerank lane.*
- **KTD-3 — Backend: fastembed `local-embed` first, sidecar service second.**
  fastembed 7.x already carries late-interaction models (`colbert-ir/colbertv2.0`)
  and runs in-process in Rust. If/when `pplx-embed-v2-late` lands in fastembed
  (or exports to ONNX cleanly) it plugs into the same lane; until then a
  Python sidecar (`sentence-transformers` ≥6, `MultiVectorEncoder`) behind a
  tiny HTTP contract is the pplx-v2 vehicle. The trait contract is identical.
  *Rejected alternative: Python-only — adds a hard runtime dep for all users.*
- **KTD-4 — Feature-gated off by default.** `late-interaction = "off" | "local" | "http"`
  config; `off` is byte-identical to today. The lane is additive and droppable
  (`DROP TABLE atoms_multivec`) — no `--rebuild-vectors` of the dense store.
- **KTD-5 — 0.6B is the target model.** 340M active params, ViDoRe markdown
  nDCG@10 61.2% vs 9B's 64.7% — the gap is small and 0.6B runs on CPU/laptop.
  The shared embedding space means a later 9B index upgrade doesn't invalidate
  0.6B query vectors.

## Units

### U1 — `MultiVectorEmbedder` trait + `MultiVectorStore`

New trait parallel to `Embedder` (`src/embed/mod.rs`):

```rust
#[async_trait::async_trait]
pub trait MultiVectorEmbedder: Send + Sync {
    fn name(&self) -> &str;
    fn token_dim(&self) -> usize;              // 128 for pplx-embed-v2-late
    async fn embed_query(&self, text: &str) -> Result<Vec<Vec<f32>>>;
    async fn embed_document(&self, text: &str) -> Result<Vec<Vec<f32>>>;
}
```

Query/document asymmetry is real in this model family (Q/D marker ordering —
the HF card notes PyLate's marker position differs), so the trait keeps them
separate.

Store side (`src/store/mod.rs` + migrations): schema v18 adds
`atoms_multivec(atom_id TEXT, token_idx INTEGER, vec BLOB, PRIMARY KEY
(atom_id, token_idx))` and `atoms_multivec_meta(atom_id PRIMARY KEY, token_count,
model)` — created lazily only when the lane is enabled, so existing stores are
untouched. `MaxSim` scorer in `src/embed/maxsim.rs`: cosine per query-token
over doc-token matrix, sum of maxes. Unit tests on fixed vectors.

### U2 — fastembed late-interaction impl (`src/embed/late_fastembed.rs`)

Behind `local-embed` feature: wraps `fastembed::TextEmbedding` for a ColBERT
family model (start with `colbert-ir/colbertv2.0` — same contract, same
token_dim=128; pplx-v2 slots in when fastembed/ONNX export lands). Config:

```toml
[embed.late]
backend = "off"        # off | fastembed | http
model = "colbert-ir/colbertv2.0"
rerank_k = 200         # candidate pool size
max_doc_tokens = 512   # per-atom cap; long atoms stay dense-only
```

### U3 — Sidecar `LateReranker` in the search pipeline

`src/search/` or wherever the hybrid merge happens: after dense-KNN ∪ FTS
candidate fusion (top `rerank_k`), fetch multivecs for candidates, score via
MaxSim, blend: `score = α·dense_rank + (1−α)·maxsim_rank` (start α=0.4,
configurable). Quarantine/tier/supersede filters apply **before** rerank so
trust semantics are unchanged. Gated entirely on `[embed.late].backend != "off"`.

### U4 — Python sidecar for pplx-v2 (optional, `scripts/` + `embed.backend = "http"`)

Minimal `scripts/late-embed-server.py`: sentence-transformers
`MultiVectorEncoder` serving `POST /embed_query` + `POST /embed_document`
(returning `[[f32;128];T]` JSON). Rust side is a `LateHttpEmbedder` implementing
the U1 trait — same shape as `PerplexityEmbedder`. Ship as opt-in tooling for
hosted lanes; documents `pip install 'sentence-transformers>=6.0.0'`.

### U5 — Evals + docs

- `evals/`: extend the existing retrieval eval harness with a `--late` flag
  comparing dense-only vs dense+late nDCG on the demo corpus + a markdown-heavy
  fixture set. Landing criterion: rerank lane must not regress top-10 recall on
  the existing eval set; report the delta.
- `docs/`: CONCEPTS.md entry (`late-interaction`, `MaxSim`), AGENTS.md +
  folder INDEX.md rows, config.example.toml `[embed.late]` block.

## Test scenarios

- U1: MaxSim scorer on synthetic Q/D matrices (self-match scores high;
  orthogonal docs low); multivec round-trip through `atoms_multivec`.
- U2: `LateEmbedder` feature-gated builds (`--features local-embed`) embed a
  doc to N×128; empty text rejected like `reject_empty_embed_texts`.
- U3: `[embed.late] backend="off"` → identical search results byte-for-byte;
  `backend="local"` reorders a fixture where dense rank 1 is semantically
  wrong but token-overlap right; quarantined/superseded atoms never reranked
  into results.
- U4: sidecar contract test against a recorded fixture (no GPU needed in CI).
- U5: eval delta report committed as a comment on #424.

## Risks

- **pplx-v2 ONNX export may not exist yet** — U2 starts with colbertv2.0 to
  prove the lane; pplx-v2 arrival is a config value, not a redesign.
- **Latency**: MaxSim over top-200 × ~128 tokens is ~3M flops — trivial; but
  multivec storage is ~64KB/atom (512 tokens × 128 dims × 4B) → 4,145 atoms ≈
  270MB. Acceptable; `max_doc_tokens` caps growth.
- **fastembed model coverage**: verify `colbert-ir/colbertv2.0` is in fastembed
  7's registry before U2 lands; fallback is U4 sidecar first.

## Out of scope

- Replacing the dense primary index or migrating `atoms_vec`.
- Multimodal (image) atoms — the model supports them; our atom type doesn't.
  Design keeps `embed_document(text)` so images are a future extension.
- `pplx-decider-v1.1-27b` (Decision Index 61.56 vs Jev 57.9) — no API, 49GiB
  self-host; noted on #424 as a Jev-swap candidate if API-hosted later.
