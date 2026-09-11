import { useCallback, useEffect, useState } from 'react';
import * as api from '../api';
import { Chatboard } from './chatboard/Chatboard';
import {
  buildReactionIndex,
  mapMessages,
  mapThreads,
  presenceMap,
} from './chatboard/chatboard-mapping';
type ChatboardProps = Parameters<typeof Chatboard>[0];
export function HeyPanel() {
  const [threads, setThreads] = useState<api.HeyThread[]>([]);
  const [messages, setMessages] = useState<api.HeyMessage[]>([]);
  const [presence, setPresence] = useState<api.HeyPresence[]>([]);
  const [unread, setUnread] = useState<api.HeyMessage[]>([]);
  const [activeThreadId, setActiveThreadId] = useState('hey.md');
  const [refreshVersion, setRefreshVersion] = useState(0);
  const refresh = useCallback(() => {
    setRefreshVersion((version) => version + 1);
  }, []);
  useEffect(() => {
    const controller = new AbortController();
    let running = false;
    const poll = async () => {
      if (running || controller.signal.aborted) return;
      running = true;
      try {
        const [list, nextPresence, nextUnread] = await Promise.all([
          api.fetchHeyThreads(20, controller.signal),
          api.fetchHeyPresence(50, controller.signal).catch(() => undefined),
          api.fetchHeyUnread(50, undefined, controller.signal).catch(() => undefined),
        ]);
        if (controller.signal.aborted) return;
        const threadId = list.find(
          (thread) => thread.id === activeThreadId || thread.name === activeThreadId,
        )?.id || activeThreadId;
        setThreads(list);
        if (nextPresence !== undefined) setPresence(nextPresence);
        if (nextUnread !== undefined) setUnread(nextUnread);
        if (threadId !== activeThreadId) {
          setMessages([]);
          setActiveThreadId(threadId);
          return;
        }
        const nextMessages = await api.fetchHeyMessages(threadId, 40, controller.signal);
        if (!controller.signal.aborted) setMessages(nextMessages);
      } catch {
        // Keep the last successful snapshot during transient failures.
      } finally {
        running = false;
      }
    };
    void poll();
    const timer = window.setInterval(() => void poll(), 15000);
    return () => {
      controller.abort();
      window.clearInterval(timer);
    };
  }, [activeThreadId, refreshVersion]);
  const onOpenThread = useCallback((threadId: string) => {
    if (threadId !== activeThreadId) setMessages([]);
    setActiveThreadId(threadId);
    refresh();
  }, [activeThreadId, refresh]);
  const onSend = useCallback(async (body: string) => {
    try {
      await api.postHeyMessage(activeThreadId, body);
      refresh();
    } catch {
      // A failed mutation leaves the current board intact.
    }
  }, [activeThreadId, refresh]);
  const onReact = useCallback(async (messageId: string, emoji: string) => {
    try {
      await api.reactHeyMessage(messageId, emoji, activeThreadId);
      refresh();
    } catch {
      // A failed mutation leaves the current board intact.
    }
  }, [activeThreadId, refresh]);
  const unreadByThread = new Map<string, number>();
  for (const message of unread) {
    unreadByThread.set(
      message.thread_id,
      (unreadByThread.get(message.thread_id) ?? 0) + 1,
    );
  }
  const reactions = buildReactionIndex(messages);
  const boardProps: ChatboardProps = {
    threads: mapThreads(threads, { unreadByThread }),
    messages: mapMessages(messages, { reactions }),
    presence: presenceMap(presence),
    activeThreadId,
    onOpenThread,
    onSend,
    onReact,
  };
  return (
    <section className="panel chrome-panel hey-panel" aria-label="Agent message board">
      <header className="panel-head">
        <h2>Hey board</h2>
        <button type="button" className="ghost" onClick={refresh}>
          Refresh
        </button>
      </header>
      <p className="muted hey-caption">Active WIP / agent coordination — not long-term memory.</p>
      <Chatboard {...boardProps} />
    </section>
  );
}
