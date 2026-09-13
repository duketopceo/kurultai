import * as Sentry from '@sentry/react';
import { UI_VERSION } from './version.ts';

const SENSITIVE_HEADERS = new Set(['authorization', 'cookie', 'x-api-key']);

/** Resolve DSN from Vite env (build-time). Never hardcode secrets in repo. */
export function resolveGlitchtipDsn(): string | undefined {
  const env = import.meta.env ?? {};
  const raw =
    env.VITE_KURULTAI_GLITCHTIP_DSN ??
    env.VITE_SENTRY_DSN ??
    '';
  const dsn = String(raw).trim();
  return dsn.length > 0 ? dsn : undefined;
}

/** True when a non-empty DSN string is present (pure helper for tests and init). */
export function shouldInitGlitchtip(dsn: string | undefined): boolean {
  return Boolean(dsn && String(dsn).trim());
}

function scrubEvent(event: Sentry.ErrorEvent): Sentry.ErrorEvent | null {
  const headers = event.request?.headers;
  if (headers && typeof headers === 'object') {
    for (const key of Object.keys(headers)) {
      if (SENSITIVE_HEADERS.has(key.toLowerCase())) {
        headers[key] = '[redacted]';
      }
    }
  }
  return event;
}

/** Initialize Brain UI error reporting. No-op when DSN is unset. */
export function initGlitchtip(): void {
  const dsn = resolveGlitchtipDsn();
  if (!dsn) return;

  const environment =
    (import.meta.env.VITE_KURULTAI_ENV as string | undefined)?.trim() || 'dev';

  Sentry.init({
    dsn,
    environment,
    release: `kurultai-ui@${UI_VERSION}`,
    sendDefaultPii: false,
    beforeSend: scrubEvent,
  });

  if (import.meta.env.VITE_KURULTAI_GLITCHTIP_TEST === '1') {
    Sentry.captureMessage('GlitchTip test event (Brain UI)');
  }
}
