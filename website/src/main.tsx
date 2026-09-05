import { StrictMode, useCallback, useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import './styles.css';
import { App } from './App';
import { RepoBrainPage } from './components/RepoBrain';
import { HumanLoginGate } from './components/HumanAccess';
import {
  AUTH_CHANGED_EVENT,
  AuthModeContext,
  TOKEN_INVALID_EVENT,
  probeAuthMode,
  type AuthMode,
} from './auth';

type GateState =
  | { phase: 'booting' }
  | { phase: 'gate'; mode: AuthMode; error: string | null }
  | { phase: 'ready'; mode: AuthMode };

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
  const [gate, setGate] = useState<GateState>({ phase: 'booting' });

  const refresh = useCallback(async (error: string | null = null) => {
    try {
      const { mode, okWithStoredToken } = await probeAuthMode();
      if (mode === 'locked' && !okWithStoredToken) {
        setGate({ phase: 'gate', mode, error });
        return;
      }
      setGate({ phase: 'ready', mode });
    } catch {
      setGate({ phase: 'ready', mode: 'open' });
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    const onInvalid = (e: Event) => {
      const detail = (e as CustomEvent<{ reason?: string }>).detail;
      void refresh(
        detail?.reason === 'unauthorized'
          ? 'That token was rejected. Paste a valid human key.'
          : 'Session expired. Sign in again.',
      );
    };
    const onChanged = () => {
      void refresh();
    };
    window.addEventListener(TOKEN_INVALID_EVENT, onInvalid);
    window.addEventListener(AUTH_CHANGED_EVENT, onChanged);
    return () => {
      window.removeEventListener(TOKEN_INVALID_EVENT, onInvalid);
      window.removeEventListener(AUTH_CHANGED_EVENT, onChanged);
    };
  }, [refresh]);

  if (gate.phase === 'booting') {
    return (
      <div className="token-gate" aria-busy="true">
        <p className="muted">Checking access…</p>
      </div>
    );
  }

  if (gate.phase === 'gate') {
    return (
      <StrictMode>
        <HumanLoginGate
          mode={gate.mode}
          error={gate.error}
          onSaved={() => {
            void refresh();
          }}
          onContinueOpen={() => setGate({ phase: 'ready', mode: 'open' })}
        />
      </StrictMode>
    );
  }

  return (
    <StrictMode>
      <AuthModeContext.Provider value={gate.mode}>
        <Router />
      </AuthModeContext.Provider>
    </StrictMode>
  );
}

const root = document.getElementById('root');
if (!root) throw new Error('No #root element found');
createRoot(root).render(<Root />);
