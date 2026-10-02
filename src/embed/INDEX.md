---
index: kurultai/v1
folder: src/embed
parent: src/INDEX.md
updated: 2026-08-16
version: 1
---

# `src/embed`

**Does:** Embeddings (cloud / local / null)
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`local.rs`](local.rs) | Optional fastembed ONNX local embedder | `src/embed` · `src/error` | `src/app/context.rs` · `src/doctor.rs` · `src/http/ingest.rs` · `src/mcp/brain.rs` · `src/pipeline/mod.rs` | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Embedder trait, OpenRouter, NullEmbedder | `src/error` | `src/app/context.rs` · `src/doctor.rs` · `src/embed/local.rs` · `src/http/ingest.rs` · `src/mcp/brain.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`perplexity.rs`](perplexity.rs) | Perplexity pplx-embed-v1 embedder (int8 base64 → f32 decode) | `src/error` · `base64` · `reqwest` | `src/app/context.rs` | 2026-10-02 | 1 | 2026-10-02 added |

## Recent

- 2026-10-02 — `perplexity.rs` added: `embed.backend = "perplexity"` → PerplexityEmbedder via `PERPLEXITY_API_KEY`; API returns quantized base64 int8 only, decoded client-side
- 2026-08-16 — indexed this folder (v1 seed)

