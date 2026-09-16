import { useCallback, useEffect, useMemo, useState } from 'react';
import { useKanban } from '@mshafiqyajid/react-kanban';
import type { KanbanCard, KanbanColumn } from '@mshafiqyajid/react-kanban';
import * as api from '../../api';
import { laneBody, messagesToColumns, withLane, type LaneMessage } from './kanbanMapping';

type CardData = { message: LaneMessage };

export type HeyKanbanProps = {
  threadId: string;
  messages: api.HeyMessage[];
  /** Called after a successful mutation so the parent can refetch. */
  onChanged(): void;
};

function toColumns(messages: api.HeyMessage[]): KanbanColumn<CardData>[] {
  return messagesToColumns(messages as LaneMessage[]).map((col) => ({
    id: col.id,
    title: col.title,
    cards: col.messages.map(
      (m): KanbanCard<CardData> => ({
        id: m.id,
        content: laneBody(m.content) || '(empty)',
        label: m.agent_codename || undefined,
        data: { message: m },
      }),
    ),
  }));
}

/** Kanban view over a Hey thread — lanes come from `[lane]` tokens in
 *  message content; moving a card PATCHes the message. (#331) */
export function HeyKanban({ threadId, messages, onChanged }: HeyKanbanProps) {
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [editing, setEditing] = useState<string | null>(null);
  const [editDraft, setEditDraft] = useState('');
  const [pending, setPending] = useState(false);

  const columns = useMemo(() => toColumns(messages), [messages]);

  const persistMove = useCallback(
    async (card: KanbanCard<CardData>, toColumnId: string) => {
      const msg = card.data?.message;
      if (!msg) return;
      setPending(true);
      try {
        await api.updateHeyMessage(msg.id, withLane(msg.content, toColumnId));
        onChanged();
      } catch {
        onChanged(); // revert to server state
      } finally {
        setPending(false);
      }
    },
    [onChanged],
  );

  const kanban = useKanban<CardData>({
    columns,
    onChange: () => {},
    onCardMove: (card, _from, to) => void persistMove(card, to),
  });

  useEffect(() => {
    kanban.setColumns(columns);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [columns]);

  const addCard = useCallback(
    async (lane: string) => {
      const body = (drafts[lane] ?? '').trim();
      if (!body || pending) return;
      setPending(true);
      try {
        await api.postHeyMessage(threadId, withLane(body, lane));
        setDrafts((d) => ({ ...d, [lane]: '' }));
        onChanged();
      } catch {
        // leave draft in place
      } finally {
        setPending(false);
      }
    },
    [drafts, pending, threadId, onChanged],
  );

  const saveEdit = useCallback(
    async (card: KanbanCard<CardData>, columnId: string) => {
      const body = editDraft.trim();
      setEditing(null);
      if (!body) return;
      setPending(true);
      try {
        await api.updateHeyMessage(card.id, withLane(body, columnId));
        onChanged();
      } finally {
        setPending(false);
      }
    },
    [editDraft, onChanged],
  );

  return (
    <div
      className="hey-kanban"
      {...kanban.getBoardProps()}
      aria-label="Kanban view of thread"
      aria-busy={pending}
    >
      {kanban.columns.map((col) => (
        <section key={col.id} className="hey-kanban-col" {...kanban.getColumnDropProps(col.id)}>
          <header className="hey-kanban-col-head">
            <h3 className="hey-kanban-col-title">{col.title}</h3>
            <span className="hey-kanban-count" aria-label={`${col.cards.length} cards`}>
              {col.cards.length}
            </span>
          </header>
          <ul className="hey-kanban-cards">
            {col.cards.map((card) => (
              <li key={card.id}>
                {editing === card.id ? (
                  <form
                    className="hey-kanban-edit"
                    onSubmit={(e) => {
                      e.preventDefault();
                      void saveEdit(card, col.id);
                    }}
                  >
                    <textarea
                      className="kb-focus-ring"
                      rows={3}
                      value={editDraft}
                      autoFocus
                      onChange={(e) => setEditDraft(e.currentTarget.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Escape') setEditing(null);
                        if (e.key === 'Enter' && !e.shiftKey) {
                          e.preventDefault();
                          void saveEdit(card, col.id);
                        }
                      }}
                    />
                    <div className="hey-kanban-edit-actions">
                      <button type="submit" className="kb-send kb-focus-ring">Save</button>
                      <button type="button" className="ghost" onClick={() => setEditing(null)}>
                        Cancel
                      </button>
                    </div>
                  </form>
                ) : (
                  <article
                    className="hey-kanban-card kb-focus-ring"
                    {...kanban.getCardProps(card.id, col.id)}
                  >
                    <p className="hey-kanban-card-body">{card.content}</p>
                    <footer className="hey-kanban-card-meta">
                      <span className="hey-kanban-author">
                        {card.data?.message.agent_codename || 'agent'}
                      </span>
                      <button
                        type="button"
                        className="ghost hey-kanban-edit-btn"
                        aria-label="Edit card"
                        onClick={(e) => {
                          e.stopPropagation();
                          setEditing(card.id);
                          setEditDraft(card.content);
                        }}
                      >
                        Edit
                      </button>
                    </footer>
                  </article>
                )}
              </li>
            ))}
          </ul>
          <form
            className="hey-kanban-add"
            onSubmit={(e) => {
              e.preventDefault();
              void addCard(col.id);
            }}
          >
            <input
              className="kb-focus-ring"
              type="text"
              placeholder="+ add card"
              aria-label={`Add card to ${col.title}`}
              value={drafts[col.id] ?? ''}
              onChange={(e) =>
                setDrafts((d) => ({ ...d, [col.id]: e.currentTarget.value }))
              }
            />
          </form>
        </section>
      ))}
    </div>
  );
}
