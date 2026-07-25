# Test fixtures

| Path | Purpose | Golden token |
|------|---------|--------------|
| `vault/` | Markdown connector + index E2E | `KNOWN_PHRASE_KURULTAI_42` (`ops/deploy.md`) |
| `code_repo/` | GitHub filesystem connector | `KNOWN_GITHUB_PHRASE_42` (`src/lib.rs`) |
| `pond_stub.sh` | Fake `pond` binary for Pond connector E2E | `KNOWN_POND_PHRASE_77` |

Dayflow uses an in-test temp SQLite (`timeline_cards`) — golden: `KNOWN_DAYFLOW_PHRASE_88`.
