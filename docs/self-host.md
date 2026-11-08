# Self-host Kurultai

Run your own brain in a few minutes. No API keys are required: keyword (FTS5)
search works out of the box, and you can add meaning-based search later.

## Quick start (Docker Compose)

```bash
git clone https://github.com/duketopceo/kurultai.git && cd kurultai
docker compose up -d
docker compose logs kurultai | rg -A1 "ACCESS KEY"   # or: docker compose logs kurultai | grep -A1 "ACCESS KEY"
```

Open <http://127.0.0.1:8421/ui/>. An empty brain opens the welcome page, which walks
you through a first note, importing a folder, and choosing embeddings. The access key is
printed once on first start and stored at `/data/.local/share/kurultai/selfhost.key` in
the volume (`docker exec kurultai cat /data/.local/share/kurultai/selfhost.key`).
Paste it where the UI asks for an access key.

Without Compose:

```bash
docker run -d --name kurultai --restart unless-stopped \
  -p 127.0.0.1:8421:8421 -v kurultai-data:/data \
  -e KURULTAI_SELFHOST=1 \
  ghcr.io/duketopceo/kurultai:latest daemon
```

`KURULTAI_SELFHOST=1` makes the first start write a config, point `[sources.notes]` at
`/data/notes`, and mint the access key (a container must listen on a non-loopback
address, and the daemon refuses that without auth). It changes nothing for non-container
installs. The port is published on loopback only; see "Remote access" to expose it.

## Import your notes

Mount a folder of Markdown files at `/data/notes` (uncomment the volume line in
`docker-compose.yml`, or `-v ~/notes:/data/notes:ro`). It is indexed on start and watched
for changes. A file needs at least one tag (YAML `tags:` or a `#tag` line) or it lands in
quarantine and is left out of default search.

## Embeddings (search by meaning)

| Choice | How | Needs |
|--------|-----|-------|
| Keywords only (default) | nothing | nothing |
| Local, offline | `docker compose -f docker-compose.yml -f docker-compose.local-embed.yml up -d --build` | builds the image with ONNX (`--features local-embed`); downloads a ~90 MB model once, then works offline |
| OpenRouter | `OPENROUTER_API_KEY=...` in `.env`, `KURULTAI_EMBED_BACKEND=openrouter`, or `kurultai connect openrouter` | an OpenRouter key (it must have an expiration date) |

Choose before you index much: the store's vector size is fixed at first index, so
switching embedders later needs a new volume (or `storage.path`). The code supports the
local model already; it is not the default image because it adds a large native
dependency. Explicit `KURULTAI_EMBED_BACKEND=local` always wins over a key that happens
to be set.

All variables are listed and commented in [`.env.example`](../.env.example).

## Remote access, TLS

The compose file publishes `127.0.0.1` only. Pick one:

- **Cloudflare Tunnel** (no open ports): `deploy/selfhost/cloudflared.compose.example.yml`.
  Public hostname to `http://kurultai:8421`. Add a Cloudflare Access app and set
  `KURULTAI_CF_ACCESS_TEAM` / `KURULTAI_CF_ACCESS_AUDS` for SSO browser logins.
- **Caddy** (automatic HTTPS): `deploy/selfhost/Caddyfile.example`.
- **Tailscale**: `tailscale serve 8421` on the host, or set `KURULTAI_HUB_BIND=tailscale`.

Keep the access key. Anything reachable from the internet must stay behind it (or Access).

## Connect agents

```bash
kurultai connect http://127.0.0.1:8421          # browser-approved device key + MCP wiring
```

or point an MCP client at `http://<host>/mcp` with `KURULTAI_MCP_HTTP_SECRET` (read-only).

## Backups and restore

All state is in the `kurultai-data` volume (SQLite store, config, key, model cache).

```bash
docker compose stop kurultai
docker run --rm -v kurultai_kurultai-data:/data:ro -v "$PWD":/backup debian:bookworm-slim \
  tar -czf /backup/kurultai-$(date +%F).tar.gz -C /data .
docker compose start kurultai
```

Restore into an empty volume:

```bash
docker compose down
docker volume rm kurultai_kurultai-data && docker volume create kurultai_kurultai-data
docker run --rm -v kurultai_kurultai-data:/data -v "$PWD":/backup debian:bookworm-slim \
  sh -c 'tar -xzf /backup/kurultai-YYYY-MM-DD.tar.gz -C /data && chown -R 1000:1000 /data'
docker compose up -d
```

(Check the real volume name with `docker volume ls`; Compose prefixes it with the project
name `kurultai`.) Stopping first keeps the SQLite WAL consistent.

## Upgrade

```bash
docker compose pull && docker compose up -d      # or pin KURULTAI_TAG=v0.7.1 in .env
```

Back up first. If the new version misbehaves, set `KURULTAI_TAG` to the previous tag and
restore the backup if the schema moved. Releases are published as
`ghcr.io/duketopceo/kurultai:<version>` on every `vX.Y.Z` tag.

## Troubleshooting

- *"non-loopback bind with hub.auth=none"*: you ran the image without
  `KURULTAI_SELFHOST=1` or keys. Add the flag or set `KURULTAI_HUB_AUTH=api_key` and
  `KURULTAI_HUB_API_KEYS`.
- *Lost the key*: `docker exec kurultai cat /data/.local/share/kurultai/selfhost.key`.
- *No results from imported notes*: tag them (see above) or check
  `docker compose logs kurultai`.
- *Health*: `docker compose ps` should show `healthy`; `curl http://127.0.0.1:8421/health`.
