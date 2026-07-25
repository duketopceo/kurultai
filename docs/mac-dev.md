# Mac laptop — dev / debug load

Stay on **dev** until you deliberately move to staging/prod.

## Install (one line)

```bash
curl -fsSL https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.sh | bash
```

## Shell env

```bash
export KURULTAI_ENV=dev
export RUST_LOG=kurultai=debug
# optional later: OPENROUTER_API_KEY=… for embeddings / rerank / LLM ask
```

## Wire + index

```bash
kurultai init --agent cursor
# edit ~/.config/kurultai/config.toml — environment = "dev"
kurultai index --full
kurultai status
```

Dayflow (macOS): `kind = "dayflow"` when the app’s `chunks.sqlite` is present. Pond: `kind = "pond"` with `pond` on `PATH`. Notes: `kind = "markdown"` + `root_path`.

## Smoke

```bash
kurultai search "test" --limit 5
kurultai daemon --port 8421
```

Storage default: `~/.local/share/kurultai/dev/store.db`.
