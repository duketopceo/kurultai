# Changelog

## [0.4.1] — unreleased

Production-prep for the next GitHub Release (tag `v0.4.1` after merge to `main`).

- CLI help grouped: Setup · Knowledge · Serve · Packs · Maintenance; `who_knows` alias; status lists versioned feature flags (`KURULTAI_FEATURE_<ID>`).
- Solo on-device docs: `kurultai init --docs` provisions a tagged markdown folder (`Documents/kurultai` or `~/kurultai`).
- Brain UI: `scripts/build-ui.sh` + CI job so `website/` source matches embedded `ui/` (daemon `GET /ui/`).
- Docs: crate version aligned with the v0.4.x line (GitHub latest remains [v0.4.0](https://github.com/duketopceo/kurultai/releases/tag/v0.4.0) until tagged).

Already on `main` since v0.4.0 (included in this crate bump): MCP HTTP/SSE, thin metrics, HUB-1 visibility scopes, config-not-code adapters (inbox / ingest / folder parity).

## [0.4.0] — 2026-07-30

Brain UI: solar · pulse · purple · max. See [GitHub Release](https://github.com/duketopceo/kurultai/releases/tag/v0.4.0).

## [0.3.0] — 2026-07-26

Unification: ingestion jobs, hardening, UI. See [GitHub Release](https://github.com/duketopceo/kurultai/releases/tag/v0.3.0).
