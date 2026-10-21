import "server-only";

/**
 * Server-side client for the hub daemon's admin surface (`/api/hub/*`).
 *
 * U8/R14 — every call carries an `is_admin` hub key in the Authorization
 * header. The key is read from process env on the server only; this module
 * must never be imported by a client component (the `server-only` import
 * makes that a build error).
 *
 * Env:
 *   KURULTAI_HUB_API_URL — base URL of the hub daemon, e.g.
 *                          https://api-hub.shippedit.dev or http://127.0.0.1:8423
 *   KURULTAI_ADMIN_KEY   — `is_admin` hub key (`kurultai hub key issue --admin`);
 *                          HUB_ADMIN_KEY accepted as a fallback name.
 */

export interface HubKeyRecord {
  id: number;
  key_prefix: string;
  agent_id: string;
  team_id: string;
  is_admin: boolean;
  created_at: string;
  revoked_at: string | null;
}

export interface HubDeviceFlow {
  id: string;
  user_code: string;
  codename: string;
  instance_id: string;
  client_id: string;
  status: string;
  created_at: string;
  expires_at: string;
  approved_at: string | null;
  approved_by: string | null;
  agent_id: string | null;
  team_hint?: string;
  team_id?: string;
}

export interface HubScopeCount {
  visibility: string;
  team_id: string | null;
  atoms: number;
}

export interface HubActivityEntry {
  id: number;
  at: string;
  agent_id: string;
  team_id: string;
  namespace: string;
  transport: string;
  reason: string | null;
  atom_id: string | null;
}

/** Discriminated result — `status: null` means the daemon was unreachable. */
export type HubResult<T> =
  | { ok: true; data: T }
  | { ok: false; status: number | null; error: string };

const FETCH_TIMEOUT_MS = 8_000;

function hubConfig(): { url: string; key: string } | { error: string } {
  const url = process.env.KURULTAI_HUB_API_URL?.replace(/\/+$/, "");
  const key = process.env.KURULTAI_ADMIN_KEY ?? process.env.HUB_ADMIN_KEY;
  if (!url) {
    return { error: "KURULTAI_HUB_API_URL is not set" };
  }
  if (!key) {
    return {
      error:
        "KURULTAI_ADMIN_KEY is not set — mint one with `kurultai hub key issue --admin`",
    };
  }
  return { url, key };
}

/** Configured hub base URL for display (empty string when unset). */
export function hubApiUrl(): string {
  return process.env.KURULTAI_HUB_API_URL?.replace(/\/+$/, "") ?? "";
}

async function hubFetch<T>(
  path: string,
  init?: { method?: string; body?: unknown },
): Promise<HubResult<T>> {
  const cfg = hubConfig();
  if ("error" in cfg) {
    return { ok: false, status: null, error: cfg.error };
  }
  try {
    const res = await fetch(`${cfg.url}${path}`, {
      method: init?.method ?? "GET",
      body: init?.body === undefined ? undefined : JSON.stringify(init.body),
      cache: "no-store",
      signal: AbortSignal.timeout(FETCH_TIMEOUT_MS),
      headers: {
        authorization: `Bearer ${cfg.key}`,
        accept: "application/json",
        ...(init?.body === undefined
          ? {}
          : { "content-type": "application/json" }),
      },
    });
    const body: unknown = await res.json().catch(() => null);
    if (!res.ok) {
      const msg =
        body &&
        typeof body === "object" &&
        "error" in body &&
        typeof (body as { error: unknown }).error === "string"
          ? (body as { error: string }).error
          : `HTTP ${res.status}`;
      return { ok: false, status: res.status, error: msg };
    }
    return { ok: true, data: body as T };
  } catch (e) {
    return {
      ok: false,
      status: null,
      error: `unreachable: ${e instanceof Error ? e.message : String(e)}`,
    };
  }
}

/** `GET /api/hub/keys` — metadata only; plaintext/hash never serialize. */
export function listKeys() {
  return hubFetch<{ ok: true; keys: HubKeyRecord[] }>("/api/hub/keys");
}

/** `POST /api/hub/keys` — plaintext `key` is returned exactly once. */
export function issueKey(agentId: string, teamId: string, isAdmin: boolean) {
  return hubFetch<{ ok: true; id: number; key: string }>("/api/hub/keys", {
    method: "POST",
    body: { agent_id: agentId, team_id: teamId, is_admin: isAdmin },
  });
}

/** `DELETE /api/hub/keys/{id}` — 404 when already revoked/unknown. */
export function revokeKey(id: number) {
  return hubFetch<{ ok: true; id: number; revoked: boolean }>(
    `/api/hub/keys/${id}`,
    { method: "DELETE" },
  );
}

/** `GET /api/hub/device/pending` — pending flows first, then recent decisions. */
export function listDeviceFlows(limit = 100) {
  return hubFetch<{ ok: true; flows: HubDeviceFlow[] }>(
    `/api/hub/device/pending?limit=${limit}`,
  );
}

/**
 * `POST /api/hub/device/approve` — mints the team-bound member key for the
 * polling CLI (the plaintext goes to the device, never to this console).
 * 400 = invalid team_id; 409 = unknown/already-decided/expired code.
 */
export function approveDeviceFlow(code: string, teamId: string) {
  return hubFetch<{ ok: true; code: string; team_id: string | null }>(
    "/api/hub/device/approve",
    { method: "POST", body: { code, team_id: teamId } },
  );
}

/** `GET /api/hub/scopes` — atom census by visibility + team_id. */
export function listScopes() {
  return hubFetch<{ ok: true; scopes: HubScopeCount[] }>("/api/hub/scopes");
}

/** `GET /api/hub/activity?limit` — admin-only hub write/admin audit feed. */
export function listActivity(limit = 50) {
  return hubFetch<{ ok: true; entries: HubActivityEntry[] }>(
    `/api/hub/activity?limit=${limit}`,
  );
}

/** Distinct team ids for datalists — union of key teams, census teams, hints. */
export function collectTeams(
  keys: HubKeyRecord[] | undefined,
  scopes: HubScopeCount[] | undefined,
  flows: HubDeviceFlow[] | undefined,
): string[] {
  const teams = new Set<string>();
  for (const k of keys ?? []) teams.add(k.team_id);
  for (const s of scopes ?? []) if (s.team_id) teams.add(s.team_id);
  for (const f of flows ?? []) {
    if (f.team_hint) teams.add(f.team_hint);
    if (f.team_id) teams.add(f.team_id);
  }
  return [...teams].filter(Boolean).sort();
}
