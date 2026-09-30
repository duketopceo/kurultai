// settings-access.tsx
import { useEffect, useId, useMemo, useRef, useState } from "react";
import type { ClipboardEvent, FormEvent } from "react";
import "./settings-access.css";

/* ───────────────────────── types ───────────────────────── */

export type AccessMode = "open" | "token" | "cloudflare";

export interface DaemonInfo {
  version?: string | null;
  online: boolean;
  instance?: string | null; // e.g. "Ulaanbaatar"
  uptimeSec?: number | null; // omitted/null → not shown
}

export interface PresenceSeat {
  codename: string;
  instance_id: string;
  last_seen: string | number | null; // ISO string or epoch ms
}

export interface SettingsAccessProps {
  daemon: DaemonInfo | null; // null → daemon has not answered
  presence: PresenceSeat[] | null; // null → presence feed unavailable, [] → nobody posted
  mode: AccessMode;
  onLogin: (token: string) => Promise<void> | void;
  cfAccess?: boolean;
}

/* ───────────────────────── helpers ───────────────────────── */

const MODE_LABEL: Record<AccessMode, string> = {
  open: "open",
  token: "token",
  cloudflare: "cloudflare",
};

const MODE_HINT: Record<AccessMode, string> = {
  open: "no credential required — anyone who can reach this daemon can read it",
  token: "a human access token is required",
  cloudflare: "gated by Cloudflare Access",
};

const STALE_MS = 5 * 60 * 1000;
const MAX_TOKEN_LEN = 4096;

/** Returns an error message if `raw` looks like a key list / CSV rather than one token. */
function listError(raw: string): string | null {
  const t = raw.trim();
  if (/[\r\n]/.test(t)) return "that looks like a key list — paste a single token";
  if (/[,;\t]/.test(t)) return "that looks like CSV — paste a single token, not a list";
  if (/\s/.test(t)) return "tokens don't contain spaces — paste a single token";
  return null;
}

function validateToken(raw: string): string | null {
  const t = raw.trim();
  if (!t) return "enter your access token";
  if (t.length > MAX_TOKEN_LEN) return "that's too long to be a single token";
  return listError(t);
}

function toMs(v: string | number | null): number | null {
  if (v == null) return null;
  const ms = typeof v === "number" ? v : Date.parse(v);
  return Number.isFinite(ms) ? ms : null;
}

function fmtAgo(ms: number | null, now: number): string {
  if (ms == null) return "unknown";
  const s = Math.max(0, Math.round((now - ms) / 1000));
  if (s < 10) return "just now";
  if (s < 60) return `${s}s ago`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 48) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

function fmtUptime(sec: number): string {
  const s = Math.max(0, Math.floor(sec));
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  if (d) return `${d}d ${h}h ${m}m`;
  if (h) return `${h}h ${m}m`;
  if (m) return `${m}m`;
  return `${s}s`;
}

function errMessage(e: unknown): string {
  if (e instanceof Error && e.message) return e.message;
  if (typeof e === "string" && e) return e;
  return "token rejected";
}

function cfLoginHref(): string {
  if (typeof window === "undefined") return "/cdn-cgi/access/login";
  return `/cdn-cgi/access/login?redirect_url=${encodeURIComponent(window.location.href)}`;
}

function useNow(intervalMs: number): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const id = window.setInterval(() => setNow(Date.now()), intervalMs);
    return () => window.clearInterval(id);
  }, [intervalMs]);
  return now;
}

/* ───────────────────────── brand mark ───────────────────────── */

function BrandMark({ size = 40 }: { size?: number }) {
  const seats = useMemo(
    () =>
      Array.from({ length: 8 }, (_, i) => {
        const a = (i / 8) * Math.PI * 2 - Math.PI / 2;
        return { x: 16 + Math.cos(a) * 11, y: 16 + Math.sin(a) * 11 };
      }),
    []
  );
  return (
    <svg className="k-mark" width={size} height={size} viewBox="0 0 32 32" aria-hidden="true">
      <circle cx="16" cy="16" r="11" className="k-mark__ring" />
      {seats.map((p, i) => (
        <circle key={i} cx={p.x} cy={p.y} r="1.9" className="k-mark__seat" />
      ))}
      <circle cx="16" cy="16" r="3.4" className="k-mark__core" />
    </svg>
  );
}

/* ───────────────────────── token form (shared) ───────────────────────── */

interface TokenFormProps {
  onLogin: SettingsAccessProps["onLogin"];
  cfAccess?: boolean;
  instance?: string | null;
  autoFocus?: boolean;
  compact?: boolean;
}

function TokenForm({ onLogin, cfAccess, instance, autoFocus, compact }: TokenFormProps) {
  const inputId = useId();
  const errId = useId();
  const [token, setToken] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const mounted = useRef(true);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const onPaste = (e: ClipboardEvent<HTMLInputElement>) => {
    const text = e.clipboardData.getData("text");
    const err = listError(text);
    if (err) {
      e.preventDefault();
      setError(err);
    } else {
      setError(null);
    }
  };

  const onSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (loading) return;
    const err = validateToken(token);
    if (err) {
      setError(err);
      inputRef.current?.focus();
      return;
    }
    setError(null);
    setLoading(true);
    try {
      await onLogin(token.trim());
    } catch (x) {
      if (mounted.current) {
        setError(errMessage(x));
        inputRef.current?.select();
      }
    } finally {
      if (mounted.current) setLoading(false);
    }
  };

  return (
    <form
      className={`k-token${compact ? " k-token--compact" : ""}`}
      method="post"
      action="#"
      onSubmit={onSubmit}
      noValidate
    >
      {/* Username anchor so password managers key the saved credential to this brain. */}
      <input
        className="k-visually-hidden"
        type="text"
        name="username"
        autoComplete="username"
        value={instance || "kurultai"}
        readOnly
        tabIndex={-1}
        aria-hidden="true"
      />

      <label className="k-token__label" htmlFor={inputId}>
        access token
      </label>
      <input
        ref={inputRef}
        id={inputId}
        className="k-token__input"
        type="password"
        name="password"
        autoComplete="current-password"
        spellCheck={false}
        autoCapitalize="off"
        autoCorrect="off"
        autoFocus={autoFocus}
        value={token}
        disabled={loading}
        aria-invalid={error ? true : undefined}
        aria-describedby={error ? errId : undefined}
        onPaste={onPaste}
        onChange={(e) => {
          setToken(e.target.value);
          if (error) setError(null);
        }}
      />
      <p id={errId} className="k-token__error" role="alert" aria-live="assertive">
        {error ?? ""}
      </p>

      <button className="k-btn k-btn--primary" type="submit" disabled={loading} aria-busy={loading}>
        {loading ? (
          <>
            <span className="k-spin" aria-hidden="true" />
            <span>checking…</span>
          </>
        ) : (
          "continue"
        )}
      </button>

      {cfAccess && (
        <a className="k-token__cf" href={cfLoginHref()}>
          or sign in with Cloudflare →
        </a>
      )}
    </form>
  );
}

/* ───────────────────────── HumanAccess gate ───────────────────────── */

export function HumanAccess({ daemon, onLogin, cfAccess }: SettingsAccessProps) {
  return (
    <main className="k-gate">
      <section className="k-gate__card k-glass" aria-labelledby="k-gate-title">
        <header className="k-gate__head">
          <BrandMark size={44} />
          <h1 id="k-gate-title" className="k-wordmark">
            KURULTAI
          </h1>
          <p className="k-gate__lede">this brain is private — enter your key</p>
          {daemon?.instance && <p className="k-gate__instance">{daemon.instance}</p>}
        </header>
        <TokenForm onLogin={onLogin} cfAccess={cfAccess} instance={daemon?.instance} autoFocus />
      </section>
    </main>
  );
}

/* ───────────────────────── Settings panel ───────────────────────── */

function Row({ k, children }: { k: string; children: React.ReactNode }) {
  return (
    <div className="k-row">
      <dt className="k-row__k">{k}</dt>
      <dd className="k-row__v">{children}</dd>
    </div>
  );
}

export function SettingsPanel({ daemon, presence, mode, onLogin, cfAccess }: SettingsAccessProps) {
  const now = useNow(30_000);
  const [accessOpen, setAccessOpen] = useState(false);
  const accessRegion = useId();

  const seats = useMemo(() => {
    if (!presence) return null;
    return presence
      .map((s) => ({ ...s, ms: toMs(s.last_seen) }))
      .sort((a, b) => (b.ms ?? -Infinity) - (a.ms ?? -Infinity));
  }, [presence]);

  const status: "online" | "offline" | "unknown" =
    daemon == null ? "unknown" : daemon.online ? "online" : "offline";

  return (
    <section className="k-settings k-glass" aria-label="settings">
      {/* Daemon */}
      <div className="k-sect">
        <h2 className="k-sect__title">daemon</h2>
        <dl className="k-rows">
          <Row k="status">
            <span className={`k-status k-status--${status}`}>
              <span className="k-dot" aria-hidden="true" />
              {status === "unknown" ? "no response" : status}
            </span>
          </Row>
          <Row k="instance">
            {daemon?.instance ? daemon.instance : <span className="k-muted">—</span>}
          </Row>
          <Row k="version">
            {daemon?.version ? daemon.version : <span className="k-muted">—</span>}
          </Row>
          {daemon?.uptimeSec != null && Number.isFinite(daemon.uptimeSec) && (
            <Row k="uptime">{fmtUptime(daemon.uptimeSec)}</Row>
          )}
        </dl>
      </div>

      {/* Access */}
      <div className="k-sect">
        <h2 className="k-sect__title">access</h2>
        <dl className="k-rows">
          <Row k="mode">
            <span className={`k-pill k-pill--${mode}`}>{MODE_LABEL[mode]}</span>
          </Row>
        </dl>
        <p className={`k-hint${mode === "open" ? " k-hint--warn" : ""}`}>{MODE_HINT[mode]}</p>
        <button
          type="button"
          className="k-btn k-btn--ghost"
          aria-expanded={accessOpen}
          aria-controls={accessRegion}
          onClick={() => setAccessOpen((v) => !v)}
        >
          {accessOpen ? "close" : mode === "token" ? "enter / replace token" : "token settings"}
        </button>
        <div id={accessRegion} hidden={!accessOpen} className="k-sect__drawer">
          {accessOpen && (
            <TokenForm
              onLogin={onLogin}
              cfAccess={cfAccess}
              instance={daemon?.instance}
              autoFocus
              compact
            />
          )}
        </div>
      </div>

      {/* Agent seats */}
      <div className="k-sect">
        <h2 className="k-sect__title">
          agent seats
          {seats && seats.length > 0 && <span className="k-count">{seats.length}</span>}
        </h2>
        {seats == null ? (
          <p className="k-empty">presence unavailable</p>
        ) : seats.length === 0 ? (
          <p className="k-empty">no seats have posted presence</p>
        ) : (
          <ul className="k-seats">
            {seats.map((s) => {
              const stale = s.ms == null || now - s.ms > STALE_MS;
              return (
                <li
                  key={`${s.codename}@${s.instance_id}`}
                  className={`k-seat${stale ? " k-seat--stale" : ""}`}
                >
                  <span className="k-dot" aria-hidden="true" />
                  <span className="k-seat__id">
                    <span className="k-seat__code">{s.codename}</span>
                    <span className="k-seat__at">@{s.instance_id}</span>
                  </span>
                  <time
                    className="k-seat__seen"
                    dateTime={s.ms != null ? new Date(s.ms).toISOString() : undefined}
                    title={s.ms != null ? new Date(s.ms).toLocaleString() : "no timestamp"}
                  >
                    {fmtAgo(s.ms, now)}
                  </time>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </section>
  );
}

export default SettingsPanel;
