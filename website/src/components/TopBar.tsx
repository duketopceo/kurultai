import { useEffect, useState } from 'react';

interface Props {
  daemonOk: boolean;
  daemonVersion: string;
}

function initialTheme(): string {
  const saved = localStorage.getItem('kurultai-theme');
  if (saved === 'light' || saved === 'dark') return saved;
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
}

export function TopBar({ daemonOk, daemonVersion }: Props) {
  const [theme, setTheme] = useState(initialTheme);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem('kurultai-theme', theme);
  }, [theme]);

  const toggleTheme = () => setTheme((t) => (t === 'dark' ? 'light' : 'dark'));

  return (
    <header className="topbar">
      <a className="brand" href="/ui/" aria-label="Kurultai home">
        <span className="brand-mark" aria-hidden="true">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="3" />
            <circle cx="19" cy="5" r="2" /><circle cx="5" cy="19" r="2" />
            <circle cx="19" cy="19" r="2" /><circle cx="5" cy="5" r="2" />
            <line x1="12" y1="12" x2="19" y2="5" />
            <line x1="12" y1="12" x2="5" y2="19" />
            <line x1="12" y1="12" x2="19" y2="19" />
            <line x1="12" y1="12" x2="5" y2="5" />
          </svg>
        </span>
        <span>KURULTAI</span>
        <small>LOCAL BRAIN</small>
      </a>
      <nav className="topbar-nav" aria-label="Site navigation">
        <a href="/ui/index.html">Home</a>
        <a href="/ui/" className="active">Brain Explorer</a>
      </nav>
      <div className="topbar-status" aria-live="polite">
        <span className="status-dot" style={{ background: daemonOk ? 'var(--electric-dim)' : 'var(--danger)' }} />
        <span id="daemon-status">{daemonOk ? `online${daemonVersion ? ' · v' + daemonVersion : ''}` : 'connecting'}</span>
      </div>
      <button
        id="theme-toggle"
        className="icon-button"
        type="button"
        aria-label={`Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`}
        aria-pressed={theme === 'light'}
        onClick={toggleTheme}
      >
        <span aria-hidden="true">◐</span>
        <span className="button-copy">Theme</span>
      </button>
    </header>
  );
}
