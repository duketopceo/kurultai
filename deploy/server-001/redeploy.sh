#!/usr/bin/env bash
# Redeploy the solo Kurultai instances (personal + work) on server-001.
#
#   redeploy.sh                 build kurultai:solo from this checkout, swap
#   redeploy.sh --pull v0.7.1   pull ghcr.io/duketopceo/kurultai:v0.7.1, swap
#   redeploy.sh backup          back up the SQLite volumes only
#   redeploy.sh rollback        retag kurultai:solo-prev -> kurultai:solo, swap
#
# Swap = stop containers -> back up /data volumes -> compose up -> health gate.
# If the gate fails, the previous image is restored automatically (data is NOT
# rolled back; restore from the backup, see docs/deploy.md).
#
# Always run on the server from the repo root. Secrets come from the
# server-local .env (never committed); it is passed with --env-file.
# Env knobs: KURULTAI_ENV_FILE, KURULTAI_BACKUP_DIR, KEEP_BACKUPS (7),
# HEALTH_TIMEOUT seconds (120), SERVICES, KURULTAI_IMAGE_REPO.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
COMPOSE="$ROOT/deploy/server-001/docker-compose.kurultai.yml"
ENV_FILE="${KURULTAI_ENV_FILE:-$ROOT/.env}"
BACKUP_DIR="${KURULTAI_BACKUP_DIR:-$HOME/kurultai-backups}"
KEEP_BACKUPS="${KEEP_BACKUPS:-7}"
HEALTH_TIMEOUT="${HEALTH_TIMEOUT:-120}"
IMAGE_REPO="${KURULTAI_IMAGE_REPO:-ghcr.io/duketopceo/kurultai}"
SERVICES="${SERVICES:-kurultai-personal kurultai-work landing}"
SOLO_CONTAINERS="kurultai-personal kurultai-work"
# Backups use a pinned base image already used by the runtime stage.
BACKUP_IMAGE="debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc"

log() { printf '==> %s\n' "$*"; }
die() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }
dc() { docker compose -f "$COMPOSE" --env-file "$ENV_FILE" "$@"; }

preflight() {
  command -v docker >/dev/null || die "docker not found"
  [[ -f "$ENV_FILE" ]] || die "missing $ENV_FILE (server-local secrets file)"
  [[ -f "$COMPOSE" ]] || die "missing $COMPOSE"
  mkdir -p /home/khan/kurultai-repos "$BACKUP_DIR"
}

# Name of the volume mounted at /data in a container (empty if no container).
data_volume() {
  docker inspect "$1" --format '{{range .Mounts}}{{if eq .Destination "/data"}}{{.Name}}{{end}}{{end}}' 2>/dev/null || true
}

# Cold backup (containers must be stopped so SQLite WAL is checkpointed/idle).
backup_volumes() {
  local ts dest c vol
  ts="$(date -u +%Y%m%dT%H%M%SZ)"
  dest="$BACKUP_DIR/$ts"
  mkdir -p "$dest"
  for c in $SOLO_CONTAINERS; do
    vol="$(data_volume "$c")"
    if [[ -z "$vol" ]]; then
      log "backup: $c has no /data volume yet (first deploy), skipping"
      continue
    fi
    log "backup: $vol -> $dest/$vol.tar.gz"
    docker run --rm -v "$vol":/data:ro -v "$dest":/backup "$BACKUP_IMAGE" \
      tar -czf "/backup/$vol.tar.gz" -C /data .
  done
  echo "$dest" > "$BACKUP_DIR/LATEST"
  # prune: keep the newest $KEEP_BACKUPS timestamped dirs
  # shellcheck disable=SC2012
  ls -1d "$BACKUP_DIR"/2* 2>/dev/null | sort -r | tail -n +"$((KEEP_BACKUPS + 1))" | xargs -r rm -rf
  log "backup done: $dest"
}

# 0 when every solo container reports healthy within HEALTH_TIMEOUT.
health_gate() {
  local deadline c status ok
  deadline=$((SECONDS + HEALTH_TIMEOUT))
  while ((SECONDS < deadline)); do
    ok=1
    for c in $SOLO_CONTAINERS; do
      status="$(docker inspect "$c" --format '{{if .State.Running}}{{if .State.Health}}{{.State.Health.Status}}{{else}}running{{end}}{{else}}down{{end}}' 2>/dev/null || echo missing)"
      [[ "$status" == "healthy" ]] || ok=0
    done
    if ((ok)); then log "health: all solo instances healthy"; return 0; fi
    sleep 3
  done
  for c in $SOLO_CONTAINERS; do
    log "health: $c = $(docker inspect "$c" --format '{{.State.Status}}/{{if .State.Health}}{{.State.Health.Status}}{{end}}' 2>/dev/null || echo missing)"
  done
  return 1
}

swap() {
  # shellcheck disable=SC2086
  dc stop $SOLO_CONTAINERS 2>/dev/null || true
  backup_volumes
  # shellcheck disable=SC2086
  dc up -d --force-recreate $SERVICES
}

rollback_image() {
  docker image inspect kurultai:solo-prev >/dev/null 2>&1 || die "no kurultai:solo-prev to roll back to"
  log "rollback: kurultai:solo-prev -> kurultai:solo"
  docker tag kurultai:solo-prev kurultai:solo
  # shellcheck disable=SC2086
  dc up -d --force-recreate $SERVICES
}

report() {
  docker exec kurultai-personal kurultai --version || true
  docker exec kurultai-personal kurultai status 2>&1 | head -25 || true
}

cmd="deploy"
pull_tag=""
while (($#)); do
  case "$1" in
    deploy | backup | rollback) cmd="$1" ;;
    --pull) shift; pull_tag="${1:?--pull needs a tag, e.g. v0.7.1}" ;;
    -h | --help) sed -n '2,12p' "$0"; exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
  shift
done

preflight

case "$cmd" in
  backup)
    # shellcheck disable=SC2086
    dc stop $SOLO_CONTAINERS
    backup_volumes
    # shellcheck disable=SC2086
    dc up -d $SERVICES
    health_gate
    ;;
  rollback)
    rollback_image
    health_gate || die "still unhealthy after rollback; restore data per docs/deploy.md"
    report
    ;;
  deploy)
    if docker image inspect kurultai:solo >/dev/null 2>&1; then
      log "keeping current image as kurultai:solo-prev"
      docker tag kurultai:solo kurultai:solo-prev
    else
      log "no existing kurultai:solo (first deploy): rollback unavailable"
    fi
    sha="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
    if [[ -n "$pull_tag" ]]; then
      log "pull $IMAGE_REPO:$pull_tag"
      docker pull "$IMAGE_REPO:$pull_tag"
      docker tag "$IMAGE_REPO:$pull_tag" kurultai:solo
      docker tag "$IMAGE_REPO:$pull_tag" "kurultai:solo-$pull_tag"
    else
      log "build kurultai:solo @ $sha"
      DOCKER_BUILDKIT=1 docker build \
        -f "$ROOT/deploy/server-001/Dockerfile.solo" \
        --build-arg "REVISION=$sha" --build-arg "VERSION=$sha" \
        -t kurultai:solo -t "kurultai:solo-$sha" "$ROOT"
    fi
    log "swap"
    swap
    if health_gate; then
      report
      echo "next: deploy/server-001/sync-and-reindex-repos.sh"
    else
      log "UNHEALTHY after deploy: rolling back image"
      if rollback_image && health_gate; then
        die "deploy failed health gate; previous image restored (data backup: $(cat "$BACKUP_DIR/LATEST" 2>/dev/null))"
      fi
      die "deploy AND rollback unhealthy; see docs/deploy.md (restore data from $(cat "$BACKUP_DIR/LATEST" 2>/dev/null))"
    fi
    ;;
esac
