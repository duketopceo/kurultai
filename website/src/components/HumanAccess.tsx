import { useEffect, useId, useState } from 'react';
import { clearToken, emitAuthChanged, readToken, saveToken, type AuthMode } from '../auth';

type Props = {
  mode: AuthMode;
  error?: string | null;
  onSaved: (token: string) => void;
  onContinueOpen?: () => void;
};

/** Full-screen human access gate — only when the instance is locked. */
export function HumanLoginGate({ mode, error, onSaved, onContinueOpen }: Props) {
  const [value, setValue] = useState('');
  const [localError, setLocalError] = useState<string | null>(error ?? null);
  const inputId = useId();

  useEffect(() => {
    setLocalError(error ?? null);
  }, [error]);

  const submit = () => {
    const trimmed = value.trim();
    if (!trimmed) {
      setLocalError('Paste a human access token to continue.');
      return;
    }
    saveToken(trimmed);
    onSaved(trimmed);
  };

  return (
    <div className="token-gate" role="dialog" aria-labelledby="human-login-title">
      <div className="token-card human-login-card">
        <p className="eyebrow">Human access</p>
        <h1 id="human-login-title">Sign in to this Brain</h1>
        <p className="lede">
          This public Kurultai instance requires your <strong>owner / hub API key</strong>.
          Agent keys from <code>kurultai agent add</code> are for MCP tools — use those in
          agents, not here.
        </p>
        {mode === 'open' && onContinueOpen ? (
          <p className="hint">
            This instance looks open (no auth).{' '}
            <button type="button" className="linkish" onClick={onContinueOpen}>
              Continue without a key
            </button>
          </p>
        ) : null}
        <label htmlFor={inputId}>Human access token</label>
        <input
          id={inputId}
          type="password"
          autoComplete="current-password"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder="Paste token from kurultai admin key issue"
          aria-invalid={localError ? true : undefined}
          autoFocus
          onKeyDown={(e) => {
            if (e.key === 'Enter') submit();
          }}
        />
        {localError ? <p className="form-error" role="alert">{localError}</p> : null}
        <button type="button" className="btn-primary" onClick={submit} disabled={value.trim().length === 0}>
          Continue
        </button>
        <div className="help-block">
          <p>
            Mint a human key on the host:{' '}
            <code>kurultai admin key issue --name human --max-tier private</code>
          </p>
          <p>
            Or paste <strong>one</strong> hub key from <code>KURULTAI_HUB_API_KEYS</code>{' '}
            / <code>PERSONAL_API_KEYS</code> (a single 64-char key — not the whole
            comma-separated list).
          </p>
          <p className="muted">
            Agents: keep using API-key auth via MCP (<code>kurultai agent add</code>) — that
            path stays separate and is working.
          </p>
        </div>
      </div>
    </div>
  );
}

type SettingsProps = {
  mode: AuthMode;
  onChanged: () => void;
};

/** Compact Access settings control for the top bar. */
export function AccessSettingsButton({ mode, onChanged }: SettingsProps) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState('');
  const hasToken = readToken().length > 0;
  const panelId = useId();

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open]);

  const save = () => {
    const trimmed = draft.trim();
    if (!trimmed) return;
    saveToken(trimmed);
    setDraft('');
    setOpen(false);
    onChanged();
    emitAuthChanged();
  };

  const signOut = () => {
    clearToken();
    setOpen(false);
    onChanged();
    emitAuthChanged();
  };

  return (
    <div className="access-settings">
      <button
        type="button"
        className="icon-button"
        aria-expanded={open}
        aria-controls={panelId}
        onClick={() => setOpen((v) => !v)}
      >
        <span aria-hidden="true">⚿</span>
        <span className="button-copy">Access</span>
      </button>
      {open ? (
        <div id={panelId} className="access-panel" role="dialog" aria-label="Access settings">
          <header>
            <strong>Access settings</strong>
            <button type="button" className="ghost" onClick={() => setOpen(false)} aria-label="Close">
              ×
            </button>
          </header>
          <p className="access-mode">
            Instance:{' '}
            <strong>{mode === 'locked' ? 'locked (token required)' : mode === 'open' ? 'open (local / no gate)' : '…'}</strong>
          </p>
          <p className="access-mode">
            Session: <strong>{hasToken ? 'human token stored' : 'no human token'}</strong>
          </p>
          <label htmlFor={`${panelId}-token`}>Update human token</label>
          <input
            id={`${panelId}-token`}
            type="password"
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Paste new owner / hub key"
            autoComplete="off"
          />
          <div className="access-actions">
            <button type="button" className="btn-primary" disabled={!draft.trim()} onClick={save}>
              Save token
            </button>
            <button type="button" className="ghost" disabled={!hasToken} onClick={signOut}>
              Sign out
            </button>
          </div>
          <p className="muted small">
            Agents keep MCP API keys (<code>kurultai agent add</code>). This panel is for
            human Brain UI access only.
          </p>
        </div>
      ) : null}
    </div>
  );
}
