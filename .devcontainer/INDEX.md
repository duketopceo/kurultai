---
index: kurultai/v1
folder: .devcontainer
parent: INDEX.md
updated: 2026-09-01
version: 1
---

# `.devcontainer`

**Does:** Local dev-container definitions for dogfood testing on Debian 12 and Ubuntu 24.04
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Files

|| File | Does | Needs | Touches | Stamp | Ver | Changelog |
||------|------|-------|---------|-------|-----|-----------|
|| [`Dockerfile`](Dockerfile) | Multi-runtime build image: installs Rust and runs `cargo test` + `kurultai --version` | — | — | 2026-09-01 | 1 | 2026-09-01 v0.5.0 Debian + Ubuntu dogfood |
|| [`INDEX.md`](INDEX.md) | INDEX.md | — | — | 2026-09-01 | 1 | 2026-09-01 indexed (v1 seed) |
|| [`devcontainer.json`](devcontainer.json) | VS Code dev container base config (Debian 12) | — | — | 2026-09-01 | 1 | 2026-09-01 tracked local devcontainer |
|| [`docker-compose.yml`](docker-compose.yml) | Debian 12 + Ubuntu 24.04 smoke-test services | [`Dockerfile`](Dockerfile) | — | 2026-09-01 | 1 | 2026-09-01 v0.5.0 dogfood |

## Recent

- 2026-09-01 — v0.5.0 dogfood green on Debian 12 and Ubuntu 24.04 via `docker compose`
