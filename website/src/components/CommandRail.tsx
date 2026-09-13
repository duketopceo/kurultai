import { type ReactNode } from 'react';
import { useAuthMode } from '../auth';
import { AccessSettingsButton } from './HumanAccess';

export type RailTab = 'hey' | 'repos' | 'logs' | 'settings';

const TABS: { id: RailTab; label: string; key: string }[] = [
  { id: 'hey', label: 'Hey', key: 'H' },
  { id: 'repos', label: 'Repos', key: 'R' },
  { id: 'logs', label: 'Logs', key: 'L' },
  { id: 'settings', label: 'Settings', key: 'S' },
];

interface Props {
  active: RailTab;
  onChange: (tab: RailTab) => void;
  children: Record<RailTab, ReactNode>;
  daemonOk: boolean;
  daemonVersion: string;
}

export function CommandRail({ active, onChange, children, daemonOk, daemonVersion }: Props) {
  const authMode = useAuthMode();
  return (
    <aside className="command-rail" aria-label="Command rail">
      <div className="rail-tabs" role="tablist" aria-label="Command tabs">
        {TABS.map(({ id, label, key }) => (
          <button
            key={id}
            role="tab"
            aria-selected={active === id}
            className={`rail-tab${active === id ? ' is-active' : ''}`}
            type="button"
            onClick={() => onChange(id)}
            title={`${label} (${key})`}
          >
            <span>{label}</span>
            <kbd>{key}</kbd>
          </button>
        ))}
      </div>
      <div className="rail-status" aria-live="polite">
        <span className="status-dot" style={{ background: daemonOk ? 'var(--chrome-ok)' : 'var(--chrome-danger)' }} />
        <span className="status-text">
          {daemonOk ? `online · ${daemonVersion || '—'}` : 'connecting'}
        </span>
      </div>
      <div className="rail-content" role="tabpanel" aria-label={`${active} panel`}>
        {children[active]}
      </div>
      <div className="rail-footer">
        <AccessSettingsButton mode={authMode} onChanged={() => { /* root listens for auth-changed */ }} />
      </div>
    </aside>
  );
}
