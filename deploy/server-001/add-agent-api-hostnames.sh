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

ACCOUNT_ID="${CF_ACCOUNT_ID:?Set CF_ACCOUNT_ID in deploy/server-001/.env}"
TUNNEL_ID="${CF_TUNNEL_ID:?Set CF_TUNNEL_ID in deploy/server-001/.env}"
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
  CURRENT=$(CFG="$CURRENT" NEW_HOSTNAME="$hostname" NEW_SERVICE="$service" python3 <<'PY'
import json, os
cfg = json.loads(os.environ["CFG"])
ingress = cfg.get("ingress", [])
ingress = [r for r in ingress if r.get("service") != "http_status:404"]
ingress.append({
    "hostname": os.environ["NEW_HOSTNAME"],
    "service": os.environ["NEW_SERVICE"],
    "originRequest": {
        "httpHostHeader": os.environ["NEW_HOSTNAME"],
        "noTLSVerify": True
    }
})
ingress.append({"service": "http_status:404"})
print(json.dumps({"config": {"ingress": ingress}}))
PY
)
}

add_host "api-knowledge.${ZONE_NAME}" "http://kurultai-personal:8421"
add_host "api-work.${ZONE_NAME}" "http://kurultai-work:8422"

api PUT "/accounts/${ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/configurations" \
  --data "$CURRENT" >/dev/null
echo "    tunnel config updated"

cname_target="${TUNNEL_ID}.cfargotunnel.com"
for hostname in "api-knowledge.${ZONE_NAME}" "api-work.${ZONE_NAME}"; do
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
echo "  curl -fsS https://api-knowledge.${ZONE_NAME}/api/status"
echo "  curl -fsS https://api-work.${ZONE_NAME}/api/status"
