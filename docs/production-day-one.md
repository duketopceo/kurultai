# Production day-one — Phase 4 solo stack

Use this when you want Kurultai indexing notes + optional Dayflow / Pond / local GitHub checkouts **today**.

Phase 4 connectors and Phase 5 daemon poll/watch are already on `main`. This is the operator path, not a new feature build.

## 1. Build

```bash
cargo build --release
# optional: cargo install --path . --locked
```

## 2. Init (Cursor MCP)

```bash
kurultai init --agent cursor
```

Creates `~/.config/kurultai/config.toml` if missing (empty sources) and wires Cursor MCP to `kurultai mcp`.

## 3. Configure sources

```bash
cp examples/config.solo.toml ~/.config/kurultai/config.toml
# edit paths; set enabled = true for sources you have
```

| Source | `kind` | Needs |
|--------|--------|--------|
| Notes / vault | `markdown` | `root_path` to a `.md` folder |
| Dayflow | `dayflow` | macOS Dayflow install (or `db_path`) |
| Pond | `pond` | `pond` binary on PATH (or `pond_bin`) |
| Code | `github` | local checkout `root_path` |

**AppFlowy** remains deferred ([#4](https://github.com/duketopceo/kurultai/issues/4)).

FTS-first works **without** `OPENROUTER_API_KEY`. Set the key only when you want embeddings / optional rerank.

For prod storage defaults:

```bash
export KURULTAI_ENV=prod
# or: environment = "prod" in config.toml
```

## 4. Index

```bash
kurultai index --full
kurultai status
```

Incremental later:

```bash
kurultai index
```

## 5. Query

```bash
kurultai search "database migration" --limit 10
kurultai ask "what deployments are we running?"
kurultai who-knows "sqlite"
```

## 6. Agents + daemon

```bash
# MCP stdio (Cursor / agents) — usually via init wiring
kurultai mcp

# HTTP + incremental poll + filesystem watch (markdown/github roots)
kurultai daemon --port 8421
# kurultai daemon --no-poll
# kurultai daemon --no-watch
```

## Soft-fail notes

- Missing Dayflow DB / Pond binary → clear connector error; other sources still index.
- Watch covers markdown + github roots; Pond/Dayflow stay poll-interval based.
- Daemon binds localhost; do not expose without auth (not shipped).

## More

- Example config: [`examples/config.solo.toml`](../examples/config.solo.toml)
- Phase 4 wrap: [`docs/plans/phase-4-complete.md`](plans/phase-4-complete.md)
- Master plan: [#27](https://github.com/duketopceo/kurultai/issues/27)
