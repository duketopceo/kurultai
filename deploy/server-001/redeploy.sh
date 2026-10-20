#!/usr/bin/env bash
# Rebuild kurultai:solo + kurultai:hub from current checkout and recreate the
# compose stack. Run on server-001 from the kurultai repo root (public main +
# deploy/ present). Hub requires the KURULTAI_HUB_* / KURULTAI_ADMIN_EMAILS /
# HUB_CF_ACCESS_AUDS vars in .env (see deploy/server-001/.env.example) — the
# compose `:?` guards fail loudly without them. See docs/runbook-hub.md.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
COMPOSE="$ROOT/deploy/server-001/docker-compose.kurultai.yml"
ENV_FILE="${KURULTAI_ENV_FILE:-$ROOT/.env}"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "ERROR: missing $ENV_FILE" >&2
  exit 1
fi

mkdir -p /home/khan/kurultai-repos

echo "==> build kurultai:solo @ $(git -C "$ROOT" rev-parse --short HEAD)"
docker build -f "$ROOT/deploy/server-001/Dockerfile.solo" -t kurultai:solo "$ROOT"

echo "==> build kurultai:hub"
docker build -f "$ROOT/deploy/server-001/Dockerfile.hub" -t kurultai:hub "$ROOT"

echo "==> compose up"
docker compose -f "$COMPOSE" --env-file "$ENV_FILE" up -d

echo "==> health"
for i in 1 2 3 4 5 6; do
  if docker exec kurultai-personal curl -fsS http://127.0.0.1:8421/health >/dev/null; then
    echo "personal healthy"
    break
  fi
  sleep 2
done

for i in 1 2 3 4 5 6; do
  if docker exec kurultai-hub curl -fsS http://127.0.0.1:8423/health >/dev/null 2>&1; then
    echo "hub healthy"
    break
  fi
  sleep 2
done

docker exec kurultai-personal kurultai --version || true
docker exec kurultai-personal kurultai status 2>&1 | head -25 || true

echo "next: deploy/server-001/sync-and-reindex-repos.sh"
