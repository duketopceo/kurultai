// worksurface.tsx
import {
  useEffect,
  useId,
  useRef,
  useState,
  type FormEvent,
  type KeyboardEvent,
  type ReactNode,
} from "react";
import "./worksurface.css";

export type Loadable<T> =
  | { status: "loading" }
  | { status: "error"; error: string }
  | { status: "ready"; data: T };

export interface Thread {
  id: string;
  name: string;
  unread: boolean;
  lastActiveAt: string; // ISO timestamp
}

export interface Message {
  id: string;
  codename: string;
  instanceId: string;
  createdAt: string;
  body: string;
  replyToId?: string;
  canEdit: boolean;
  canDelete: boolean;
  reactions?: readonly {
    emoji: string;
    count: number;
    reactedByMe: boolean;
  }[];
}

export type TaskStatus = "queued" | "doing" | "done";

export interface Task {
  id: string;
  title: string;
  status: TaskStatus;
  assignee?: string;
  priority?: string;
}

export interface HeyProps {
  presence: {
    codename: string;
    instanceId: string;
    status: "online" | "away" | "offline";
    workspace: string;
  } | null;
  threads: Loadable<readonly Thread[]>;
  conversations: Readonly<
    Record<string, Loadable<readonly Message[]> | undefined>
  >;
  board: Loadable<readonly Task[]>;
  onSelectThread: (threadId: string) => void;
  onCreateThread: () => Promise<void>;
  onCreateTask: () => Promise<void>;
  onRetry: (
    section: "threads" | "conversation" | "board",
    threadId?: string,
  ) => void;
  onSend: (
    threadId: string,
    body: string,
    replyToId?: string,
  ) => Promise<void>;
  onEditMessage: (
    threadId: string,
    messageId: string,
    body: string,
  ) => Promise<void>;
  onDeleteMessage: (threadId: string, messageId: string) => Promise<void>;
  onReact: (
    threadId: string,
    messageId: string,
    emoji: string,
  ) => Promise<void>;
  onMoveTask: (taskId: string, status: TaskStatus) => Promise<void>;
}

export interface MemoryEvent {
  id: string;
  event: string;
  title: string;
  source: string;
}

export interface PulseSnapshot {
  stats: {
    loaded: number;
    total: number;
    hot: number;
    warm: number;
    cold: number;
    trusted: number;
  };
  events: readonly MemoryEvent[];
}

export interface PulseProps {
  data: Loadable<PulseSnapshot>;
  onRetry: () => void;
  /**
   * Emit authoritative snapshots, newest events first.
   * Reconnection must emit the current snapshot, including events missed
   * while paused. Return a function that closes the subscription.
   */
  subscribe: (
    onSnapshot: (snapshot: PulseSnapshot) => void,
    onError: (error: Error) => void,
  ) => () => void;
}

export interface ClassInstance {
  id: string;
  label: string;
}

export interface OntologyClass {
  id: string;
  name: string;
  instanceCount: number;
  instances: Loadable<readonly ClassInstance[]>;
}

export type SixClasses = readonly [
  OntologyClass,
  OntologyClass,
  OntologyClass,
  OntologyClass,
  OntologyClass,
  OntologyClass,
];

export interface Proposal {
  id: string;
  description: string;
  status: "pending" | "approved" | "rejected";
}

export interface OntologyProps {
  classes: Loadable<SixClasses>;
  proposals: Loadable<readonly Proposal[]>;
  onExpandClass: (classId: string) => void;
  onRetryClass: (classId: string) => void;
  onRetry: (section: "classes" | "proposals") => void;
  onDecide: (
    proposalId: string,
    decision: "approved" | "rejected",
  ) => Promise<void>;
}

export interface AskProps {
  initialAnswer?: string;
  onAsk: (question: string, signal: AbortSignal) => Promise<string>;
}

export interface Atom {
  id: string;
  title: string;
  tier: "hot" | "warm" | "cold";
  indexedAt: string | null;
}

export interface StoreProps {
  atoms: Loadable<readonly Atom[]>;
  onRetry: () => void;
}

export interface WorkSurfaceProps {
  hey: HeyProps;
  pulse: PulseProps;
  ontology: OntologyProps;
  ask: AskProps;
  store: StoreProps;
  defaultTab?: Tab;
}

type Tab = "hey" | "pulse" | "ontology" | "ask" | "store";

const TABS: readonly Tab[] = ["hey", "pulse", "ontology", "ask", "store"];
const STATUSES: readonly TaskStatus[] = ["queued", "doing", "done"];

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "Request failed. Try again.";
}

function useMutation() {
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const busy = useRef(false);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  async function run(action: () => Promise<void>): Promise<boolean> {
    if (busy.current) return false;
    busy.current = true;
    setPending(true);
    setError(null);

    try {
      await action();
      return true;
    } catch (error) {
      if (mounted.current) setError(errorMessage(error));
      return false;
    } finally {
      busy.current = false;
      if (mounted.current) setPending(false);
    }
  }

  return { pending, error, run };
}

function Failure({ children }: { children: ReactNode }) {
  return (
    <p className="ws-error" role="alert">
      {children}
    </p>
  );
}

function Hint({ children }: { children: ReactNode }) {
  return <p className="ws-hint">{children}</p>;
}

function Resource<T>({
  value,
  onRetry,
  children,
}: {
  value: Loadable<T>;
  onRetry: () => void;
  children: (data: T) => ReactNode;
}) {
  if (value.status === "loading") {
    return (
      <p className="ws-state" role="status">
        <span className="ws-loading-dot" aria-hidden="true" />
        Loading…
      </p>
    );
  }

  if (value.status === "error") {
    return (
      <div className="ws-state">
        <Failure>{value.error}</Failure>
        <button type="button" className="ws-link" onClick={onRetry}>
          Try again ↻
        </button>
      </div>
    );
  }

  return <>{children(value.data)}</>;
}

function Time({ value }: { value: string }) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return <span className="ws-time">Unknown time</span>;
  }

  return (
    <time className="ws-time" dateTime={value} title={date.toLocaleString()}>
      {date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
    </time>
  );
}

/**
 * Data stays authoritative in the host. Mutation callbacks must reject on
 * failure and publish updated props after committing; no seeded or optimistic
 * messages, task moves, counts, or proposal decisions are manufactured here.
 */
export default function WorkSurface({
  hey,
  pulse,
  ontology,
  ask,
  store,
  defaultTab = "hey",
}: WorkSurfaceProps) {
  const [active, setActive] = useState<Tab>(defaultTab);
  const id = useId();
  const tabRefs = useRef<Partial<Record<Tab, HTMLButtonElement | null>>>({});

  function onTabKey(event: KeyboardEvent<HTMLButtonElement>, tab: Tab) {
    const index = TABS.indexOf(tab);
    let next = index;

    if (event.key === "ArrowRight") next = (index + 1) % TABS.length;
    else if (event.key === "ArrowLeft")
      next = (index - 1 + TABS.length) % TABS.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = TABS.length - 1;
    else return;

    event.preventDefault();
    setActive(TABS[next]);
    tabRefs.current[TABS[next]]?.focus();
    tabRefs.current[TABS[next]]?.scrollIntoView({
      block: "nearest",
      inline: "nearest",
    });
  }

  const content: Record<Tab, ReactNode> = {
    hey: <Hey {...hey} />,
    pulse: <Pulse {...pulse} />,
    ontology: <Ontology {...ontology} />,
    ask: <Ask {...ask} />,
    store: <Store {...store} />,
  };

  return (
    <section className="ws" aria-label="WorkSurface console">
      <header className="ws-chrome">
        <div className="ws-brand">
          <span className="ws-brand-mark" aria-hidden="true">⌘</span>
          WORKSURFACE
          <span className="ws-brand-detail">/ COORDINATION CONSOLE</span>
        </div>
        <span className="ws-chrome-note">KURULTAI</span>
      </header>

      <div className="ws-tab-scroll">
        <div className="ws-tabs" role="tablist" aria-label="Console views">
          {TABS.map((tab) => (
            <button
              key={tab}
              ref={(element) => {
                tabRefs.current[tab] = element;
              }}
              id={`${id}-tab-${tab}`}
              type="button"
              role="tab"
              aria-selected={active === tab}
              aria-controls={`${id}-panel-${tab}`}
              tabIndex={active === tab ? 0 : -1}
              className="ws-tab"
              onClick={() => setActive(tab)}
              onKeyDown={(event) => onTabKey(event, tab)}
            >
              {tab}
            </button>
          ))}
        </div>
      </div>

      {/* Keep panels mounted: drafts, selected threads, and answers survive tabs. */}
      {TABS.map((tab) => (
        <section
          key={tab}
          id={`${id}-panel-${tab}`}
          role="tabpanel"
          aria-labelledby={`${id}-tab-${tab}`}
          hidden={active !== tab}
          tabIndex={0}
          className="ws-panel"
        >
          {content[tab]}
        </section>
      ))}
    </section>
  );
}

function Hey(props: HeyProps) {
  const [mode, setMode] = useState<"conversation" | "kanban">("conversation");
  const [selected, setSelected] = useState<string | null>(null);
  const create = useMutation();
  const presence = props.presence;
  const thread =
    props.threads.status === "ready"
      ? props.threads.data.find((item) => item.id === selected)
      : undefined;

  useEffect(() => {
    if (
      selected &&
      props.threads.status === "ready" &&
      !props.threads.data.some((item) => item.id === selected)
    ) {
      setSelected(null);
    }
  }, [props.threads, selected]);

  return (
    <div className="ws-hey">
      <div className="ws-toolbar">
        <div className="ws-presence">
          <span
            className="ws-presence-dot"
            data-status={presence?.status ?? "unknown"}
            aria-hidden="true"
          />
          {presence ? (
            <span>
              <strong>{presence.codename}@{presence.instanceId}</strong>{" "}
              {presence.status}
              <span className="ws-muted"> · {presence.workspace}</span>
            </span>
          ) : (
            <span className="ws-muted">Presence unavailable</span>
          )}
        </div>

        <div className="ws-segment" role="group" aria-label="Coordination view">
          {(["conversation", "kanban"] as const).map((value) => (
            <button
              key={value}
              type="button"
              aria-pressed={mode === value}
              onClick={() => setMode(value)}
            >
              {value}
            </button>
          ))}
        </div>
      </div>

      {mode === "kanban" ? (
        <Kanban {...props} />
      ) : (
        <div className={`ws-chat-layout ${selected ? "has-selection" : ""}`}>
          <aside className="ws-threads" aria-label="Threads">
            <div className="ws-section-heading">
              <h2>Threads</h2>
              <button
                type="button"
                className="ws-link"
                disabled={create.pending}
                onClick={() => void create.run(props.onCreateThread)}
              >
                {create.pending ? "Opening…" : "+ New"}
              </button>
            </div>
            {create.error && <Failure>{create.error}</Failure>}
            <Resource value={props.threads} onRetry={() => props.onRetry("threads")}>
              {(threads) =>
                threads.length === 0 ? (
                  <Hint>No threads yet — use + New to start coordination.</Hint>
                ) : (
                  <ul className="ws-thread-list">
                    {threads.map((item) => (
                      <li key={item.id}>
                        <button
                          type="button"
                          className="ws-thread"
                          aria-current={selected === item.id ? "true" : undefined}
                          onClick={() => {
                            setSelected(item.id);
                            props.onSelectThread(item.id);
                          }}
                        >
                          <span className="ws-thread-name">
                            <span className="ws-unread" data-unread={item.unread}>
                              <span className="ws-sr-only">
                                {item.unread ? "Unread: " : ""}
                              </span>
                            </span>
                            {item.name}
                          </span>
                          <Time value={item.lastActiveAt} />
                        </button>
                      </li>
                    ))}
                  </ul>
                )
              }
            </Resource>
          </aside>

          <div className="ws-conversation">
            {selected ? (
              <>
                <div className="ws-section-heading">
                  <div className="ws-thread-heading">
                    <button
                      type="button"
                      className="ws-link ws-back"
                      onClick={() => setSelected(null)}
                    >
                      ← Threads
                    </button>
                    <h2>{thread?.name ?? "Conversation"}</h2>
                  </div>
                  <span className="ws-eyebrow">Transcript</span>
                </div>
                <Conversation key={selected} {...props} threadId={selected} />
              </>
            ) : (
              <Hint>Select a thread or create one to start a conversation.</Hint>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function Conversation(props: HeyProps & { threadId: string }) {
  const [body, setBody] = useState("");
  const [replyTo, setReplyTo] = useState<Message | null>(null);
  const send = useMutation();
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const textareaId = useId();
  const messages = props.conversations[props.threadId] ?? { status: "loading" };

  async function submit(event: FormEvent) {
    event.preventDefault();
    const text = body.trim();
    if (!text) return;
    const ok = await send.run(() =>
      props.onSend(props.threadId, text, replyTo?.id),
    );
    if (ok) {
      setBody("");
      setReplyTo(null);
      inputRef.current?.focus();
    }
  }

  return (
    <>
      <div className="ws-transcript">
        <Resource
          value={messages}
          onRetry={() => props.onRetry("conversation", props.threadId)}
        >
          {(items) =>
            items.length === 0 ? (
              <Hint>No messages yet — send the first update below.</Hint>
            ) : (
              <ol className="ws-message-list" aria-label="Conversation transcript">
                {items.map((message) => (
                  <MessageRow
                    key={message.id}
                    message={message}
                    onReply={() => {
                      setReplyTo(message);
                      inputRef.current?.focus();
                    }}
                    onEdit={(text) =>
                      props.onEditMessage(props.threadId, message.id, text)
                    }
                    onDelete={() =>
                      props.onDeleteMessage(props.threadId, message.id)
                    }
                    onReact={(emoji) =>
                      props.onReact(props.threadId, message.id, emoji)
                    }
                  />
                ))}
              </ol>
            )
          }
        </Resource>
      </div>

      <form className="ws-composer" onSubmit={submit}>
        {replyTo && (
          <div className="ws-reply-banner">
            <span>Replying to {replyTo.codename}@{replyTo.instanceId}</span>
            <button
              type="button"
              className="ws-link"
              onClick={() => setReplyTo(null)}
            >
              Cancel
            </button>
          </div>
        )}
        <label className="ws-sr-only" htmlFor={textareaId}>Message</label>
        <div className="ws-compose-row">
          <textarea
            ref={inputRef}
            id={textareaId}
            rows={2}
            value={body}
            disabled={send.pending}
            placeholder="Write an update…"
            onChange={(event) => setBody(event.target.value)}
          />
          <button
            type="submit"
            className="ws-button ws-button-primary"
            disabled={send.pending || !body.trim() || messages.status !== "ready"}
          >
            {send.pending ? "Sending…" : "Send ↗"}
          </button>
        </div>
        {send.error && <Failure>{send.error}</Failure>}
      </form>
    </>
  );
}

function MessageRow({
  message,
  onReply,
  onEdit,
  onDelete,
  onReact,
}: {
  message: Message;
  onReply: () => void;
  onEdit: (body: string) => Promise<void>;
  onDelete: () => Promise<void>;
  onReact: (emoji: string) => Promise<void>;
}) {
  const [editing, setEditing] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [draft, setDraft] = useState(message.body);
  const mutation = useMutation();

  return (
    <li className="ws-message">
      <div className="ws-message-meta">
        <strong>{message.codename}@{message.instanceId}</strong>
        <Time value={message.createdAt} />
      </div>

      {message.replyToId && (
        <span className="ws-reply-reference">↳ Reply to message {message.replyToId}</span>
      )}

      {editing ? (
        <form
          className="ws-edit-form"
          onSubmit={async (event) => {
            event.preventDefault();
            if (!draft.trim()) return;
            if (await mutation.run(() => onEdit(draft.trim()))) setEditing(false);
          }}
        >
          <textarea
            aria-label="Edit message"
            autoFocus
            rows={3}
            value={draft}
            disabled={mutation.pending}
            onChange={(event) => setDraft(event.target.value)}
          />
          <div className="ws-inline-actions">
            <button
              className="ws-button"
              type="submit"
              disabled={mutation.pending || !draft.trim()}
            >
              {mutation.pending ? "Saving…" : "Save"}
            </button>
            <button
              className="ws-link"
              type="button"
              disabled={mutation.pending}
              onClick={() => setEditing(false)}
            >
              Cancel
            </button>
          </div>
        </form>
      ) : (
        <p className="ws-message-body">{message.body}</p>
      )}

      {!!message.reactions?.length && (
        <div className="ws-reactions" aria-label="Reactions">
          {message.reactions.map((reaction) => (
            <button
              type="button"
              key={reaction.emoji}
              aria-label={`React ${reaction.emoji}, ${reaction.count} reactions`}
              aria-pressed={reaction.reactedByMe}
              disabled={mutation.pending}
              onClick={() => void mutation.run(() => onReact(reaction.emoji))}
            >
              {reaction.emoji} {reaction.count}
            </button>
          ))}
        </div>
      )}

      <div className="ws-message-actions">
        <button type="button" onClick={onReply}>Reply</button>
        <button
          type="button"
          disabled={mutation.pending}
          aria-label="React with thumbs up"
          onClick={() => void mutation.run(() => onReact("👍"))}
        >
          React +
        </button>
        {message.canEdit && (
          <button
            type="button"
            disabled={mutation.pending}
            onClick={() => {
              setDraft(message.body);
              setEditing(true);
              setConfirmDelete(false);
            }}
          >
            Edit
          </button>
        )}
        {message.canDelete && (
          <button
            type="button"
            disabled={mutation.pending}
            onClick={() => setConfirmDelete(true)}
          >
            Delete
          </button>
        )}
      </div>

      {confirmDelete && (
        <div className="ws-delete-confirm">
          <span>Delete this message?</span>
          <button
            type="button"
            className="ws-link ws-danger"
            disabled={mutation.pending}
            onClick={() => void mutation.run(onDelete)}
          >
            {mutation.pending ? "Deleting…" : "Delete"}
          </button>
          <button
            type="button"
            className="ws-link"
            disabled={mutation.pending}
            onClick={() => setConfirmDelete(false)}
          >
            Cancel
          </button>
        </div>
      )}
      {mutation.error && <Failure>{mutation.error}</Failure>}
    </li>
  );
}

function Kanban(props: HeyProps) {
  const mutation = useMutation();
  const create = useMutation();
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [over, setOver] = useState<TaskStatus | null>(null);

  function move(task: Task, status: TaskStatus) {
    if (task.status !== status) {
      void mutation.run(() => props.onMoveTask(task.id, status));
    }
  }

  return (
    <div className="ws-board-wrap">
      <div className="ws-section-heading">
        <h2>Coordination board</h2>
        <button
          type="button"
          className="ws-link"
          disabled={create.pending}
          onClick={() => void create.run(props.onCreateTask)}
        >
          {create.pending ? "Opening…" : "+ Task"}
        </button>
      </div>
      {mutation.error && <Failure>{mutation.error}</Failure>}
      {create.error && <Failure>{create.error}</Failure>}
      <Resource value={props.board} onRetry={() => props.onRetry("board")}>
        {(tasks) => (
          <>
            {tasks.length === 0 && (
              <Hint>No tasks yet — use + Task to queue work.</Hint>
            )}
            <div className="ws-board" aria-busy={mutation.pending}>
              {STATUSES.map((status) => {
                const column = tasks.filter((task) => task.status === status);
                return (
                  <section
                    key={status}
                    className="ws-board-column"
                    data-over={over === status}
                    aria-label={`${status} tasks`}
                    onDragOver={(event) => {
                      if (!draggedId || mutation.pending) return;
                      event.preventDefault();
                      event.dataTransfer.dropEffect = "move";
                      setOver(status);
                    }}
                    onDragLeave={(event) => {
                      if (!event.currentTarget.contains(event.relatedTarget as Node)) {
                        setOver(null);
                      }
                    }}
                    onDrop={(event) => {
                      event.preventDefault();
                      const id = event.dataTransfer.getData("application/x-ws-task");
                      const task = tasks.find((item) => item.id === id);
                      if (id === draggedId && task && !mutation.pending) move(task, status);
                      setDraggedId(null);
                      setOver(null);
                    }}
                  >
                    <h3>
                      <span className={`ws-column-dot ws-column-dot-${status}`} />
                      {status}
                      <span className="ws-count">{column.length}</span>
                    </h3>
                    <ul className="ws-task-list">
                      {column.map((task) => (
                        <li
                          key={task.id}
                          className="ws-task"
                          draggable={!mutation.pending}
                          data-dragging={draggedId === task.id}
                          onDragStart={(event) => {
                            event.dataTransfer.setData("application/x-ws-task", task.id);
                            event.dataTransfer.effectAllowed = "move";
                            setDraggedId(task.id);
                          }}
                          onDragEnd={() => {
                            setDraggedId(null);
                            setOver(null);
                          }}
                        >
                          <p>{task.title}</p>
                          {(task.assignee || task.priority) && (
                            <div className="ws-task-meta">
                              {task.assignee && <span>{task.assignee}</span>}
                              {task.priority && <span>{task.priority}</span>}
                            </div>
                          )}
                          {/* Native select is the keyboard and touch move path. */}
                          <label className="ws-task-move">
                            Move
                            <select
                              aria-label={`Move ${task.title}`}
                              value={task.status}
                              disabled={mutation.pending}
                              onChange={(event) =>
                                move(task, event.target.value as TaskStatus)
                              }
                            >
                              {STATUSES.map((value) => (
                                <option key={value} value={value}>{value}</option>
                              ))}
                            </select>
                          </label>
                        </li>
                      ))}
                    </ul>
                    {column.length === 0 && tasks.length > 0 && (
                      <Hint>Move a task here using its Move control.</Hint>
                    )}
                  </section>
                );
              })}
            </div>
          </>
        )}
      </Resource>
    </div>
  );
}

function Pulse({ data, subscribe, onRetry }: PulseProps) {
  const [live, setLive] = useState(true);
  const [snapshot, setSnapshot] = useState<Loadable<PulseSnapshot>>(data);
  const [streamError, setStreamError] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(true);
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    if (live) setSnapshot(data);
  }, [data, live]);

  useEffect(() => {
    if (!live) return;
    let disposed = false;
    let unsubscribe: (() => void) | undefined;

    setConnecting(true);
    setStreamError(null);
    try {
      unsubscribe = subscribe(
        (next) => {
          if (disposed) return;
          setSnapshot({ status: "ready", data: next });
          setConnecting(false);
          setStreamError(null);
        },
        (error) => {
          if (disposed) return;
          setConnecting(false);
          setStreamError(errorMessage(error));
        },
      );
    } catch (error) {
      setConnecting(false);
      setStreamError(errorMessage(error));
    }

    return () => {
      disposed = true;
      unsubscribe?.();
    };
  }, [live, subscribe, attempt]);

  function retry() {
    onRetry();
    setAttempt((value) => value + 1);
  }

  return (
    <div className="ws-padded">
      <div className="ws-section-heading">
        <h2>Memory telemetry</h2>
        <div className="ws-live-control">
          <span className="ws-eyebrow" role="status">
            {!live ? "Paused" : streamError ? "Disconnected" : connecting ? "Connecting…" : "Live"}
          </span>
          <button
            type="button"
            className="ws-button"
            aria-pressed={!live}
            onClick={() => setLive((value) => !value)}
          >
            {live ? "Pause stream" : "Resume stream"}
          </button>
        </div>
      </div>

      {streamError && (
        <div className="ws-stream-error">
          <Failure>Stream disconnected: {streamError}</Failure>
          {live && (
            <button type="button" className="ws-link" onClick={retry}>
              Reconnect ↻
            </button>
          )}
        </div>
      )}

      <Resource value={snapshot} onRetry={retry}>
        {({ stats, events }) => (
          <>
            <dl className="ws-stats">
              <div className="ws-stat ws-stat-main">
                <dt>Memories <span>loaded / total</span></dt>
                <dd>{stats.loaded.toLocaleString()}<small> / {stats.total.toLocaleString()}</small></dd>
              </div>
              {(["hot", "warm", "cold", "trusted"] as const).map((key) => (
                <div className="ws-stat" key={key}>
                  <dt>{key}</dt>
                  <dd>{stats[key].toLocaleString()}</dd>
                </div>
              ))}
            </dl>

            <div className="ws-stream">
              <div className="ws-stream-heading">
                <span>Event</span><span>Memory</span><span>Source</span>
              </div>
              {events.length === 0 ? (
                <Hint>No memory events yet — ingest a source to start the stream.</Hint>
              ) : (
                <ul aria-label="Memory events">
                  {events.map((event) => (
                    <li key={event.id} className="ws-stream-row">
                      <span className="ws-event">{event.event}</span>
                      <span>{event.title}</span>
                      <span className="ws-source">{event.source}</span>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </>
        )}
      </Resource>
    </div>
  );
}

function Ontology(props: OntologyProps) {
  return (
    <div className="ws-ontology">
      <aside className="ws-class-pane" aria-label="Ontology classes">
        <div className="ws-section-heading"><h2>Class tree</h2></div>
        <Resource value={props.classes} onRetry={() => props.onRetry("classes")}>
          {(classes) => (
            <ul className="ws-class-tree">
              {classes.map((item) => (
                <ClassRow
                  key={item.id}
                  item={item}
                  onExpand={() => props.onExpandClass(item.id)}
                  onRetry={() => props.onRetryClass(item.id)}
                />
              ))}
            </ul>
          )}
        </Resource>
      </aside>
      <div className="ws-proposals">
        <div className="ws-section-heading"><h2>Proposals queue</h2></div>
        <Resource value={props.proposals} onRetry={() => props.onRetry("proposals")}>
          {(proposals) =>
            proposals.length === 0 ? (
              <Hint>No proposals yet — ingest a source to discover ontology changes.</Hint>
            ) : (
              <ul className="ws-proposal-list">
                {proposals.map((proposal) => (
                  <ProposalCard
                    key={proposal.id}
                    proposal={proposal}
                    onDecide={(decision) => props.onDecide(proposal.id, decision)}
                  />
                ))}
              </ul>
            )
          }
        </Resource>
      </div>
    </div>
  );
}

function ClassRow({
  item,
  onExpand,
  onRetry,
}: {
  item: OntologyClass;
  onExpand: () => void;
  onRetry: () => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const id = useId();

  return (
    <li>
      <button
        type="button"
        className="ws-class-toggle"
        aria-expanded={expanded}
        aria-controls={id}
        onClick={() => {
          if (!expanded) onExpand();
          setExpanded((value) => !value);
        }}
      >
        <span className="ws-chevron" data-open={expanded} aria-hidden="true">›</span>
        <span>{item.name}</span>
        <span className="ws-count">{item.instanceCount.toLocaleString()}</span>
      </button>
      <div id={id} hidden={!expanded} className="ws-class-children">
        {expanded && (
          <Resource value={item.instances} onRetry={onRetry}>
            {(instances) =>
              instances.length === 0 ? (
                <Hint>No instances yet — ingest a source for this class.</Hint>
              ) : (
                <ul>
                  {instances.map((instance) => (
                    <li key={instance.id}>{instance.label}</li>
                  ))}
                </ul>
              )
            }
          </Resource>
        )}
      </div>
    </li>
  );
}

function ProposalCard({
  proposal,
  onDecide,
}: {
  proposal: Proposal;
  onDecide: (decision: "approved" | "rejected") => Promise<void>;
}) {
  const mutation = useMutation();

  return (
    <li className="ws-proposal" data-status={proposal.status}>
      <div className="ws-proposal-top">
        <span className="ws-eyebrow">Proposal</span>
        <span className="ws-status-tag">{proposal.status}</span>
      </div>
      <p>{proposal.description}</p>
      {proposal.status === "pending" && (
        <div className="ws-inline-actions">
          <button
            type="button"
            className="ws-button ws-button-primary"
            disabled={mutation.pending}
            onClick={() => void mutation.run(() => onDecide("approved"))}
          >
            Approve
          </button>
          <button
            type="button"
            className="ws-button"
            disabled={mutation.pending}
            onClick={() => void mutation.run(() => onDecide("rejected"))}
          >
            Reject
          </button>
          {mutation.pending && <span className="ws-eyebrow" role="status">Saving…</span>}
        </div>
      )}
      {mutation.error && <Failure>{mutation.error}</Failure>}
    </li>
  );
}

function Ask({ initialAnswer, onAsk }: AskProps) {
  const [question, setQuestion] = useState("");
  const [answer, setAnswer] = useState(initialAnswer ?? "");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const controller = useRef<AbortController | null>(null);
  const questionId = useId();

  useEffect(() => () => controller.current?.abort(), []);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (!question.trim() || controller.current) return;

    const request = new AbortController();
    controller.current = request;
    setPending(true);
    setError(null);
    setAnswer("");

    try {
      const result = await onAsk(question.trim(), request.signal);
      if (!request.signal.aborted) {
        if (!result.trim()) throw new Error("No answer returned. Rephrase your question and try again.");
        setAnswer(result);
      }
    } catch (error) {
      if (!request.signal.aborted) setError(errorMessage(error));
    } finally {
      if (!request.signal.aborted) {
        controller.current = null;
        setPending(false);
      }
    }
  }

  return (
    <div className="ws-ask ws-padded">
      <div className="ws-section-heading"><h2>Ask your memory</h2></div>
      <form onSubmit={submit}>
        <label className="ws-field-label" htmlFor={questionId}>Question</label>
        <textarea
          id={questionId}
          rows={4}
          value={question}
          disabled={pending}
          placeholder="What would you like to understand?"
          onChange={(event) => setQuestion(event.target.value)}
        />
        <div className="ws-ask-actions">
          <span className="ws-muted">Grounded in your connected knowledge.</span>
          <button
            type="submit"
            className="ws-button ws-button-primary"
            disabled={pending || !question.trim()}
          >
            {pending ? "Thinking…" : "Ask ↗"}
          </button>
        </div>
      </form>

      <div className="ws-answer-region" aria-busy={pending}>
        {pending && <p className="ws-state" role="status">Thinking…</p>}
        {error && <Failure>{error}</Failure>}
        {!pending && !error && !answer && (
          <Hint>Enter a question above to query your connected knowledge.</Hint>
        )}
        {answer && (
          <article className="ws-answer" aria-label="Answer" aria-live="polite">
            <h3>Answer</h3>
            <div>{answer}</div>
          </article>
        )}
      </div>
    </div>
  );
}

function Store({ atoms, onRetry }: StoreProps) {
  return (
    <div className="ws-padded">
      <div className="ws-section-heading">
        <h2>Recent atoms</h2>
        <a className="ws-link" href="#/db">Open Store →</a>
      </div>
      <Resource value={atoms} onRetry={onRetry}>
        {(items) =>
          items.length === 0 ? (
            <Hint>No atoms yet — open Store to ingest your first source.</Hint>
          ) : (
            <div className="ws-table-scroll">
              <table className="ws-table">
                <caption className="ws-sr-only">Recent atoms and indexing state</caption>
                <thead>
                  <tr><th scope="col">Title</th><th scope="col">Tier</th><th scope="col">Indexed</th></tr>
                </thead>
                <tbody>
                  {items.map((atom) => (
                    <tr key={atom.id}>
                      <td>{atom.title}</td>
                      <td><span className={`ws-tier ws-tier-${atom.tier}`}>{atom.tier}</span></td>
                      <td>
                        {atom.indexedAt ? <Time value={atom.indexedAt} /> : <span className="ws-muted">Not indexed</span>}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )
        }
      </Resource>
    </div>
  );
}
