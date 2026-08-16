---
index: kurultai/v1
folder: .github/workflows
parent: .github/INDEX.md
updated: 2026-08-16
version: 1
---

# `.github/workflows`

**Does:** GitHub Actions
**Up:** [`.github/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`ci.yml`](ci.yml) | CI: fmt/clippy/nextest, Brain UI, macOS smoke, cargo-audit, postgres, agent-index | `.github/workflows/ci.yml` | `scripts/audit-agent-index.py` | 2026-08-16 | 2 | 2026-08-16 added Agent index job · 2026-08-16 indexed (v1 seed) |
| [`deploy.yml`](deploy.yml) | Deploy pipeline: staging branch → staging env, main → production env. | — | — | 2026-07-19 | 1 | 2026-08-16 indexed (v1 seed) |
| [`release.yml`](release.yml) | Build platform binaries and attach them to a GitHub Release. | — | — | 2026-07-26 | 1 | 2026-08-16 indexed (v1 seed) |
| [`self-hosted.yml`](self-hosted.yml) | Disabled until ARC runners exist (kurultai-runners scale set — #20). | — | — | 2026-08-07 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-16 — `ci.yml`: Agent index job runs `scripts/audit-agent-index.py`
- 2026-08-16 — indexed this folder (v1 seed)

