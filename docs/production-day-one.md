# Production day-one — Phase 4 solo stack

Use this when you want Kurultai indexing notes + optional Dayflow / Pond / local GitHub checkouts **today**.

Phase 4 connectors and Phase 5 daemon poll/watch are already on `main`. This is the operator path, not a new feature build.

## 1. Build

```bash
cargo build --release
# optional (puts `kurultai` on PATH): cargo install --path . --locked
BIN=./target/release/kurultai   # or just `kurultai` after install
```

## 2. Init (Cursor MCP)

```bash
$BIN init --agent cursor
```

Creates `~/.config/kurultai/config.toml` if missing (empty sources) and wires Cursor MCP to `$BIN mcp` (or `kurultai mcp` after install).

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
$BIN index --full
$BIN status
```

Incremental later:

```bash
$BIN index
```

## 5. Query

```bash
$BIN search "database migration" --limit 10
$BIN ask "what deployments are we running?"
$BIN who-knows "sqlite"
```

## 6. Agents + daemon

```bash
# MCP stdio (Cursor / agents) — usually via init wiring
$BIN mcp

# HTTP + incremental poll + filesystem watch (markdown/github roots)
$BIN daemon --port 8421
# $BIN daemon --no-poll
# $BIN daemon --no-watch
```

## Soft-fail notes

- Missing Dayflow DB / Pond binary → clear connector error; other sources still index.
- Watch covers markdown + github roots; Pond/Dayflow stay poll-interval based.
- Daemon binds localhost; do not expose without auth (not shipped).

## More

- Example config: [`examples/config.solo.toml`](../examples/config.solo.toml)
- Phase 4 wrap: [`docs/plans/phase-4-complete.md`](plans/phase-4-complete.md)
- Master plan: [#27](https://github.com/duketopceo/kurultai/issues/27)
