# Crew instance on one box (node3-fedora-16gb)

Runbook for a **shared** kurultai instance serving N Claude Code sessions that
all run on one machine as **one unix user**.

Scope: Track A only. Not the hub, not multi-tenant governance, not the Brain UI.

## The constraint that shapes everything

Every session runs as the same unix user. Any session can already read every
other session's files and `/proc`. Per-agent ACLs would therefore be theatre.
What is actually buyable on this box is:

1. **One store** the sessions share, opened safely by many processes at once.
2. **Closed write paths** — a local process cannot silently promote its own
   writes into what other agents later read as trusted.
3. **Namespacing** — sessions do not pollute each other's recall.

This document covers (1) and the deployment. (2) and (3) are separate changes;
see "Not yet done" below.

## No daemon. Nothing binds a port.

Verified in source, not docs:

- `Commands::Mcp` (`src/main.rs`) calls `kurultai::mcp::run_stdio`, which is
  pure stdin/stdout JSON-RPC (`src/mcp/server.rs`). It opens SQLite directly
  via `App::from_config`. **Zero network involvement.** N `kurultai mcp` child
  processes each open `store.db` themselves.
- The only other MCP entry is `src/http/mcp.rs`, reachable only through the
  daemon, and it is `ToolSurface::ReadOnly`.
- `kurultai daemon` binds unconditionally: `src/daemon/mod.rs` awaits
  `http::serve_with` as its main body, and `src/http/mod.rs` does
  `TcpListener::bind`. `--no-poll` / `--no-watch` disable the *loops*, not the
  listener. There is no flag to run the daemon without HTTP.
- Background indexing is not exclusive to the daemon. `Commands::Index` is a
  standalone one-shot over the same pipeline.

**So: run no daemon.** A systemd *user* timer invoking `kurultai index` gives
incremental indexing with nothing listening on any port.

The only CLI feature that degrades is `kurultai status --metrics`, which needs
the daemon's HTTP endpoint. It degrades gracefully (reports unreachable).

## Store: one SQLite file, WAL

`~/.local/share/kurultai/store.db`, shared by every session.

The crew shape is N independent OS processes on one file. `SqliteVecStore`'s
internal `Mutex<Connection>` only serializes *within* one process, so the
database itself has to be configured for cross-process use. `SqliteVecStore::open`
now does that (`configure_multiprocess`, `src/store/mod.rs`):

| pragma | value | why |
| --- | --- | --- |
| `journal_mode` | `WAL` | Persisted in the DB header. Without it, the default `DELETE` journal makes a single writer take an exclusive lock over the whole file — every concurrent `search` blocks for the duration of an index run. Read back and verified, because `journal_mode` is a query pragma and a failed switch is otherwise silent. |
| `busy_timeout` | 5000ms | Per-connection, re-applied every open. Override with `KURULTAI_SQLITE_BUSY_TIMEOUT_MS`. |
| `synchronous` | `NORMAL` | Standard companion to WAL. |

WAL creates `store.db-wal` and `store.db-shm` sidecars. Two consequences:

- The **directory** holding `store.db` must be writable, not just the file.
- Anything that replaces `store.db` by a file copy must delete the sidecars, or
  SQLite replays the old WAL onto the new database. `import --replace` now calls
  `store::remove_wal_sidecars`. `export` uses the `rusqlite` backup API, which
  handles WAL correctly and needs no change.

`docs/multi-user-kurultai.md`'s warning about sharing a store over
Dropbox/iCloud still stands, and is now more true, not less: WAL does not work
over network filesystems. Local disk only.

### Do not put the store (or a build dir) on `tmpfs`

On this box `/tmp` is a tmpfs — files there consume RAM. On a 16GB machine
already running N sessions, a Rust `target/` directory under `/tmp` is enough
to exhaust memory and get processes OOM-killed. Keep `store.db` and any build
output on real disk under `/home`.

## Config

One config, at the default location `~/.config/kurultai/config.toml`
(`config_path()`, `src/config/loader.rs`, honours `$KURULTAI_CONFIG` first).

```toml
environment = "prod"

[storage]
# Explicit. Without this key the env-derived default wins regardless of
# `environment` — see file_to_runtime() in src/config/loader.rs.
path = "/home/<user>/.local/share/kurultai/store.db"

[embed]
model = "openai/text-embedding-3-large"
dimension = 3072

[runtime]
poll_interval_secs = 300
```

`environment = "prod"` is safe with no embedding API key on the box.
`Environment::requires_embed_api_key()` returns true for prod but **has no call
sites** — it is dead code. The live path is `build_embedder`
(`src/app/context.rs`), which warns and falls back to `NullEmbedder`. With no
key you get an FTS-only brain: `search`, `who_knows`, and extractive `ask` all
work.

## Initial sources: curated and small

**Do not point a source at a repo root.** `kind = "markdown"` does not mean
markdown. `MarkdownConnector::collect_atoms` calls `walk_dump_files(root, &[], …)`
— the exclude list is *empty* — and `detect_format` (`src/ingest/dump.rs`)
accepts `.md`, `.markdown`, `.json`, `.jsonl`, `.ndjson`, `.txt`. Only
dot-directories are skipped. A source rooted at a repo therefore ingests
`package.json`, every lockfile, and the entire contents of `node_modules/` and
`target/` as thousands of JSON atoms. `kurultai prune --generated` exists
because this has already bitten someone, and its pattern list is Next.js
specific — it will not catch `node_modules` or `target`.

Start with three or four doc directories, a few hundred atoms:

```toml
[sources.it_docs]
enabled = true
kind = "markdown"
root_path = "/home/<user>/dev/repos/<org>/kb-it-docs"
default_corpus_tier = "private"
default_visibility_labels = "it"

[sources.it_kb]
enabled = true
kind = "markdown"
root_path = "/home/<user>/dev/repos/<org>/it-knowledge-base"
default_corpus_tier = "private"
default_visibility_labels = "it"

[sources.kurultai_docs]
enabled = true
kind = "markdown"
root_path = "/home/<user>/dev/repos/<org>/kurultai/docs"
```

Before enabling any root, check it:

```sh
find "$ROOT" \( -name node_modules -o -name target -o -name '*.json' \
              -o -name '*.txt' -o -name '*.jsonl' \) -not -path '*/.git/*' | head
```

Empty output means the root is safe for a `markdown` source.

**Source roots must not move.** `App::from_config` builds the connector
registry on *every* command including `mcp`, and connector init calls
`validate_readable_path`, which errors when the root does not exist. One
deleted or renamed source directory therefore breaks **every session's MCP
server at startup**, not just indexing.

## Install and wire

```sh
# 1. Install to a stable path — NOT the target/debug build.
#    resolve_kurultai_bin() prefers current_exe(), so running `init` from
#    target/debug pins ~/.claude.json to a build artifact.
scripts/install.sh                     # defaults INSTALL_DIR=~/.local/bin

# 2. Write the config (create_new — never clobbers an existing one),
#    then edit in the sources above.
~/.local/bin/kurultai init

# 3. First index. Confirm the atom count is in the hundreds, not thousands.
~/.local/bin/kurultai index --full
~/.local/bin/kurultai status
```

Wiring writes a single `mcpServers.kurultai` entry into `~/.claude.json`:

```json
{ "command": "/home/<user>/.local/bin/kurultai", "args": ["mcp"], "type": "stdio" }
```

The merge is safe: sibling `mcpServers` entries are preserved, malformed JSON
is refused rather than overwritten, and the write is atomic at `0600`
(`wire_json_mcp_at`, `src/mcp/init.rs`).

Every session picks this up on its next start. Confirm with `/mcp` in a
session, or by hand:

```sh
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \
  | ~/.local/bin/kurultai mcp
```

## Incremental indexing without a daemon

`~/.config/systemd/user/kurultai-index.service`:

```ini
[Unit]
Description=Kurultai incremental index

[Service]
Type=oneshot
ExecStart=%h/.local/bin/kurultai index
```

`~/.config/systemd/user/kurultai-index.timer`:

```ini
[Unit]
Description=Run kurultai index every 15 minutes

[Timer]
OnBootSec=5min
OnUnitActiveSec=15min
Persistent=true

[Install]
WantedBy=timers.target
```

```sh
systemctl --user daemon-reload
systemctl --user enable --now kurultai-index.timer
loginctl enable-linger "$USER"   # so the timer survives logout
```

Under WAL this runs concurrently with live sessions without blocking their
reads.

## Not yet done — known gaps in the crew story

These are real and verified in source. They are not addressed by this document
and each needs its own change.

- **Any session can promote its own writes.** `run_stdio` hands
  `ToolSurface::Full` to every message, so every session gets `remember`,
  `promote`, and `ontology_promote`. The quality gate quarantines untagged /
  short / boilerplate writes, but the same session can then call `promote` on
  the atom it just wrote; self-promotion is unrestricted. The closed-write-path
  shape is a third `ToolSurface` where the agent may only `remember` into
  quarantine, with `promote` living only in the CLI (`Commands::Promote`,
  which already exists and works) as a deliberate operator act.
- **All sessions' memories share one namespace.** `remember` passes an empty
  metadata slice (`src/mcp/server.rs`), so nothing sets `project_id` and every
  session's atoms land in one undifferentiated `agent` source with no session
  tag. `KnowledgeAtom::project_id()` and `BrainService::recall_for_agent` exist,
  but `recall_for_agent` is not exposed as an MCP tool and filters in memory
  after over-fetching — its own doc comment says the project filter belongs in
  SQL. `SearchFilter` is `{ trusted_only }` and nothing else; no project,
  visibility, or label predicate reaches SQL anywhere.
- **One `~/.claude.json` serves every session.** `wire_json_mcp_at` cannot emit
  an `env` block, and there is one shared `mcpServers.kurultai` entry, so
  per-session identity (a namespace) and per-session tool surface cannot be
  expressed through `init` as written. Either add an `env` map parameter to
  `wire_agent`/`wire_json_mcp_at`, or hand-write distinct per-session entries
  that `init` will not manage.
- **Connector init failure aborts `kurultai mcp` startup**, taking down every
  session at once. Making connector init non-fatal for the `mcp` command is a
  genuine robustness fix.

## Open decisions

Two questions are unresolved and are **not** answered here, deliberately.

1. **Which store backend suits one box with N sessions.** The Postgres store is
   gated behind a `postgres` cargo feature *plus* `KURULTAI_FEATURE_HUB=1`, and
   is commented "One database per organization" — that is hub framing, not
   local-shared framing. This runbook assumes shared SQLite because that is
   what runs today with no feature flags; it is not an argument that SQLite is
   the right long-term answer.
2. **Whether a namespace is per-session, per-project, or both, and whether
   cross-namespace reads default open or closed.** Nothing here picks. The
   `remember`-side seam is `RememberArgs` plus the `brain.remember` call in
   `src/mcp/server.rs`; the read-side seam is `SearchFilter` in
   `src/store/mod.rs`.
