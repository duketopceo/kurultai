import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import type { MessageVM, ThreadVM } from './chatboard-mapping';
export type ChatboardProps = {
  threads: ThreadVM[];
  messages: MessageVM[];
  activeThreadId: string | null;
  presence: Record<string, string>;
  onOpenThread(id: string): void;
  onSend(body: string, parentId?: string): void;
  onReact(messageId: string, emoji: string): void;
  onEdit?(messageId: string, body: string): void;
  onDelete?(messageId: string): void;
  disabled?: boolean;
};
function field(value: unknown, key: string): unknown {
  return typeof value === 'object' && value !== null
    ? (value as Record<string, unknown>)[key]
    : undefined;
}
function text(value: unknown, ...keys: string[]): string {
  for (const key of keys) {
    const result = field(value, key);
    if (typeof result === 'string' && result.length > 0) {
      return result;
    }
  }
  return '';
}
function presenceStatus(presence: Record<string, string>, key: string): string {
  const status = presence[key];
  return status === 'online' || status === 'away' ? status : 'offline';
}
const reactions = [
  { emoji: '👍', label: 'Thumbs up' },
  { emoji: '❤️', label: 'Heart' },
  { emoji: '👀', label: 'Eyes' },
];
function shortThreadLabel(label: string): string {
  return /^[0-9a-f]{8}-[0-9a-f-]{20,}$/i.test(label) ? label.slice(0, 8) : label;
}
export function Chatboard({
  threads,
  messages,
  activeThreadId,
  presence,
  onOpenThread,
  onSend,
  onReact,
  onEdit,
  onDelete,
  disabled = false,
}: ChatboardProps) {
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [focusedThreadId, setFocusedThreadId] = useState<string | null>(null);
  const [replyTo, setReplyTo] = useState<{ id: string; label: string; body: string } | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editDraft, setEditDraft] = useState('');
  const [expanded, setExpanded] = useState(false);
  const threadButtons = useRef(new Map<string, HTMLButtonElement>());
  const stream = useRef<HTMLDivElement>(null);
  const nearBottom = useRef(true);
  const previousThreadId = useRef<string | null>(null);
  const threadViews = threads.map((thread) => {
    const fullLabel = text(thread, 'peerLabel', 'title', 'name') || text(thread, 'id');
    return {
    id: text(thread, 'id'),
    label: shortThreadLabel(fullLabel),
    fullLabel,
    agentKey: text(thread, 'peerKey', 'agentKey', 'peerAgentKey'),
    preview: text(thread, 'preview', 'lastSnippet', 'lastMessagePreview'),
    unread: typeof field(thread, 'unread') === 'number' ? field(thread, 'unread') as number : 0,
    };
  });
  const messageViews = messages.map((message, index) => {
    const timestamp = text(message, 'timeLabel', 'createdAt', 'timestamp', 'created_at');
    const parsedTime = Date.parse(timestamp);
    const rawReactions = field(message, 'reactions');
    return {
      id: text(message, 'id'),
      label: text(message, 'senderLabel', 'authorLabel', 'peerLabel', 'agentLabel') || 'Agent',
      agentKey: text(message, 'agentKey', 'senderKey', 'authorKey'),
      body: text(message, 'body', 'content'),
      timestamp,
      time: typeof field(message, 'createdAtMs') === 'number'
        ? field(message, 'createdAtMs') as number
        : Number.isFinite(parsedTime) ? parsedTime : null,
      own: field(message, 'isOwn') === true,
      reactions: Array.isArray(rawReactions)
        ? rawReactions
            .map((r) => ({ emoji: text(r, 'emoji'), count: typeof field(r, 'count') === 'number' ? field(r, 'count') as number : 0 }))
            .filter((r) => r.emoji && r.count > 0)
        : [],
      index,
    };
  }).sort((a, b) => (a.time ?? 0) - (b.time ?? 0) || a.index - b.index);
  const activeThread = threadViews.find((thread) => thread.id === activeThreadId);
  const tabThreadId = threadViews.some((thread) => thread.id === focusedThreadId)
    ? focusedThreadId
    : activeThread?.id ?? threadViews[0]?.id;
  const draft = activeThreadId === null ? '' : drafts[activeThreadId] ?? '';
  const composerDisabled = disabled || activeThreadId === null;
  useEffect(() => {
    const element = stream.current;
    const changedThread = previousThreadId.current !== activeThreadId;
    if (element && (changedThread || nearBottom.current)) {
      element.scrollTop = element.scrollHeight;
      nearBottom.current = true;
    }
    previousThreadId.current = activeThreadId;
  }, [activeThreadId, messages]);
  useEffect(() => {
    if (!expanded) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setExpanded(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [expanded]);
  function send() {
    const body = draft.trim();
    if (composerDisabled || activeThreadId === null || !body) {
      return;
    }
    onSend(body, replyTo?.id);
    setReplyTo(null);
    setDrafts((current) => ({ ...current, [activeThreadId]: '' }));
  }
  function saveEdit(messageId: string) {
    const body = editDraft.trim();
    setEditingId(null);
    if (body && onEdit) onEdit(messageId, body);
  }
  const board = (
    <div className={`kb-chatboard${expanded ? ' kb-expanded' : ''}`} aria-label="Agent message board">
      <div className="kb-panes">
        <nav className="kb-thread-pane" aria-label="Chat threads">
          <h3 className="kb-pane-title">Threads</h3>
          <ul className="hey-threads kb-thread-list">
            {threadViews.map((thread, index) => {
              const status = presenceStatus(presence, thread.agentKey);
              return (
                <li key={thread.id} className="kb-thread-item">
                  <button
                    ref={(element) => {
                      if (element) {
                        threadButtons.current.set(thread.id, element);
                      } else {
                        threadButtons.current.delete(thread.id);
                      }
                    }}
                    type="button"
                    className={`hey-thread kb-thread kb-focus-ring${thread.id === activeThreadId ? ' active' : ''}`}
                    aria-current={thread.id === activeThreadId ? 'true' : undefined}
                    tabIndex={thread.id === tabThreadId ? 0 : -1}
                    disabled={disabled}
                    title={thread.fullLabel}
                    onFocus={() => setFocusedThreadId(thread.id)}
                    onClick={() => onOpenThread(thread.id)}
                    onKeyDown={(event) => {
                      let nextIndex: number;
                      switch (event.key) {
                        case 'ArrowDown':
                          nextIndex = (index + 1) % threadViews.length;
                          break;
                        case 'ArrowUp':
                          nextIndex = (index - 1 + threadViews.length) % threadViews.length;
                          break;
                        case 'Home':
                          nextIndex = 0;
                          break;
                        case 'End':
                          nextIndex = threadViews.length - 1;
                          break;
                        default:
                          return;
                      }
                      event.preventDefault();
                      const nextThread = threadViews[nextIndex];
                      if (nextThread) {
                        threadButtons.current.get(nextThread.id)?.focus();
                      }
                    }}
                  >
                    <span className="kb-peer">
                      <span className={`kb-presence kb-presence-${status}`} role="img" aria-label={status} title={status}>●</span>
                      <span className="kb-peer-label">{thread.label}</span>
                    </span>
                    {thread.unread > 0 ? (
                      <span className="kb-unread" aria-label={`${thread.unread} unread`}>{thread.unread}</span>
                    ) : null}
                    {thread.preview ? <span className="muted kb-thread-preview">{thread.preview}</span> : null}
                  </button>
                </li>
              );
            })}
          </ul>
          {!threads.length ? <p className="muted kb-empty">No threads yet.</p> : null}
        </nav>
        <div className="kb-conversation" style={{ display: 'flex', flexDirection: 'column', minWidth: 0 }}>
          <header className="kb-conversation-head">
            <h3 className="kb-pane-title">
              {activeThread ? (
                <span className="kb-peer">
                  <span
                    className={`kb-presence kb-presence-${presenceStatus(presence, activeThread.agentKey)}`}
                    role="img"
                    aria-label={presenceStatus(presence, activeThread.agentKey)}
                  >●</span>
                  <span className="kb-peer-label">{activeThread.label}</span>
                </span>
              ) : activeThreadId === null ? 'Select a thread' : 'Conversation'}
            </h3>
            <button
              type="button"
              className="ghost kb-expand kb-focus-ring"
              aria-label={expanded ? 'Collapse board' : 'Expand board to full view'}
              aria-expanded={expanded}
              title={expanded ? 'Collapse (Esc)' : 'Expand — view the whole thread'}
              onClick={() => setExpanded((current) => !current)}
            >
              {expanded ? '✕ Close' : '⤢ Expand'}
            </button>
          </header>
          <div
            ref={stream}
            className="kb-stream kb-focus-ring"
            role="log"
            aria-label="Thread messages"
            aria-live="polite"
            aria-relevant="additions text"
            tabIndex={0}
            onScroll={(event) => {
              const element = event.currentTarget;
              nearBottom.current = element.scrollHeight - element.scrollTop - element.clientHeight <= 64;
            }}
          >
            <ul className="hey-messages kb-message-list">
              {activeThreadId !== null ? messageViews.map((message) => (
                <li key={message.id} className={`kb-msg${message.own ? ' kb-msg-own' : ''}`}>
                  <div className="hey-meta kb-msg-meta">
                    <span className="kb-peer">
                      <span
                        className={`kb-presence kb-presence-${presenceStatus(presence, message.agentKey)}`}
                        role="img"
                        aria-label={presenceStatus(presence, message.agentKey)}
                      >●</span>
                      <span className="kb-peer-label">{message.label}</span>
                    </span>
                    {message.time !== null ? (
                      <time className="kb-msg-time" dateTime={new Date(message.time).toISOString()}>
                        {message.timestamp}
                      </time>
                    ) : null}
                  </div>
                  {editingId === message.id ? (
                    <form
                      className="kb-msg-edit"
                      onSubmit={(event) => {
                        event.preventDefault();
                        saveEdit(message.id);
                      }}
                    >
                      <textarea
                        className="kb-composer-input kb-focus-ring"
                        rows={3}
                        value={editDraft}
                        autoFocus
                        onChange={(event) => setEditDraft(event.currentTarget.value)}
                        onKeyDown={(event) => {
                          if (event.key === 'Escape') setEditingId(null);
                          if (event.key === 'Enter' && !event.shiftKey) {
                            event.preventDefault();
                            saveEdit(message.id);
                          }
                        }}
                      />
                      <div className="kb-msg-edit-actions">
                        <button type="submit" className="kb-send kb-focus-ring">Save</button>
                        <button type="button" className="ghost kb-focus-ring" onClick={() => setEditingId(null)}>
                          Cancel
                        </button>
                      </div>
                    </form>
                  ) : (
                    <p className="kb-msg-body">
                      {message.body.split('\n').map((line, index) => (
                        <span key={index}>{index > 0 ? <br /> : null}{line}</span>
                      ))}
                    </p>
                  )}
                  <div className="kb-msg-actions" role="group" aria-label="Message actions">
                    <button
                      type="button"
                      className="ghost kb-msg-action kb-focus-ring"
                      disabled={disabled}
                      onClick={() => setReplyTo(message)}
                    >
                      Reply
                    </button>
                    {onEdit ? (
                      <button
                        type="button"
                        className="ghost kb-msg-action kb-focus-ring"
                        disabled={disabled}
                        onClick={() => {
                          setEditingId(message.id);
                          setEditDraft(message.body);
                        }}
                      >
                        Edit
                      </button>
                    ) : null}
                    {onDelete ? (
                      <button
                        type="button"
                        className="ghost kb-msg-action kb-focus-ring"
                        disabled={disabled}
                        onClick={() => onDelete(message.id)}
                      >
                        Delete
                      </button>
                    ) : null}
                  </div>
                  <div className="kb-reactions" role="group" aria-label={`React to message from ${message.label}`}>
                    {message.reactions.map((r) => (
                      <span key={r.emoji} className="kb-reaction-count" aria-label={`${r.count} ${r.emoji}`}>
                        {r.emoji} {r.count}
                      </span>
                    ))}
                    {reactions.map((reaction) => (
                      <button
                        key={reaction.emoji}
                        type="button"
                        className="kb-reaction kb-focus-ring"
                        aria-label={`React with ${reaction.label}`}
                        title={reaction.label}
                        disabled={disabled}
                        onClick={() => onReact(message.id, reaction.emoji)}
                      >
                        <span aria-hidden="true">{reaction.emoji}</span>
                      </button>
                    ))}
                  </div>
                </li>
              )) : null}
            </ul>
            {activeThreadId === null ? (
              <p className="muted kb-empty">Choose a thread to read and send messages.</p>
            ) : !messages.length ? (
              <p className="muted kb-empty">No messages yet.</p>
            ) : null}
          </div>
          <form
            className="kb-composer"
            aria-label="Send a message"
            onSubmit={(event) => {
              event.preventDefault();
              send();
            }}
          >
            {replyTo ? (
              <div className="kb-reply-banner" role="status">
                <span className="muted">
                  Replying to {replyTo.label}: {replyTo.body.slice(0, 80)}
                  {replyTo.body.length > 80 ? '…' : ''}
                </span>
                <button
                  type="button"
                  className="ghost kb-focus-ring"
                  aria-label="Cancel reply"
                  onClick={() => setReplyTo(null)}
                >
                  ×
                </button>
              </div>
            ) : null}
            <textarea
              className="kb-composer-input kb-focus-ring"
              aria-label="Message; Enter sends, Shift+Enter adds a new line"
              rows={2}
              value={draft}
              disabled={composerDisabled}
              onChange={(event) => {
                if (activeThreadId !== null) {
                  const body = event.currentTarget.value;
                  setDrafts((current) => ({ ...current, [activeThreadId]: body }));
                }
              }}
              onKeyDown={(event) => {
                if (
                  event.key === 'Enter'
                  && !event.shiftKey
                  && !event.nativeEvent.isComposing
                  && event.nativeEvent.keyCode !== 229
                ) {
                  event.preventDefault();
                  send();
                }
              }}
            />
            <div className="kb-composer-actions">
              <span className="muted kb-composer-hint">Enter to send · Shift+Enter for a new line</span>
              <button type="submit" className="kb-send kb-focus-ring" disabled={composerDisabled || !draft.trim()}>
                Send
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
  // Portal to <body> in expanded mode — ancestors with transform/filter trap
  // position:fixed, which confined the overlay to the rail. Portaling also
  // drops the .hey-panel descendant overrides, restoring the two-pane layout.
  return expanded ? createPortal(board, document.body) : board;
}
