import { FormEvent, useEffect, useId, useState } from 'react';
import { clearToken, emitAuthChanged, readToken, saveToken, type AuthMode } from '../auth';

type Props = {
  mode: AuthMode;
  error?: string | null;
  onSaved: (token: string) => void;
  onContinueOpen?: () => void;
};

const LOGIN_USERNAME = 'knowledge.shippedit.dev';

/** Full-screen human access gate — only when the instance is locked. */
export function HumanLoginGate({ mode, error, onSaved, onContinueOpen }: Props) {
  const [value, setValue] = useState('');
  const [localError, setLocalError] = useState<string | null>(error ?? null);
  const userId = useId();
  const passId = useId();

  useEffect(() => {
    setLocalError(error ?? null);
  }, [error]);

  const submit = (e?: FormEvent) => {
    e?.preventDefault();
    const trimmed = value.trim();
    if (!trimmed) {
      setLocalError('Paste your hub API key (password field) to continue.');
      return;
    }
    // Reject accidental paste of the whole CSV list.
    if (trimmed.includes(',')) {
      setLocalError('Paste one key only — not the comma-separated PERSONAL_API_KEYS list.');
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
          Use the <strong>hub API key</strong> for this instance (1Password login:{' '}
          <code>{LOGIN_USERNAME}</code>). Agent keys from <code>kurultai agent add</code> are
          MCP-only — not for this form.
        </p>
        {mode === 'open' && onContinueOpen ? (
          <p className="hint">
            This instance looks open (no auth).{' '}
            <button type="button" className="linkish" onClick={onContinueOpen}>
              Continue without a key
            </button>
          </p>
        ) : null}
        <form
          className="human-login-form"
          method="post"
          action="#"
          autoComplete="on"
          onSubmit={submit}
        >
          {/* Username anchors 1Password / browser save to this site */}
          <label htmlFor={userId}>Account</label>
          <input
            id={userId}
            name="username"
            type="text"
            autoComplete="username"
            value={LOGIN_USERNAME}
            readOnly
          />
          <label htmlFor={passId}>API key</label>
          <input
            id={passId}
            name="password"
            type="password"
            autoComplete="current-password"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="64-char hub key"
            aria-invalid={localError ? true : undefined}
            autoFocus
          />
          {localError ? <p className="form-error" role="alert">{localError}</p> : null}
          <button type="submit" className="btn-primary" disabled={value.trim().length === 0}>
            Sign in
          </button>
        </form>
        <div className="help-block">
          <p>
            <strong>Where the key lives:</strong> server-001{' '}
            <code>/home/khan/kurultai/.env</code> → <code>PERSONAL_API_KEYS</code> (first key
            before the comma). Also in 1Password as “Kurultai — knowledge.shippedit.dev”.
          </p>
          <p>
            Or mint a named key:{' '}
            <code>docker exec -u kurultai kurultai-personal kurultai admin key issue --name human --max-tier private</code>
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

  const save = (e?: FormEvent) => {
    e?.preventDefault();
    const trimmed = draft.trim();
    if (!trimmed || trimmed.includes(',')) return;
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
          <form method="post" action="#" autoComplete="on" onSubmit={save}>
            <label htmlFor={`${panelId}-user`}>Account</label>
            <input
              id={`${panelId}-user`}
              name="username"
              type="text"
              autoComplete="username"
              defaultValue={LOGIN_USERNAME}
              readOnly
            />
            <label htmlFor={`${panelId}-token`}>API key</label>
            <input
              id={`${panelId}-token`}
              name="password"
              type="password"
              autoComplete="current-password"
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              placeholder="Paste hub key"
            />
            <div className="access-actions">
              <button type="submit" className="btn-primary" disabled={!draft.trim() || draft.includes(',')}>
                Save token
              </button>
              <button type="button" className="ghost" disabled={!hasToken} onClick={signOut}>
                Sign out
              </button>
            </div>
          </form>
          <p className="muted small">
            1Password: login “Kurultai — knowledge.shippedit.dev”. Agents keep MCP keys separate.
          </p>
        </div>
      ) : null}
    </div>
  );
}
