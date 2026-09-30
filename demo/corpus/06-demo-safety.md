---
tags: [demo, safety, access]
---

# Demo instance boundaries

This instance exists to let visitors poke a live kurultai without touching
real data:

- Isolated SQLite store (`~/.local/share/kurultai/demo/store.db`).
- Corpus = this fixture directory only; no personal notes, git history,
  or connector secrets are reachable.
- Intended to sit behind Cloudflare Tunnel + Access — the daemon already
  verifies `Cf-Access-Jwt-Assertion` when `KURULTAI_CF_ACCESS_TEAM` /
  `KURULTAI_CF_ACCESS_AUDS` are set.
- Loopback-bound by default; the only public path is the tunnel.
