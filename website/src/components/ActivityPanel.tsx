import { useEffect, useState, useRef } from 'react';
import { fetchActivity } from '../api';
import type { ActivityItem } from '../types';

interface Props {
  live: boolean;
  onLiveToggle: (live: boolean) => void;
}

export function ActivityPanel({ live, onLiveToggle }: Props) {
  const [items, setItems] = useState<ActivityItem[]>([]);
  const abortRef = useRef<AbortController | null>(null);

  useEffect(() => {
    if (!live) return;
    let alive = true;
    const poll = async () => {
      while (alive) {
        try {
          abortRef.current?.abort();
          const ac = new AbortController();
          abortRef.current = ac;
          const data = await fetchActivity(ac.signal);
          if (alive) setItems(data.slice(0, 30));
        } catch { /* ignore */ }
        await new Promise((r) => setTimeout(r, 4000));
      }
    };
    poll();
    return () => { alive = false; abortRef.current?.abort(); };
  }, [live]);

  return (
    <aside className="panel chrome-panel stream-panel" aria-labelledby="stream-heading">
      <div className="panel-heading">
        <p className="eyebrow">Pulse</p>
        <h2 id="stream-heading">Memory stream</h2>
        <button
          id="stream-toggle"
          className="quiet-button"
          type="button"
          aria-pressed={live}
          onClick={() => onLiveToggle(!live)}
        >
          {live ? 'live' : 'paused'}
        </button>
      </div>
      <div id="activity-stream" className="activity-stream" aria-live="polite">
        {items.length === 0 ? (
          <p className="empty-state">Listening for local activity…</p>
        ) : (
          items.map((item) => (
            <div key={item.id} className="activity-item">
              <span className="activity-event">{item.event}</span>
              {item.title && <span className="activity-title">{item.title}</span>}
              {item.source && <span className="activity-source">{item.source}</span>}
            </div>
          ))
        )}
      </div>
    </aside>
  );
}
