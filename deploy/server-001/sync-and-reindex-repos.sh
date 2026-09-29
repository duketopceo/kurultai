#!/usr/bin/env bash
# Sync duketopceo org checkouts into /home/khan/kurultai-repos/duketopceo/
# then reindex the personal Kurultai instance (Brain Repos section).
set -euo pipefail

REPOS_ROOT="${KURULTAI_REPOS_ROOT:-/home/khan/kurultai-repos}"
ORG="${KURULTAI_REPOS_ORG:-duketopceo}"
CONTAINER="${KURULTAI_PERSONAL_CONTAINER:-kurultai-personal}"
# Override with space-separated list, or leave default allowlist.
DEFAULT_REPOS=(
  kurultai
  kurultai-private
  luke-agents
  Pace-Server
  Bartlett-server-001
  portfolio-hub
  dayflow-linux
  openrouter_usage
)

if [[ "${1:-}" == "--help" ]]; then
  cat <<EOF
Usage: $(basename "$0") [--pull-only|--index-only]

  Sync \$ORG repos under \$REPOS_ROOT/\$ORG/<repo>, then
  docker exec \$CONTAINER kurultai index --full

Env:
  KURULTAI_REPOS_ROOT   default $REPOS_ROOT
  KURULTAI_REPOS_ORG    default $ORG
  KURULTAI_REPOS_LIST   space-separated repo names (overrides default allowlist)
  KURULTAI_PERSONAL_CONTAINER  default $CONTAINER
EOF
  exit 0
fi

mkdir -p "$REPOS_ROOT/$ORG"
cd "$REPOS_ROOT/$ORG"

if [[ -n "${KURULTAI_REPOS_LIST:-}" ]]; then
  # shellcheck disable=SC2206
  REPOS=($KURULTAI_REPOS_LIST)
else
  REPOS=("${DEFAULT_REPOS[@]}")
fi

pull_only=0
index_only=0
case "${1:-}" in
  --pull-only) pull_only=1 ;;
  --index-only) index_only=1 ;;
  "") ;;
  *) echo "unknown arg: $1" >&2; exit 2 ;;
esac

if [[ "$index_only" -eq 0 ]]; then
  for repo in "${REPOS[@]}"; do
    url="https://github.com/${ORG}/${repo}.git"
    if [[ -d "$repo/.git" ]]; then
      echo "==> pull $ORG/$repo"
      git -C "$repo" fetch --depth=1 origin
      # Prefer main, fall back to master
      branch=$(git -C "$repo" remote show origin 2>/dev/null | awk '/HEAD branch/ {print $NF}')
      branch=${branch:-main}
      git -C "$repo" reset --hard "origin/${branch}" || git -C "$repo" reset --hard origin/main || true
    else
      echo "==> clone $ORG/$repo"
      git clone --depth=1 "$url" "$repo" || {
        echo "WARN: skip $repo (clone failed — private? set deploy key)" >&2
        continue
      }
    fi
  done
fi

if [[ "$pull_only" -eq 1 ]]; then
  echo "pull-only done under $REPOS_ROOT/$ORG"
  exit 0
fi

if ! docker ps --format '{{.Names}}' | grep -qx "$CONTAINER"; then
  echo "ERROR: container $CONTAINER not running" >&2
  exit 1
fi

echo "==> ensure config sources.repos → /data/repos"
docker exec -u 0 "$CONTAINER" mkdir -p /data/.config/kurultai
docker exec -u 0 "$CONTAINER" bash -c 'cat > /data/.config/kurultai/config.toml <<EOF
# Managed by sync-and-reindex-repos.sh — Brain Repos lattice from org checkouts.
[sources.repos]
enabled = true
kind = "github"
# Paths under here are duketopceo/<repo>/... → lattice duketopceo/<repo>
root_path = "/data/repos"

[sources.pond]
enabled = false
kind = "pond"
EOF
chown -R 1000:1000 /data/.config/kurultai'

echo "==> kurultai index --full (personal)"
docker exec "$CONTAINER" kurultai index --full

echo "done: synced ${#REPOS[@]} repos, reindexed $CONTAINER"
