import { useCallback, useEffect, useState } from 'react';
import { fetchHeyMessages, fetchHeyThreads, type HeyMessage, type HeyThread } from '../api';

function formatWho(m: HeyMessage): string {
  const name = m.agent_codename || m.agent_id.slice(0, 8);
  const inst = m.instance_id ? `@${m.instance_id}` : '';
  const repo = m.repo ? ` · ${m.repo}` : '';
  return `${name}${inst}${repo}`;
}

/** Thin agent message-board panel — does not touch BrainStage. */
export function HeyPanel() {
  const [threads, setThreads] = useState<HeyThread[]>([]);
  const [messages, setMessages] = useState<HeyMessage[]>([]);
  const [active, setActive] = useState<string>('hey.md');
  const [err, setErr] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setErr(null);
      const list = await fetchHeyThreads(20);
      setThreads(list);
      const key = list.find((t) => t.name === active)?.id || active;
      const msgs = await fetchHeyMessages(key, 40);
      setMessages(msgs);
    } catch (e) {
      setErr((e as Error).message || 'hey board unavailable');
      setThreads([]);
      setMessages([]);
    }
  }, [active]);

  useEffect(() => {
    void refresh();
    const id = window.setInterval(() => void refresh(), 15000);
    return () => window.clearInterval(id);
  }, [refresh]);

  return (
    <section className="panel chrome-panel hey-panel" aria-label="Agent message board">
      <header className="panel-head">
        <h2>Hey board</h2>
        <button type="button" className="ghost" onClick={() => void refresh()}>
          Refresh
        </button>
      </header>
      <p className="muted hey-caption">Active WIP / agent coordination — not long-term memory.</p>
      {err ? <p className="muted">{err}</p> : null}
      <div className="hey-threads" role="list">
        {(threads.length ? threads : [{ id: 'hey.md', name: 'hey.md' } as HeyThread]).map((t) => (
          <button
            key={t.id}
            type="button"
            className={active === t.name || active === t.id ? 'hey-thread active' : 'hey-thread'}
            onClick={() => setActive(t.name || t.id)}
          >
            {t.name}
          </button>
        ))}
      </div>
      <ul className="hey-messages">
        {messages.map((m) => (
          <li key={m.id}>
            <span className="hey-meta">
              {formatWho(m)} · {m.kind} · {m.created_at.slice(0, 19)}
            </span>
            <p>{m.content}</p>
          </li>
        ))}
        {!messages.length && !err ? <li className="muted">No posts yet — agents use hey_post with optional repo + instance_id.</li> : null}
      </ul>
    </section>
  );
}
