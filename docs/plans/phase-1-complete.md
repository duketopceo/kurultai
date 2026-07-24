# Phase 1 complete — wrap-up

**Status:** ✅ Shipped on `main` (2026-07-21) · **Tracker closeout:** [phase-1-closeout.md](phase-1-closeout.md)  
**Plan:** [phase-1-work-orders.md](phase-1-work-orders.md) · Tracking [#42](https://github.com/duketopceo/kurultai/issues/42) (close via closeout script)  
**Exit loop:**

```
markdown folder → KnowledgeAtom → SQLite (FTS + vec) → CLI search → MCP search/cite/remember
```

---

## What shipped

| Work order | PR | Notes |
|------------|-----|--------|
| #18 Framework | [#19](https://github.com/duketopceo/kurultai/pull/19) | Config, security, logging |
| #29 / #32 Environments | [#30](https://github.com/duketopceo/kurultai/pull/30) | dev/staging/prod paths |
| #37 Doctrine | [#38](https://github.com/duketopceo/kurultai/pull/38) | SQL brain + `AgentAtomView` |
| #40 Upstream matrix | [#41](https://github.com/duketopceo/kurultai/pull/41) | `docs/upstream-inspiration.md` |
| #1 Storage | [#44](https://github.com/duketopceo/kurultai/pull/44) | FTS5 + sqlite-vec `=0.1.6`, zero-vector guard |
| #2 Embed + #31 Markdown | [#45](https://github.com/duketopceo/kurultai/pull/45) | OpenRouter + NullEmbedder; fixture vault |
| #5 CLI + #11 MCP slice | [#46](https://github.com/duketopceo/kurultai/pull/46) | Brain views, stdio MCP, `init --agent cursor`, hash-skip, tests |
| CE plan archive | [#43](https://github.com/duketopceo/kurultai/pull/43) | This plan file on `main` |
| Compound learning | [#47](https://github.com/duketopceo/kurultai/pull/47) | FTS-first / no zero vectors |
| Phase 2 graph note | [#48](https://github.com/duketopceo/kurultai/pull/48) | Diamond orchestration for #6/#7 |

**Deferred (non-blocking):** [#4](https://github.com/duketopceo/kurultai/issues/4) AppFlowy — stub remains; markdown is the exit path.

---

## Exit criteria — verified

1. `kurultai index --full` indexes fixture vault → atoms in SQLite  
2. `kurultai search "KNOWN_PHRASE_KURULTAI_42"` returns FTS hits (vectors when key present)  
3. MCP `search` returns capped `AgentAtomView` excerpts  
4. CI green: fmt, clippy `-D warnings`, test `--locked`, audit, macOS smoke  

Local proof: `cargo test` (lib + `tests/cli_smoke.rs`), `cargo clippy --all-targets -- -D warnings`.

---

## Operator quick path

```bash
cargo build --release
kurultai init --agent cursor          # config + ~/.cursor/mcp.json
# edit ~/.config/kurultai/config.toml — enable markdown source with root_path
kurultai index --full
kurultai search "KNOWN_PHRASE" --limit 5
kurultai status
# Restart Cursor → MCP tools: search / cite / remember
```

FTS-only without `OPENROUTER_API_KEY`. Set the key for live embeddings.

---

## Hardened invariants (do not regress)

| Invariant | Where |
|-----------|--------|
| No zero / near-zero vectors in `atoms_vec` | `store` `MIN_EMBEDDING_NORM` |
| No API key → `NullEmbedder`, FTS-only | `app` / `embed` |
| MCP stdout is JSON-RPC only (logs → stderr) | `mcp/server`, logging |
| Bounded MCP stdin frames + errors (not silent drop) | `mcp/server` |
| Malformed `mcp.json` → refuse overwrite | `mcp/init` |
| Hash-skip preserves vec when content unchanged | `pipeline` + `store` |
| sqlite-vec pinned `=0.1.6` | `Cargo.toml` |

Learning doc: [fts-first-null-embedder-no-zero-vectors.md](../solutions/architecture-patterns/fts-first-null-embedder-no-zero-vectors.md)

---

## Known residuals (accepted → Phase 2+)

| Item | Owner |
|------|--------|
| Hash-skip ignores embed model / title-in-embed text | polish |
| Serial MCP stdio (one slow tool blocks) | Later MCP |
| AppFlowy real connector | **#4** (moved off Phase 1 exit → Expansion) |
| Schema post-train fields | **#33** (Phase 3+) |
| Testing gates (ongoing) | **#23** (cross-cutting; not Milestone 1) |
| Close/move GitHub trackers | Maintainer: [phase-1-closeout.md](phase-1-closeout.md) / `scripts/phase-1-closeout.sh` |

---

## Next

Phase 2 search (**#6**) already merged (#51). Finish hygiene above, then Phase 3 (#7) drafts #54/#55 and testing #53.
