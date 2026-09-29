#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env"

if [[ -f "$ENV_FILE" ]]; then
  set -a
  # shellcheck source=/dev/null
  source "$ENV_FILE"
  set +a
fi

ACCOUNT_ID="${CF_ACCOUNT_ID:-1661907b2d7e4a20800306e6a57844c5}"
TUNNEL_ID="${CF_TUNNEL_ID:-16ff6454-f890-40ab-ae8d-3632ed23ee2d}"
ZONE_NAME="${CF_ZONE_NAME:-shippedit.dev}"

if [[ -z "${CLOUDFLARE_API_TOKEN:-}" ]]; then
  echo "ERROR: Set CLOUDFLARE_API_TOKEN in .env or environment." >&2
  exit 1
fi

api() {
  local method="$1" path="$2"
  shift 2
  curl -sfS -X "$method" \
    -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
    -H "Content-Type: application/json" \
    "https://api.cloudflare.com/client/v4${path}" "$@"
}

echo "==> Resolving zone ID for ${ZONE_NAME}"
ZONE_ID=$(api GET "/zones?name=${ZONE_NAME}" | python3 -c "import sys,json; r=json.load(sys.stdin); print(r['result'][0]['id'])")
echo "    zone_id=${ZONE_ID}"

echo "==> Fetching tunnel ingress config"
CFG=$(api GET "/accounts/${ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/configurations")
CURRENT=$(echo "$CFG" | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin)['result']['config']))")

add_host() {
  local hostname="$1" service="$2"
  if echo "$CURRENT" | grep -q "\"hostname\": \"${hostname}\""; then
    echo "    ${hostname} already in ingress"
    return
  fi
  echo "==> Adding ingress rule for ${hostname} -> ${service}"
  CURRENT=$(python3 <<PY
import json
cfg = json.loads('''${CURRENT}''')
ingress = cfg.get("ingress", [])
ingress = [r for r in ingress if r.get("service") != "http_status:404"]
ingress.append({
    "hostname": "${hostname}",
    "service": "${service}",
    "originRequest": {
        "httpHostHeader": "${hostname}",
        "noTLSVerify": True
    }
})
ingress.append({"service": "http_status:404"})
print(json.dumps({"config": {"ingress": ingress}}))
PY
)
}

add_host "knowledge.${ZONE_NAME}" "http://kurultai-personal:8421"
add_host "work.${ZONE_NAME}" "http://kurultai-work:8422"

api PUT "/accounts/${ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/configurations" \
  --data "$CURRENT" >/dev/null
echo "    tunnel config updated"

cname_target="${TUNNEL_ID}.cfargotunnel.com"
for hostname in "knowledge.${ZONE_NAME}" "work.${ZONE_NAME}"; do
  EXISTING=$(api GET "/zones/${ZONE_ID}/dns_records?type=CNAME&name=${hostname}" \
    | python3 -c "import sys,json; r=json.load(sys.stdin); print(len(r.get('result',[])))")
  if [[ "$EXISTING" != "0" ]]; then
    echo "    DNS CNAME ${hostname} already exists"
  else
    echo "==> Creating CNAME ${hostname} -> ${cname_target}"
    api POST "/zones/${ZONE_ID}/dns_records" --data "$(python3 <<PY
import json
print(json.dumps({
  "type": "CNAME",
  "name": "${hostname}",
  "content": "${cname_target}",
  "proxied": True
}))
PY
)" >/dev/null
    echo "    CNAME created"
  fi
done

echo ""
echo "Done. After cloudflared connects, verify:"
echo "  curl -fsS https://knowledge.${ZONE_NAME}/api/status"
echo "  curl -fsS https://work.${ZONE_NAME}/api/status"
