---
title: "feat: v0.4.1 production release prep (CLI, UI build, feature flags)"
date: 2026-08-13
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: code
authority: "User /lfg — clean up terminal commands; UI build and load; feature flags with v# next work orders; continue to production release"
---

# feat: v0.4.1 production release prep

**Target repo:** `duketopceo/kurultai`  
**Base:** `main`  
**Do not:** tag `v0.4.1` from this PR (human tags after merge); Brain 3D redesign; hub/Postgres; crates.io publish.

## Goal

Make the solo kernel **releasable**: CLI help matches how people actually use the binary, Brain UI can be rebuilt and is what the daemon loads, product flags are named with versions, crate/docs match GitHub `v0.4.0` and prepare `v0.4.1`.

## Facts

- GitHub latest release is **v0.4.0**; `Cargo.toml` / README still say **0.3.0**.
- `website/` TypeScript 7 cannot `npm run build` (TS2882 CSS side-effect import); CI never builds UI; release binaries embed whatever is in `ui/`.
- Next team work is Wave G **v0.5.0** (`hub` flag), not a skipped v0.3.1.

## Units

- U1 CLI: clap `help_heading` groups; shorter about; `who_knows` alias; `status` prints flags.
- U2 flags: `src/features.rs` + `KURULTAI_FEATURE_<ID>=0|1`.
- U3 UI: `scripts/build-ui.sh`, `.nvmrc`, CI job `git diff --exit-code ui/`, TS2882 fix.
- U4 docs: CHANGELOG, README/install tag `v0.4.0`, crate `0.4.1`, versioned next work orders.

## Stop when

`kurultai --help` is grouped; `kurultai status` lists flags; `scripts/build-ui.sh` succeeds and CI enforces `ui/`; crate is 0.4.1; PR open. No git tag in this slice.
