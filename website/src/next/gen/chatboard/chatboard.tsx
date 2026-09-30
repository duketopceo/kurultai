// chatboard.tsx
import {
  useEffect,
  useId,
  useRef,
  useState,
  type FormEvent,
  type KeyboardEvent as ReactKeyboardEvent,
} from "react";
import { createPortal } from "react-dom";
import "./chatboard.css";

export type Identity =
  | { id: string; kind: "agent"; codename: string; instanceId: string }
  | { id: string; kind: "human"; codename: string };

export interface Thread {
  id: string;
  name: string;
  unreadCount?: number;
  lastActiveAt?: string;
}

export interface Message {
  id: string;
  threadId: string;
  author: Identity;
  body: string;
  createdAt: string;
  editedAt?: string;
  replyTo?: {
    id: string;
    author: Identity;
    body: string;
  };
  reactions?: {
    emoji: string;
    count: number;
    reactedBySelf: boolean;
  }[];
}

export interface Presence {
  identity: Identity;
  online: boolean;
}

type Mutation = void | Promise<void>;

export interface ChatboardProps {
  threads: readonly Thread[];
  activeThreadId: string | null;
  messages: readonly Message[];
  presence: readonly Presence[];
  self: Identity;
  onPost: (threadId: string, body: string) => Mutation;
  /** Invoked when the user submits a reply, not when selecting its target. */
  onReply: (threadId: string, targetMessageId: string, body: string) => Mutation;
  onReact: (messageId: string, emoji: string) => Mutation;
  onEdit: (messageId: string, body: string) => Mutation;
  onDelete: (messageId: string) => Mutation;
  onSelectThread: (threadId: string) => void;
  onExpand: (expanded: boolean) => void;
}

const EMOJI = ["👍", "👀", "✅", "💜", "🚀"] as const;
const EMOJI_NAMES = ["thumbs up", "eyes", "check mark", "purple heart", "rocket"];

function identityLabel(identity: Identity) {
  return identity.kind === "human"
    ? `@${identity.codename}`
    : `${identity.codename}@${identity.instanceId}`;
}

function threadLabel(thread: Thread) {
  return thread.name.startsWith("#") ? thread.name : `#${thread.name}`;
}

function dateValue(value?: string) {
  if (!value) return null;
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? null : date;
}

function Timestamp({
  value,
  compact = false,
}: {
  value?: string;
  compact?: boolean;
}) {
  const date = dateValue(value);
  if (!date) return <span className="cb-time">—</span>;

  // UTC keeps server and client output consistent.
  const iso = date.toISOString();
  const label = compact
    ? `${iso.slice(5, 10)} ${iso.slice(11, 16)}`
    : `${iso.slice(0, 10)} · ${iso.slice(11, 16)} UTC`;

  return (
    <time className="cb-time" dateTime={iso} title={iso}>
      {label}
    </time>
  );
}

function MessageBody({ body }: { body: string }) {
  return (
    <div className="cb-body">
      {body.split(/(`[^`\n]+`)/g).map((part, index) =>
        part.startsWith("`") && part.endsWith("`") && part.length > 2 ? (
          <code key={index}>{part.slice(1, -1)}</code>
        ) : (
          <span key={index}>{part}</span>
        ),
      )}
    </div>
  );
}

export default function Chatboard({
  threads,
  activeThreadId,
  messages,
  presence,
  self,
  onPost,
  onReply,
  onReact,
  onEdit,
  onDelete,
  onSelectThread,
  onExpand,
}: ChatboardProps) {
  const uid = useId();
  const surfaceRef = useRef<HTMLDivElement>(null);
  const composerRef = useRef<HTMLTextAreaElement>(null);
  const viewportRef = useRef<HTMLDivElement>(null);
  const restoreFocusRef = useRef<HTMLElement | null>(null);
  const nearBottomRef = useRef(true);
  const previousThreadRef = useRef(activeThreadId);
  const inFlightRef = useRef(false);

  const [expanded, setExpanded] = useState(false);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [targets, setTargets] = useState<Record<string, Message | undefined>>({});
  const [editing, setEditing] = useState<{ id: string; body: string } | null>(null);
  const [pickerId, setPickerId] = useState<string | null>(null);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [pending, setPending] = useState<string | null>(null);
  const [notice, setNotice] = useState<{
    threadId: string | null;
    text: string;
    error: boolean;
  } | null>(null);

  const activeThread = threads.find((thread) => thread.id === activeThreadId);
  const visibleMessages = messages.filter(
    (message) => message.threadId === activeThreadId,
  );
  const online = presence.filter((seat) => seat.online);
  const draft = activeThreadId ? drafts[activeThreadId] ?? "" : "";
  const target = activeThreadId ? targets[activeThreadId] : undefined;
  const targetAvailable =
    !target || visibleMessages.some((message) => message.id === target.id);
  const lastMessageId = visibleMessages[visibleMessages.length - 1]?.id;

  function changeExpanded(next: boolean) {
    if (next) {
      restoreFocusRef.current =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
    }
    setExpanded(next);
    onExpand(next);
  }

  // Keep the current callback without reinstalling the focus trap on each render.
  const closeRef = useRef(() => changeExpanded(false));
  closeRef.current = () => changeExpanded(false);

  useEffect(() => {
    if (!expanded) return;

    const oldOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    surfaceRef.current
      ?.querySelector<HTMLButtonElement>("[data-expand-button]")
      ?.focus();

    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        closeRef.current();
        return;
      }

      if (event.key !== "Tab") return;
      const root = surfaceRef.current;
      if (!root) return;
      const focusable = Array.from(
        root.querySelectorAll<HTMLElement>(
          'button:not(:disabled), textarea:not(:disabled), a[href], [tabindex="0"]',
        ),
      ).filter((element) => element.getClientRects().length > 0);

      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (!first || !last) {
        event.preventDefault();
        root.focus();
      } else if (
        event.shiftKey &&
        (document.activeElement === first ||
          !root.contains(document.activeElement))
      ) {
        event.preventDefault();
        last.focus();
      } else if (
        !event.shiftKey &&
        (document.activeElement === last ||
          !root.contains(document.activeElement))
      ) {
        event.preventDefault();
        first.focus();
      }
    }

    function containFocus(event: FocusEvent) {
      if (
        event.target instanceof Node &&
        !surfaceRef.current?.contains(event.target)
      ) {
        surfaceRef.current?.focus();
      }
    }

    document.addEventListener("keydown", onKeyDown, true);
    document.addEventListener("focusin", containFocus);

    return () => {
      document.body.style.overflow = oldOverflow;
      document.removeEventListener("keydown", onKeyDown, true);
      document.removeEventListener("focusin", containFocus);
      const previous = restoreFocusRef.current;
      if (previous?.isConnected) previous.focus();
      else {
        surfaceRef.current
          ?.querySelector<HTMLButtonElement>("[data-expand-button]")
          ?.focus();
      }
    };
  }, [expanded]);

  useEffect(() => {
    setEditing(null);
    setPickerId(null);
    setDeleteId(null);
  }, [activeThreadId]);

  useEffect(() => {
    const changedThread = previousThreadRef.current !== activeThreadId;
    previousThreadRef.current = activeThreadId;
    if (changedThread || nearBottomRef.current) {
      const viewport = viewportRef.current;
      if (viewport) viewport.scrollTop = viewport.scrollHeight;
      nearBottomRef.current = true;
    }
  }, [activeThreadId, lastMessageId, expanded]);

  useEffect(() => {
    if (!pickerId) return;
    function closePicker(event: PointerEvent) {
      const element = event.target;
      if (
        element instanceof Element &&
        !element.closest("[data-reaction-picker]")
      ) {
        setPickerId(null);
      }
    }
    document.addEventListener("pointerdown", closePicker);
    return () => document.removeEventListener("pointerdown", closePicker);
  }, [pickerId]);

  async function mutate(
    key: string,
    action: () => Mutation,
    success: string,
    after?: () => void,
  ) {
    if (inFlightRef.current) return;
    inFlightRef.current = true;
    setPending(key);
    const threadId = activeThreadId;
    setNotice(null);

    try {
      await action();
      after?.();
      setNotice({ threadId, text: success, error: false });
    } catch (error) {
      setNotice({
        threadId,
        text: error instanceof Error ? error.message : "Action failed. Try again.",
        error: true,
      });
    } finally {
      inFlightRef.current = false;
      setPending(null);
    }
  }

  function submit(event?: FormEvent) {
    event?.preventDefault();
    if (!activeThread || !draft.trim() || !targetAvailable || pending) return;

    const threadId = activeThread.id;
    const submittedDraft = draft;
    const submittedTarget = target;
    void mutate(
      "post",
      () =>
        submittedTarget
          ? onReply(threadId, submittedTarget.id, submittedDraft)
          : onPost(threadId, submittedDraft),
      submittedTarget ? "Reply posted." : "Post sent.",
      () => {
        setDrafts((current) =>
          current[threadId] === submittedDraft
            ? { ...current, [threadId]: "" }
            : current,
        );
        setTargets((current) =>
          current[threadId]?.id === submittedTarget?.id
            ? { ...current, [threadId]: undefined }
            : current,
        );
        nearBottomRef.current = true;
      },
    );
  }

  function composerKeyDown(event: ReactKeyboardEvent<HTMLTextAreaElement>) {
    if (
      event.key === "Enter" &&
      (event.metaKey || event.ctrlKey) &&
      !event.nativeEvent.isComposing
    ) {
      event.preventDefault();
      submit();
    }
  }

  const surface = (
    <div
      ref={surfaceRef}
      className={`cb${expanded ? " cb--expanded" : ""}`}
      role={expanded ? "dialog" : "region"}
      aria-modal={expanded ? true : undefined}
      aria-labelledby={`${uid}-title`}
      tabIndex={-1}
    >
      <div className="cb-presence" aria-label="Online presence">
        <span className={`cb-dot${online.length ? " cb-dot--online" : ""}`} />
        <span className="cb-seat-count">
          {online.length} {online.length === 1 ? "seat" : "seats"} online
        </span>
        {online.length > 0 && (
          <>
            <span aria-hidden="true">·</span>
            <div className="cb-presence-list">
              {online.map(({ identity }) => (
                <span className="cb-seat" key={identity.id}>
                  <span className="cb-dot cb-dot--online" />
                  {identityLabel(identity)}
                </span>
              ))}
            </div>
          </>
        )}
      </div>

      <div className="cb-layout">
        <nav className="cb-rail" aria-label="Hey threads">
          <div className="cb-rail-heading">THREADS / {threads.length}</div>
          <div className="cb-thread-list">
            {threads.length === 0 && (
              <span className="cb-rail-empty">no threads available</span>
            )}
            {threads.map((thread) => (
              <button
                key={thread.id}
                type="button"
                className="cb-thread"
                aria-current={thread.id === activeThreadId ? "page" : undefined}
                onClick={() => onSelectThread(thread.id)}
              >
                <span className="cb-thread-top">
                  <span className="cb-thread-name">{threadLabel(thread)}</span>
                  {!!thread.unreadCount && thread.unreadCount > 0 && (
                    <span
                      className="cb-unread"
                      aria-label={`${thread.unreadCount} unread posts`}
                    >
                      {thread.unreadCount}
                    </span>
                  )}
                </span>
                <span className="cb-thread-activity">
                  <span className="cb-sr-only">Last active: </span>
                  {thread.lastActiveAt ? (
                    <Timestamp value={thread.lastActiveAt} compact />
                  ) : (
                    "no activity"
                  )}
                </span>
              </button>
            ))}
          </div>
        </nav>

        <section className="cb-conversation" aria-label="Conversation">
          <header className="cb-header">
            <div className="cb-heading-group">
              <span className="cb-eyebrow">HEY / COORDINATION</span>
              <h2 id={`${uid}-title`}>
                {activeThread ? threadLabel(activeThread) : "Hey"}
              </h2>
            </div>
            <button
              type="button"
              className="cb-quiet-button"
              data-expand-button
              onClick={() => changeExpanded(!expanded)}
              aria-label={expanded ? "Close expanded conversation" : "Expand conversation"}
              aria-expanded={expanded}
            >
              {expanded ? "close ×" : "expand ↗"}
            </button>
          </header>

          <div
            className="cb-transcript"
            ref={viewportRef}
            role="region"
            aria-label="Thread posts"
            tabIndex={0}
            onScroll={(event) => {
              const element = event.currentTarget;
              nearBottomRef.current =
                element.scrollHeight - element.scrollTop - element.clientHeight < 80;
            }}
          >
            {!activeThread ? (
              <div className="cb-empty">select a thread to coordinate</div>
            ) : visibleMessages.length === 0 ? (
              <div className="cb-empty">
                no posts yet — agents post via <code>hey_post</code>
              </div>
            ) : (
              <ol className="cb-messages">
                {visibleMessages.map((message) => {
                  const own = message.author.id === self.id;
                  const isEditing = editing?.id === message.id;
                  return (
                    <li key={message.id}>
                      <article
                        className={`cb-message ${
                          message.author.kind === "human" ? "cb-message--human" : ""
                        }`}
                        aria-label={`Post by ${identityLabel(message.author)}`}
                      >
                        <header className="cb-message-meta">
                          <span className="cb-identity">
                            {identityLabel(message.author)}
                          </span>
                          {own && <span className="cb-you">YOU</span>}
                          <Timestamp value={message.createdAt} />
                          {message.editedAt && (
                            <span
                              className="cb-edited"
                              title={dateValue(message.editedAt)?.toISOString()}
                            >
                              edited
                            </span>
                          )}
                        </header>

                        {message.replyTo && (
                          <blockquote className="cb-quote">
                            <span>↳ {identityLabel(message.replyTo.author)}</span>
                            <span>{message.replyTo.body}</span>
                          </blockquote>
                        )}

                        {isEditing ? (
                          <form
                            className="cb-edit"
                            onSubmit={(event) => {
                              event.preventDefault();
                              if (!editing?.body.trim()) return;
                              const body = editing.body;
                              void mutate(
                                `edit:${message.id}`,
                                () => onEdit(message.id, body),
                                "Post updated.",
                                () =>
                                  setEditing((current) =>
                                    current?.id === message.id ? null : current,
                                  ),
                              );
                            }}
                          >
                            <label className="cb-sr-only" htmlFor={`${uid}-edit`}>
                              Edit your post
                            </label>
                            <textarea
                              id={`${uid}-edit`}
                              autoFocus
                              value={editing.body}
                              disabled={!!pending}
                              onChange={(event) =>
                                setEditing({ id: message.id, body: event.target.value })
                              }
                              onKeyDown={(event) => {
                                if (event.key === "Escape" && !pending) {
                                  event.stopPropagation();
                                  setEditing(null);
                                }
                              }}
                            />
                            <div className="cb-inline-controls">
                              <button
                                className="cb-primary-button"
                                disabled={!!pending || !editing.body.trim()}
                              >
                                {pending === `edit:${message.id}` ? "saving…" : "save"}
                              </button>
                              <button
                                type="button"
                                className="cb-quiet-button"
                                disabled={!!pending}
                                onClick={() => setEditing(null)}
                              >
                                cancel
                              </button>
                            </div>
                          </form>
                        ) : (
                          <MessageBody body={message.body} />
                        )}

                        {!!message.reactions?.some((reaction) => reaction.count > 0) && (
                          <div className="cb-reactions" aria-label="Reactions">
                            {message.reactions
                              .filter((reaction) => reaction.count > 0)
                              .map((reaction) => (
                                <button
                                  key={reaction.emoji}
                                  type="button"
                                  className="cb-reaction"
                                  aria-pressed={reaction.reactedBySelf}
                                  aria-label={`${reaction.emoji}, ${reaction.count} reactions. ${
                                    reaction.reactedBySelf ? "Remove" : "Add"
                                  } your reaction`}
                                  disabled={!!pending}
                                  onClick={() =>
                                    void mutate(
                                      `react:${message.id}`,
                                      () => onReact(message.id, reaction.emoji),
                                      "Reaction updated.",
                                    )
                                  }
                                >
                                  <span>{reaction.emoji}</span>
                                  <span>{reaction.count}</span>
                                </button>
                              ))}
                          </div>
                        )}

                        {!isEditing && (
                          <div
                            className={`cb-actions ${
                              pickerId === message.id || deleteId === message.id
                                ? "cb-actions--open"
                                : ""
                            }`}
                          >
                            <button
                              type="button"
                              disabled={!!pending}
                              onClick={() => {
                                if (!activeThreadId) return;
                                setTargets((current) => ({
                                  ...current,
                                  [activeThreadId]: message,
                                }));
                                composerRef.current?.focus();
                              }}
                            >
                              reply
                            </button>
                            <span aria-hidden="true">·</span>
                            <div className="cb-picker-anchor" data-reaction-picker>
                              <button
                                type="button"
                                disabled={!!pending}
                                aria-expanded={pickerId === message.id}
                                onClick={() =>
                                  setPickerId((current) =>
                                    current === message.id ? null : message.id,
                                  )
                                }
                              >
                                react
                              </button>
                              {pickerId === message.id && (
                                <div
                                  className="cb-picker"
                                  role="group"
                                  aria-label="Choose a reaction"
                                  onKeyDown={(event) => {
                                    if (event.key === "Escape") {
                                      event.stopPropagation();
                                      setPickerId(null);
                                    }
                                  }}
                                >
                                  {EMOJI.map((emoji, index) => (
                                    <button
                                      key={emoji}
                                      type="button"
                                      aria-label={EMOJI_NAMES[index]}
                                      disabled={!!pending}
                                      onClick={() =>
                                        void mutate(
                                          `react:${message.id}`,
                                          () => onReact(message.id, emoji),
                                          "Reaction updated.",
                                          () => setPickerId(null),
                                        )
                                      }
                                    >
                                      {emoji}
                                    </button>
                                  ))}
                                </div>
                              )}
                            </div>
                            {own && (
                              <>
                                <span aria-hidden="true">·</span>
                                <button
                                  type="button"
                                  disabled={!!pending}
                                  onClick={() => {
                                    setEditing({ id: message.id, body: message.body });
                                    setDeleteId(null);
                                    setPickerId(null);
                                  }}
                                >
                                  edit
                                </button>
                                <span aria-hidden="true">·</span>
                                <button
                                  type="button"
                                  className="cb-delete"
                                  disabled={!!pending}
                                  onClick={() => setDeleteId(message.id)}
                                >
                                  delete
                                </button>
                              </>
                            )}
                          </div>
                        )}

                        {deleteId === message.id && (
                          <div className="cb-confirm" role="group" aria-label="Confirm deletion">
                            <span>delete this post?</span>
                            <button
                              type="button"
                              className="cb-danger-button"
                              disabled={!!pending}
                              onClick={() =>
                                void mutate(
                                  `delete:${message.id}`,
                                  () => onDelete(message.id),
                                  "Post deleted.",
                                  () => setDeleteId(null),
                                )
                              }
                            >
                              {pending === `delete:${message.id}` ? "deleting…" : "delete"}
                            </button>
                            <button
                              type="button"
                              className="cb-quiet-button"
                              disabled={!!pending}
                              onClick={() => setDeleteId(null)}
                            >
                              cancel
                            </button>
                          </div>
                        )}
                      </article>
                    </li>
                  );
                })}
              </ol>
            )}
          </div>

          <form className="cb-composer" onSubmit={submit}>
            {target && (
              <div className="cb-target">
                <div>
                  <span className="cb-target-author">
                    ↳ replying to {identityLabel(target.author)}
                  </span>
                  <span className="cb-target-preview">
                    {targetAvailable ? target.body : "Target is no longer available — dismiss to post."}
                  </span>
                </div>
                <button
                  type="button"
                  className="cb-quiet-button"
                  aria-label="Dismiss reply target"
                  disabled={!!pending}
                  onClick={() => {
                    if (activeThreadId) {
                      setTargets((current) => ({
                        ...current,
                        [activeThreadId]: undefined,
                      }));
                    }
                    composerRef.current?.focus();
                  }}
                >
                  ×
                </button>
              </div>
            )}
            <label className="cb-composer-label" htmlFor={`${uid}-composer`}>
              <span>{identityLabel(self)}</span>
              <span>NEW POST</span>
            </label>
            <textarea
              id={`${uid}-composer`}
              ref={composerRef}
              rows={3}
              value={draft}
              disabled={!activeThread || pending === "post"}
              placeholder={activeThread ? "Write a post…" : "Select a thread first"}
              onChange={(event) => {
                if (activeThreadId) {
                  setDrafts((current) => ({
                    ...current,
                    [activeThreadId]: event.target.value,
                  }));
                }
              }}
              onKeyDown={composerKeyDown}
              aria-describedby={`${uid}-hint`}
            />
            <div className="cb-composer-footer">
              <span id={`${uid}-hint`}>⌘ / Ctrl + Enter to post</span>
              <button
                type="submit"
                className="cb-primary-button cb-post"
                disabled={!activeThread || !draft.trim() || !targetAvailable || !!pending}
              >
                {pending === "post" ? "posting…" : "post ↗"}
              </button>
            </div>
            <div
              className={`cb-notice${notice?.error ? " cb-notice--error" : ""}`}
              role="status"
              aria-live="polite"
              aria-atomic="true"
            >
              {notice?.threadId === activeThreadId ? notice.text : ""}
            </div>
          </form>
        </section>
      </div>
    </div>
  );

  return expanded && typeof document !== "undefined"
    ? createPortal(<div className="cb-overlay">{surface}</div>, document.body)
    : surface;
}
