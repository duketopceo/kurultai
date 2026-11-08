/**
 * "Connect OpenRouter" via the first-party `@openrouter/sdk` (OAuth PKCE, browser side).
 * Loaded with a dynamic import so the SDK stays out of the main bundle. The key is posted
 * to the daemon and is never logged or rendered.
 */
import { authHeaders } from './auth';

const VERIFIER_KEY = 'kurultai:or-verifier';
export const CONNECT_PARAM = 'or_connect';

export type ConnectResult =
  | { kind: 'none' }
  | { kind: 'ok'; restartRequired: boolean }
  | { kind: 'error'; message: string };

function callbackUrl(): string {
  return `${window.location.origin}/ui/?${CONNECT_PARAM}=1`;
}

async function sdk() {
  const { OpenRouter } = await import('@openrouter/sdk');
  return new OpenRouter();
}

export async function startConnect(): Promise<void> {
  const or = await sdk();
  const { codeChallenge, codeVerifier } = await or.oAuth.createSHA256CodeChallenge();
  sessionStorage.setItem(VERIFIER_KEY, codeVerifier);
  const url = await or.oAuth.createAuthorizationUrl({
    callbackUrl: callbackUrl(),
    codeChallenge,
    codeChallengeMethod: 'S256',
  });
  window.location.assign(url);
}

function stripQuery(): void {
  const u = new URL(window.location.href);
  u.searchParams.delete(CONNECT_PARAM);
  u.searchParams.delete('code');
  window.history.replaceState(null, '', u.pathname + (u.search || '') + u.hash);
}

/** True when the URL asks the app to begin the flow (`?or_connect=start`). */
export function connectStartRequested(): boolean {
  return new URLSearchParams(window.location.search).get(CONNECT_PARAM) === 'start';
}

export async function finishConnectIfReturning(): Promise<ConnectResult> {
  const q = new URLSearchParams(window.location.search);
  if (q.get(CONNECT_PARAM) !== '1') return { kind: 'none' };
  const code = q.get('code');
  let verifier: string | null = null;
  try { verifier = sessionStorage.getItem(VERIFIER_KEY); } catch { /* blocked */ }
  stripQuery();
  if (!code) return { kind: 'error', message: 'OpenRouter authorization was cancelled.' };
  if (!verifier) return { kind: 'error', message: 'No pending connect found; start again.' };
  try {
    const or = await sdk();
    const res = await or.oAuth.exchangeAuthCodeForAPIKey({
      requestBody: { code, codeVerifier: verifier, codeChallengeMethod: 'S256' },
    });
    const r = await fetch('/api/openrouter/key', {
      method: 'POST',
      headers: authHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({ key: res.key }),
    });
    if (!r.ok) return { kind: 'error', message: `Daemon refused the key (${r.status}).` };
    const body = (await r.json()) as { restart_required?: boolean };
    return { kind: 'ok', restartRequired: body.restart_required !== false };
  } catch {
    return {
      kind: 'error',
      message: 'OpenRouter exchange failed. If your account requires key expiration, disable that (OAuth keys carry none).',
    };
  } finally {
    try { sessionStorage.removeItem(VERIFIER_KEY); } catch { /* ignore */ }
  }
}

export async function fetchKeyPresent(): Promise<boolean> {
  try {
    const r = await fetch('/api/onboarding', { cache: 'no-store', headers: authHeaders() });
    if (!r.ok) return false;
    return Boolean(((await r.json()) as { openrouter_key_present?: boolean }).openrouter_key_present);
  } catch {
    return false;
  }
}
