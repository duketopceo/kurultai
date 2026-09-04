import { StrictMode, useState, useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import './styles.css';
import { App } from './App';
import { RepoBrainPage } from './components/RepoBrain';

const TOKEN_KEY = 'kurultai:token';

function readToken(): string {
  try {
    return localStorage.getItem(TOKEN_KEY) || '';
  } catch {
    return '';
  }
}

function saveToken(value: string) {
  try {
    if (value.length > 0) {
      localStorage.setItem(TOKEN_KEY, value);
    } else {
      localStorage.removeItem(TOKEN_KEY);
    }
  } catch { /* best-effort */ }
}

function TokenPrompt({ onSave }: { onSave: (token: string) => void }) {
  const [value, setValue] = useState('');
  return (
    <div className="token-gate" role="dialog" aria-label="API token required">
      <div className="token-card">
        <h1>Kurultai</h1>
        <p>This instance requires an API token.</p>
        <input
          type="password"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder="Paste your API key"
          aria-label="API token"
          autoFocus
          onKeyDown={(e) => { if (e.key === 'Enter' && value.length > 0) onSave(value); }}
        />
        <button onClick={() => onSave(value)} disabled={value.length === 0}>
          Continue
        </button>
      </div>
    </div>
  );
}

function Router() {
  const [hash, setHash] = useState(window.location.hash);
  useEffect(() => {
    const handler = () => setHash(window.location.hash);
    window.addEventListener('hashchange', handler);
    return () => window.removeEventListener('hashchange', handler);
  }, []);

  if (hash.startsWith('#/repo')) return <RepoBrainPage />;
  return <App />;
}

function Root() {
  const [token, setToken] = useState<string | null>(null);

  useEffect(() => {
    setToken(readToken());
  }, []);

  if (token === null) return null;
  if (token.length === 0) {
    return (
      <StrictMode>
        <TokenPrompt onSave={(value) => { saveToken(value); setToken(value); }} />
      </StrictMode>
    );
  }

  return (
    <StrictMode>
      <Router />
    </StrictMode>
  );
}

const root = document.getElementById('root');
if (!root) throw new Error('No #root element found');
createRoot(root).render(<Root />);
