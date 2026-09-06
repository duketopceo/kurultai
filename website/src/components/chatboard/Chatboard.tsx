import { useEffect, useRef, useState } from 'react';
import type { MessageVM, ThreadVM } from './chatboard-mapping';
export type ChatboardProps = {
  threads: ThreadVM[];
  messages: MessageVM[];
  activeThreadId: string | null;
  presence: Record<string, string>;
  onOpenThread(id: string): void;
  onSend(body: string): void;
  onReact(messageId: string, emoji: string): void;
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
export function Chatboard({
  threads,
  messages,
  activeThreadId,
  presence,
  onOpenThread,
  onSend,
  onReact,
  disabled = false,
}: ChatboardProps) {
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [focusedThreadId, setFocusedThreadId] = useState<string | null>(null);
  const threadButtons = useRef(new Map<string, HTMLButtonElement>());
  const stream = useRef<HTMLDivElement>(null);
  const nearBottom = useRef(true);
  const previousThreadId = useRef<string | null>(null);
  const threadViews = threads.map((thread) => ({
    id: text(thread, 'id'),
    label: text(thread, 'peerLabel', 'title', 'name') || text(thread, 'id'),
    agentKey: text(thread, 'peerKey', 'agentKey', 'peerAgentKey'),
    preview: text(thread, 'preview', 'lastSnippet', 'lastMessagePreview'),
  }));
  const messageViews = messages.map((message, index) => {
    const timestamp = text(message, 'createdAt', 'timestamp', 'created_at');
    const parsedTime = Date.parse(timestamp);
    return {
      id: text(message, 'id'),
      label: text(message, 'senderLabel', 'authorLabel', 'peerLabel', 'agentLabel') || 'Agent',
      agentKey: text(message, 'agentKey', 'senderKey', 'authorKey'),
      body: text(message, 'body', 'content'),
      timestamp,
      time: Number.isFinite(parsedTime) ? parsedTime : null,
      own: field(message, 'isOwn') === true,
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
  function send() {
    const body = draft.trim();
    if (composerDisabled || activeThreadId === null || !body) {
      return;
    }
    onSend(body);
    setDrafts((current) => ({ ...current, [activeThreadId]: '' }));
  }
  return (
    <section className="panel chrome-panel hey-panel kb-chatboard" aria-label="Agent message board">
      <header className="panel-head">
        <h2>Hey board</h2>
      </header>
      <p className="muted hey-caption">Active WIP / agent coordination — not long-term memory.</p>
      <div className="kb-panes" style={{ display: 'grid', gridTemplateColumns: 'minmax(0, 1fr) minmax(0, 3fr)' }}>
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
                  <p className="kb-msg-body">
                    {message.body.split('\n').map((line, index) => (
                      <span key={index}>{index > 0 ? <br /> : null}{line}</span>
                    ))}
                  </p>
                  <div className="kb-reactions" role="group" aria-label={`React to message from ${message.label}`}>
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
    </section>
  );
}
