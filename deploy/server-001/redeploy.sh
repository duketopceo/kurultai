#!/usr/bin/env bash
# Rebuild kurultai:solo from current checkout and recreate the compose stack.
# Run on server-001 from the kurultai repo root (public main + deploy/ present).
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

docker exec kurultai-personal kurultai --version || true
docker exec kurultai-personal kurultai status 2>&1 | head -25 || true

echo "next: deploy/server-001/sync-and-reindex-repos.sh"
