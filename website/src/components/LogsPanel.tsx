import { useMemo, useState } from 'react';

interface LogEntry {
  id: string;
  time: string;
  source: string;
  agent: string;
  message: string;
  level: 'info' | 'warn' | 'error';
}

const SEED: LogEntry[] = [
  { id: '1', time: '12:04:02', source: 'mcp', agent: 'cursor', message: 'search: "ui-uc-max" — 0 hits', level: 'info' },
  { id: '2', time: '12:03:55', source: 'pipeline', agent: 'daemon', message: 'index complete: notes · 0 atoms', level: 'info' },
  { id: '3', time: '12:03:41', source: 'brain', agent: 'devin', message: 'FDG layout converged · 500 neurons', level: 'info' },
  { id: '4', time: '12:02:18', source: 'hey', agent: 'antigravity', message: 'online via kurultai-personal bridge', level: 'info' },
  { id: '5', time: '12:01:07', source: 'ingest', agent: 'daemon', message: 'merged 1,088 candidates', level: 'warn' },
];

export function LogsPanel() {
  const [filter, setFilter] = useState('');
  const [source, setSource] = useState<string>('all');

  const sources = useMemo(() => Array.from(new Set(SEED.map((l) => l.source))), []);
  const filtered = useMemo(() => {
    return SEED.filter((l) => {
      const matchesSource = source === 'all' || l.source === source;
      const matchesText = !filter || `${l.agent} ${l.message}`.toLowerCase().includes(filter.toLowerCase());
      return matchesSource && matchesText;
    });
  }, [filter, source]);

  return (
    <section className="logs-panel" aria-label="Structured logs">
      <div className="logs-toolbar">
        <input
          type="search"
          className="logs-search"
          placeholder="Filter logs…"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        />
        <select className="logs-filter" value={source} onChange={(e) => setSource(e.target.value)}>
          <option value="all">all sources</option>
          {sources.map((s) => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
      </div>
      <ul className="logs-list">
        {filtered.map((entry) => (
          <li key={entry.id} className={`log-line log-${entry.level}`}>
            <span className="log-time">{entry.time}</span>
            <span className="log-agent">{entry.agent}</span>
            <span className="log-source">{entry.source}</span>
            <span className="log-message">{entry.message}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}
