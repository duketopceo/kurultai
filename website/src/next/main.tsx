import { StrictMode, useCallback, useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import './bridge.css';
import { NextApp } from './App';
import { HumanAccess } from './gen/settings-access/settings-access';
import {
  AUTH_CHANGED_EVENT,
  AuthModeContext,
  TOKEN_INVALID_EVENT,
  probeAuthMode,
  saveToken,
  type AuthMode,
} from '../auth';

type GateState =
  | { phase: 'booting' }
  | { phase: 'gate'; mode: AuthMode; error: string | null }
  | { phase: 'ready'; mode: AuthMode };

function Root() {
  const [gate, setGate] = useState<GateState>({ phase: 'booting' });

  const refresh = useCallback(async (error: string | null = null) => {
    try {
      const { mode, okWithStoredToken } = await probeAuthMode();
      if (mode === 'locked' && !okWithStoredToken) {
        setGate({ phase: 'gate', mode, error });
      } else {
        setGate({ phase: 'ready', mode });
      }
    } catch {
      setGate({ phase: 'ready', mode: 'unknown' });
    }
  }, []);

  useEffect(() => {
    void refresh();
    const onInvalid = () => void refresh('token rejected — sign in again');
    const onChanged = () => void refresh();
    window.addEventListener(TOKEN_INVALID_EVENT, onInvalid);
    window.addEventListener(AUTH_CHANGED_EVENT, onChanged);
    return () => {
      window.removeEventListener(TOKEN_INVALID_EVENT, onInvalid);
      window.removeEventListener(AUTH_CHANGED_EVENT, onChanged);
    };
  }, [refresh]);

  const login = useCallback(async (token: string) => {
    saveToken(token);
    await refresh();
  }, [refresh]);

  if (gate.phase === 'booting') {
    return <div style={{ padding: 32, fontFamily: 'var(--k-font-mono)', fontSize: 12, color: 'var(--text-3)' }}>probing daemon…</div>;
  }
  if (gate.phase === 'gate') {
    return (
      <HumanAccess
        daemon={{ online: false }}
        mode="token"
        presence={null}
        onLogin={login}
      />
    );
  }
  return (
    <AuthModeContext.Provider value={gate.mode}>
      <NextApp />
    </AuthModeContext.Provider>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Root />
  </StrictMode>,
);
