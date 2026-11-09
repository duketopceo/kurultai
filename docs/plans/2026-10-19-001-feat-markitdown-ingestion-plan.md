---
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan user-directed
external_source: microsoft/markitdown (github.com/microsoft/markitdown) — 191k★, MIT, Python, pushed 2026-10-04
---

# markitdown — documents → Markdown ingestion lane

## Goal Capsule

Adopt Microsoft's markitdown as the document-conversion step in the Kurultai ingestion lane (and reusable by home-index): PDF/DOCX/XLSX/HTML/images/zips → Markdown → atoms. Boring, maintained, MIT.

## Product Contract

### The problem

Ingestion currently needs a converter for non-text documents before they can become searchable atoms. markitdown is the best-maintained single tool for that step (191k★, Microsoft-maintained, MCP-server mode included).

### Scope boundaries

- **In:** `pip install markitdown[all]` (or uv), a thin wrapper in the ingestion path, coverage tests on the doc types actually seen (PDF, DOCX, XLSX, HTML, images-with-EXIF/OCR, zip).
- **Out:** OCR-heavy scans (DeepSeek-OCR stays the named follow-on for scanned/image documents — markitdown's OCR hooks are for light cases).
- **Out:** replacing the retrieval/indexing pipeline — this is only the conversion step.

### Requirements

1. Wrapper module (path per repo layout — `ingestion/` or `src/ingest/` wherever document intake lives): `convert(path) -> markdown` with per-format error reporting; never silently drop pages/sections — partial conversion must be flagged in output metadata.
2. Quality gate on real corpus: run against a sample of actual documents; record per-format fidelity (tables? images? headers?) — markitdown's known weakness is complex-PDF tables (mediocre, "usable not perfect"). Document which doc classes route to DeepSeek-OCR instead.
3. Evaluate `markitdown-mcp` mode: it's a ready-made MCP server — candidate for registration behind the ContextForge gateway when it lands (cross-link to the luke-agents gateway plan).
4. Optional-dependencies discipline: `[all]` extras pull heavy deps — pin the extras actually needed (pdf, docx, xlsx, html) not the whole tree.

### Acceptance examples

- A real PDF, DOCX, and HTML file each convert to markdown and become searchable atoms end-to-end.
- A deliberately complex PDF table produces degraded output that is *flagged degraded* (routed to OCR lane or marked) — not silently garbled.

## Repo Touchpoints

- `src/connectors/` — the extension point (dayflow/json precedents); a `markitdown` connector shelling out to the Python tool, feeding atoms through the normal quarantine/tag path
- `docs/` ingestion notes; update `INDEX.md` row for any folder touched (per repo convention)

## Test Scenarios

1. Format matrix: pdf/docx/xlsx/html/img/zip each convert or fail loudly.
2. Corrupt/empty file → error, not empty-string success.
3. Determinism: same file → same markdown twice.

## Risks / Honest Notes

- The failure mode is *silent quality loss* on complex docs — the acceptance bar is flagging degradation, not claiming fidelity.
- If ingestion is already solved for the formats actually encountered, this plan's value drops — the wrapper exists to widen coverage and standardize errors, not to invent a need.
