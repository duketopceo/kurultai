import { useEffect, useState } from 'react';
import type { LoadTier, LayoutMode } from '../types';

interface Props {
  theme: string;
  onTheme: (theme: string) => void;
  loadTier: LoadTier;
  onLoadTier: (tier: LoadTier) => void;
  layout: LayoutMode;
  onLayout: (mode: LayoutMode) => void;
}

const TIERS: LoadTier[] = ['low', 'mid', 'high', 'max'];

export function SettingsPanel({ theme, onTheme, loadTier, onLoadTier, layout, onLayout }: Props) {
  const [logsOpen, setLogsOpen] = useState(true);
  return (
    <section className="settings-panel" aria-label="Settings">
      <h3 className="panel-heading">Settings</h3>
      <div className="settings-group">
        <h4>Appearance</h4>
        <button
          type="button"
          className="settings-toggle"
          onClick={() => onTheme(theme === 'dark' ? 'light' : 'dark')}
        >
          Theme: <strong>{theme}</strong>
        </button>
      </div>
      <div className="settings-group">
        <h4>Brain</h4>
        <div className="settings-row">
          <span>Layout</span>
          <div className="settings-segment" role="group" aria-label="Layout mode">
            {(['brain', 'ontology'] as LayoutMode[]).map((mode) => (
              <button
                key={mode}
                type="button"
                className={layout === mode ? 'is-active' : ''}
                onClick={() => onLayout(mode)}
              >
                {mode}
              </button>
            ))}
          </div>
        </div>
        <div className="settings-row">
          <span>Load tier</span>
          <div className="settings-segment" role="group" aria-label="Memory tier">
            {TIERS.map((tier) => (
              <button
                key={tier}
                type="button"
                className={loadTier === tier ? 'is-active' : ''}
                onClick={() => onLoadTier(tier)}
              >
                {tier}
              </button>
            ))}
          </div>
        </div>
      </div>
      <div className="settings-group">
        <h4>Agent sources</h4>
        <ul className="settings-list">
          <li>cursor <span className="status-ok">connected</span></li>
          <li>claude <span className="status-ok">connected</span></li>
          <li>codex <span className="status-ok">connected</span></li>
          <li>antigravity <span className="status-ok">connected</span></li>
        </ul>
      </div>
      <div className="settings-group">
        <h4>Logs viewer</h4>
        <label className="settings-check">
          <input type="checkbox" checked={logsOpen} onChange={(e) => setLogsOpen(e.target.checked)} />
          Show structured logs
        </label>
      </div>
    </section>
  );
}
