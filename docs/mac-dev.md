# Mac laptop — dev / debug

Stay on **dev** until you deliberately move to staging/prod.

## Install

No binary release is published yet. Use cargo:

```bash
cargo install --git https://github.com/duketopceo/kurultai --locked
```

Or the wrapper (same cargo path until a `v*` release exists):

```bash
curl -fsSL https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.sh | bash
```

Needs [Rust](https://rustup.rs). Ensure `~/.cargo/bin` is on `PATH`.

## Shell env

```bash
export KURULTAI_ENV=dev
export RUST_LOG=kurultai=debug
# optional: OPENROUTER_API_KEY=… for embeddings / rerank / LLM ask
```

## Wire + index

```bash
kurultai init --agent cursor
# edit ~/.config/kurultai/config.toml — keep environment = "dev"
kurultai index --full
kurultai status
```

| Source | `kind` | Notes |
|--------|--------|--------|
| Notes | `markdown` | `root_path` to a `.md` folder |
| Dayflow | `dayflow` | macOS Dayflow `chunks.sqlite` |
| Pond | `pond` | `pond` on `PATH` |
| Code | `github` | local checkout `root_path` |

AppFlowy is deferred ([#4](https://github.com/duketopceo/kurultai/issues/4)).

## Smoke

```bash
kurultai search "test" --limit 5
kurultai daemon --port 8421
```

Storage default: `~/.local/share/kurultai/dev/store.db`.
