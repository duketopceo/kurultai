# Kurultai demo instance

`kurultai daemon --demo` boots an isolated, fixture-only instance for
visitors — a live knowledge brain without real data.

## What it is

- `config.toml` — isolated SQLite store
  (`~/.local/share/kurultai/demo/store.db`), one markdown source pointed at
  `demo/corpus/`, 30s poll.
- `corpus/` — hand-curated, public-safe atoms. Every file needs YAML
  frontmatter `tags:` or it lands in quarantine.
- `--demo` forces loopback bind and ignores `--bind` — the only public
  path is a Cloudflare Tunnel.

Run it:

```bash
# from repo root — root_path in demo/config.toml is cwd-relative
kurultai daemon --demo --port 8429
# or containerized
docker compose -f docker-compose.demo.yml up --build
```

`KURULTAI_DEMO_CONFIG` overrides the config path.

## Public exposure — operator runbook

Hostname chosen: **`kurultai-demo.luke-the-duke.com`**. Access side is
already provisioned via API (2026-09-30):

- ✅ Access app `Kurultai Demo` (self-hosted, `kurultai-demo.luke-the-duke.com`,
  aud `60a4ad4b5baea5dd1e85ae6ae48528c20fbfc908fc585a16d70944c152f15c78`)
- ✅ Policies: `owner emails` (allow — Luke's emails), `agent service token`
  (bypass — token `5e504c19`)
- ✅ Service token minted → omaseal `cloudflare/demo-access` (secret) +
  `cloudflare/demo-access-client-id`

Remaining (manual, needs the target host):

1. **Tunnel** — route `kurultai-demo.luke-the-duke.com` →
   `http://localhost:8429` on whatever host runs the demo container
   (named-tunnel ingress entry or dashboard → Networks → Tunnels; existing
   tunnels live on this account, e.g. `shippedit-server`).
2. **DNS** — creating the tunnel route creates the CNAME automatically.
3. **Daemon env** — set on the service:
   `KURULTAI_CF_ACCESS_TEAM=duketopceo.cloudflareaccess.com` and
   `KURULTAI_CF_ACCESS_AUDS=60a4ad4b5baea5dd1e85ae6ae48528c20fbfc908fc585a16d70944c152f15c78`
   — `src/http/cf_access.rs` then verifies `Cf-Access-Jwt-Assertion`.
4. **Portfolio link** — `demoUrl` is set in portfolio-hub `demos.ts`;
   flip the card to a live link once DNS resolves.

## Deferred

Real corpus seeding beyond fixtures, unauthenticated public tier,
auto-provisioning, `kurultai connect` device flow (kurultai#332).
