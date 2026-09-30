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

Public DNS/Access is operator-verified: no CF API token in omaseal covers
the zone (audit 2026-09-26), so these steps are manual in the dashboard.

1. **Hostname** — pick before creating DNS. Candidates:
   `demo.kurultai.dev`, `kurultai-demo.luke-the-duke.com`. Do not reuse
   `knowledge.shippedit.dev` — that's the real personal instance.
2. **Tunnel** — Zero Trust → Networks → Tunnels → route the chosen
   hostname to `http://localhost:8429` on the host running the demo
   container. (Or a named-tunnel `ingress` entry if the host already runs
   cloudflared — see `deploy/server-001/` in kurultai-private.)
3. **Access application** — Zero Trust → Access → Applications →
   self-hosted, same hostname. Policy: email OTP or the IdP already in
   use; keep it 1Password-friendly per repo preference. Copy the app
   `aud` tag.
4. **Daemon verification** — set on the service environment:
   `KURULTAI_CF_ACCESS_TEAM=<team>.cloudflareaccess.com` and
   `KURULTAI_CF_ACCESS_AUDS=<aud-tag>` — `src/http/cf_access.rs` then
   verifies `Cf-Access-Jwt-Assertion` so humans sign in via Access instead
   of pasting a key.
5. **Service token** (optional, for agent/agentic access) — Access →
   Service auth → mint, store as omaseal `cloudflare/demo-access`.
6. **Portfolio link** — set `demoUrl` on the kurultai entry in
   `portfolio-hub/src/data/demos.ts` once the hostname is live; the card
   already renders "Access-gated".

## Deferred

Real corpus seeding beyond fixtures, unauthenticated public tier,
auto-provisioning, `kurultai connect` device flow (kurultai#332).
