#!/usr/bin/env bash
set -euo pipefail
ENV_FILE="/home/khan/kurultai/.env"

gen_key() { openssl rand -hex 32; }

# Preserve Cloudflare tokens if already set
if [[ -f "$ENV_FILE" ]]; then
  set -a
  # shellcheck source=/dev/null
  source "$ENV_FILE" 2>/dev/null || true
  set +a
fi

if [[ "${CLOUDFLARE_TUNNEL_TOKEN:-}" == "your_tunnel_token_here" || -z "${CLOUDFLARE_TUNNEL_TOKEN:-}" ]]; then
  CLOUDFLARE_TUNNEL_TOKEN=""
fi
if [[ "${CLOUDFLARE_API_TOKEN:-}" == "your_api_token_here" || -z "${CLOUDFLARE_API_TOKEN:-}" ]]; then
  CLOUDFLARE_API_TOKEN=""
fi

cat > "$ENV_FILE" <<ENV
# Kurultai two-instance secrets — generated on $(date -Iseconds)
CF_ACCOUNT_ID=1661907b2d7e4a20800306e6a57844c5
CF_TUNNEL_ID=16ff6454-f890-40ab-ae8d-3632ed23ee2d
CF_ZONE_NAME=shippedit.dev

# Cloudflare connector token (from Zero Trust dashboard or API)
CLOUDFLARE_TUNNEL_TOKEN=${CLOUDFLARE_TUNNEL_TOKEN:-your_tunnel_token_here}

# Cloudflare API token with Zone:Edit and Cloudflare Tunnel:Edit for shippedit.dev
CLOUDFLARE_API_TOKEN=${CLOUDFLARE_API_TOKEN:-your_api_token_here}

PERSONAL_API_KEYS=${PERSONAL_API_KEYS:-$(gen_key)}
WORK_API_KEYS=${WORK_API_KEYS:-$(gen_key)}
PERSONAL_MCP_SECRET=${PERSONAL_MCP_SECRET:-$(gen_key)}
WORK_MCP_SECRET=${WORK_MCP_SECRET:-$(gen_key)}
PERSONAL_INGEST_SECRET=${PERSONAL_INGEST_SECRET:-$(gen_key)}
WORK_INGEST_SECRET=${WORK_INGEST_SECRET:-$(gen_key)}
ENV

echo "$ENV_FILE written"
