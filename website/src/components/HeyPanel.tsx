import { useCallback, useEffect, useState } from 'react';
import * as api from '../api';
import { Chatboard } from './chatboard/Chatboard';
import { mapThreads, mapMessages } from './chatboard/chatboard-mapping';
type ChatboardProps = Parameters<typeof Chatboard>[0];
// Optional endpoints are resolved at the API boundary for older deployments.
const endpoints: Readonly<Record<string, unknown>> = api;
async function callEndpoint(name: string, ...args: unknown[]): Promise<unknown> {
  const endpoint = endpoints[name];
  if (typeof endpoint !== 'function') {
    throw new Error('Hey endpoint unavailable');
  }
  return endpoint(...args);
}
export function HeyPanel() {
  const [threads, setThreads] = useState<api.HeyThread[]>([]);
  const [messages, setMessages] = useState<api.HeyMessage[]>([]);
  const [presence, setPresence] = useState<unknown>([]);
  const [unread, setUnread] = useState<unknown>({});
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
          callEndpoint('fetchHeyPresence').catch(() => undefined),
          callEndpoint('fetchHeyUnread').catch(() => undefined),
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
  const onSend = useCallback(async (...args: unknown[]) => {
    try {
      await callEndpoint('postHeyMessage', activeThreadId, ...args);
      refresh();
    } catch {
      // A failed mutation leaves the current board intact.
    }
  }, [activeThreadId, refresh]);
  const onReact = useCallback(async (...args: unknown[]) => {
    try {
      await callEndpoint('reactHeyMessage', ...args);
      refresh();
    } catch {
      // A failed mutation leaves the current board intact.
    }
  }, [refresh]);
  const boardProps = {
    threads: mapThreads(threads),
    messages: mapMessages(messages),
    presence,
    unread,
    activeThreadId,
    onOpenThread,
    onSend,
    onReact,
  } as ChatboardProps;
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
