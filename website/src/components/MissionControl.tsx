import { type ReactNode } from 'react';

export type MissionTab = 'pulse' | 'focus' | 'synthesize' | 'ask';

const TABS: { id: MissionTab; label: string }[] = [
  { id: 'pulse', label: 'Pulse' },
  { id: 'focus', label: 'Focus' },
  { id: 'synthesize', label: 'Synthesize' },
  { id: 'ask', label: 'Ask' },
];

interface Props {
  active: MissionTab;
  onChange: (tab: MissionTab) => void;
  children: Record<MissionTab, ReactNode>;
}

export function MissionControl({ active, onChange, children }: Props) {
  return (
    <nav className="mission-control" aria-label="Mission control">
      {TABS.map(({ id, label }) => (
        <button
          key={id}
          role="tab"
          aria-selected={active === id}
          className={`mission-tab${active === id ? ' is-active' : ''}`}
          type="button"
          onClick={() => onChange(id)}
        >
          {label}
        </button>
      ))}
      <div className="mission-content" role="tabpanel" aria-label={`${active} panel`}>
        {children[active]}
      </div>
    </nav>
  );
}
