/** Human access token storage + auth probe for the Brain UI. */

import { createContext, useContext } from 'react';

export const TOKEN_KEY = 'kurultai:token';
export const TOKEN_INVALID_EVENT = 'kurultai:token-invalid';
export const AUTH_CHANGED_EVENT = 'kurultai:auth-changed';

export type AuthMode = 'unknown' | 'open' | 'locked';

export const AuthModeContext = createContext<AuthMode>('unknown');

export function useAuthMode(): AuthMode {
  return useContext(AuthModeContext);
}

export function emitAuthChanged(): void {
  window.dispatchEvent(new Event(AUTH_CHANGED_EVENT));
}

export function readToken(): string {
  try {
    return localStorage.getItem(TOKEN_KEY) || '';
  } catch {
    return '';
  }
}

export function saveToken(value: string): void {
  try {
    const trimmed = value.trim();
    if (trimmed.length > 0) {
      localStorage.setItem(TOKEN_KEY, trimmed);
    } else {
      localStorage.removeItem(TOKEN_KEY);
    }
  } catch {
    /* best-effort */
  }
}

export function clearToken(): void {
  saveToken('');
}

export function authHeaders(init?: HeadersInit): HeadersInit {
  const token = typeof window !== 'undefined' ? readToken() : '';
  const headers: Record<string, string> = { Accept: 'application/json' };
  if (token.length > 0) {
    headers.Authorization = `Bearer ${token}`;
  }
  if (init) {
    const h = new Headers(init);
    h.forEach((v, k) => {
      headers[k] = v;
    });
  }
  return headers;
}

export function emitTokenInvalid(reason?: string): void {
  clearToken();
  window.dispatchEvent(
    new CustomEvent(TOKEN_INVALID_EVENT, { detail: { reason: reason || 'unauthorized' } }),
  );
}

/** Probe whether this instance requires a human Bearer token. */
export async function probeAuthMode(signal?: AbortSignal): Promise<{
  mode: AuthMode;
  okWithStoredToken: boolean;
}> {
  const bare = await fetch('/api/status', {
    cache: 'no-store',
    headers: { Accept: 'application/json' },
    signal,
  });
  if (bare.ok) {
    return { mode: 'open', okWithStoredToken: true };
  }
  if (bare.status !== 401 && bare.status !== 403) {
    // Daemon down / network — treat as open so local UI can still boot and show offline.
    return { mode: 'open', okWithStoredToken: false };
  }

  const token = readToken();
  if (!token) {
    return { mode: 'locked', okWithStoredToken: false };
  }
  const withToken = await fetch('/api/status', {
    cache: 'no-store',
    headers: authHeaders(),
    signal,
  });
  if (withToken.ok) {
    return { mode: 'locked', okWithStoredToken: true };
  }
  clearToken();
  return { mode: 'locked', okWithStoredToken: false };
}
