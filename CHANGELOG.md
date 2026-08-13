# Changelog

## [0.5.0] — unreleased

Shared hub track (product flag `hub`, default **off**). Crate version stays 0.4.1 until this line ships.

- HUB-2: optional `--features postgres` `PostgresStore` (pgvector) for `team`/`company` atoms. `open_store` remains SQLite. Personal upserts are rejected (AE4). One Postgres database per organization; copy shared-tier atoms only — never personal, never in-place convert of `store.db`.

## [0.4.1] — 2026-08-13

Production GitHub Release for the solo kernel (tag `v0.4.1`).

- CLI help grouped: Setup · Knowledge · Serve · Packs · Maintenance; `who_knows` alias; status lists versioned feature flags (`KURULTAI_FEATURE_<ID>`).
- Solo on-device docs: `kurultai init --docs` provisions a tagged markdown folder (`Documents/kurultai` or `~/kurultai`).
- Brain UI: `scripts/build-ui.sh` + CI job so `website/` source matches embedded `ui/` (daemon `GET /ui/`).
- Boot: graph load is no longer aborted by status polling; HTML and `/api/*` `Cache-Control: no-store`; `index` / `status` / Brain bar show UI and backend versions.

Already on `main` since v0.4.0 (included in this crate bump): MCP HTTP/SSE, thin metrics, HUB-1 visibility scopes, config-not-code adapters (inbox / ingest / folder parity). HUB-2 Postgres store is in tree behind `--features postgres` + `KURULTAI_FEATURE_HUB=1` (default off).

## [0.4.0] — 2026-07-30

Brain UI: solar · pulse · purple · max. See [GitHub Release](https://github.com/duketopceo/kurultai/releases/tag/v0.4.0).

## [0.3.0] — 2026-07-26

Unification: ingestion jobs, hardening, UI. See [GitHub Release](https://github.com/duketopceo/kurultai/releases/tag/v0.3.0).
