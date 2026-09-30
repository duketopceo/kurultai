// ui-next app shell — stitches design-lab winners onto the real api.ts facade.
// Plan docs/plans/2026-09-25-001 U4. No new backend surface: every adapter maps
// onto existing /api calls.

import { useCallback, useEffect, useRef, useState } from 'react';
import * as api from '../api';
import { useAuthMode } from '../auth';
import { describeProposal } from '../proposals';
import { BrainStage, type BrainStageHandle } from '../components/BrainStage';
import { laneBody, laneOf, withLane } from '../components/hey-kanban/kanbanMapping';
import { initPerf, reportPerf, setPerfTier } from '../perf';
import {
  LOAD_TIER_CAPS,
  type Atom,
  type LoadTier,
  type OntologyResponse,
} from '../types';
import { UI_VERSION } from '../version';

import { TopBar, CommandStrip, type NavKey, type SearchResult, type SearchState } from './gen/chrome/chrome';
import { BrainHero } from './gen/brain-hero/brain-hero';
import { Inspector, type Atom as InspectorAtom, type OntologyClass as InspectorClass } from './gen/inspector/inspector';
import WorkSurface, {
  type Loadable,
  type MemoryEvent,
  type PulseSnapshot,
  type SixClasses,
  type Task,
  type TaskStatus,
} from './gen/worksurface/worksurface';
import { SettingsPanel } from './gen/settings-access/settings-access';
import type { AccessMode } from './gen/settings-access/settings-access';

// Per-surface stylesheets (self-scoped classes — no global collisions).
import './gen/brain-hero/brain-hero.css';
import './gen/chrome/chrome.css';
import './gen/worksurface/worksurface.css';
import './gen/inspector/inspector.css';
import './gen/settings-access/settings-access.css';

type L<T> = Loadable<T>;
const loading: L<never> = { status: 'loading' };
const ready = <T,>(data: T): L<T> => ({ status: 'ready', data });
const failed = (e: unknown): L<never> => ({
  status: 'error',
  error: e instanceof Error ? e.message : String(e),
});

const LANE_TO_STATUS: Record<string, TaskStatus> = {
  inbox: 'queued',
  todo: 'queued',
  review: 'doing',
  doing: 'doing',
  done: 'done',
};
const STATUS_TO_LANE: Record<TaskStatus, string> = {
  queued: 'todo',
  doing: 'doing',
  done: 'done',
};

const CLASS_IDS = ['entity', 'source', 'event', 'concept', 'project', 'person'] as const;

export function NextApp() {
  const authMode = useAuthMode();
  // ── daemon + graph state ──────────────────────────────────────────
  const [daemon, setDaemon] = useState<{ status: 'online' | 'connecting' | 'offline'; apiVersion?: string | null }>({ status: 'connecting' });
  const [nav, setNav] = useState<NavKey>('brain');
  const [layout, setLayout] = useState<'brain' | 'ontology'>('brain');
  const [tier, setTier] = useState<LoadTier>('low');
  const [atoms, setAtoms] = useState<Atom[]>([]);
  const [atomTotal, setAtomTotal] = useState(0);
  const [graphLoaded, setGraphLoaded] = useState(false);
  const [ontology, setOntology] = useState<OntologyResponse>({ ok: true, entities: [], links: [] });
  const [selected, setSelected] = useState<Atom | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const brainRef = useRef<BrainStageHandle | null>(null);
  const graphAbort = useRef<AbortController | null>(null);

  // ── search state ──────────────────────────────────────────────────
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[] | null>(null);
  const [searchState, setSearchState] = useState<SearchState>('idle');
  const searchAbort = useRef<AbortController | null>(null);

  // ── worksurface slices ────────────────────────────────────────────
  const [threads, setThreads] = useState<L<readonly import('./gen/worksurface/worksurface').Thread[]>>(loading);
  const [conversations, setConversations] = useState<Record<string, L<readonly import('./gen/worksurface/worksurface').Message[]> | undefined>>({});
  const [activeThread, setActiveThread] = useState<string | null>(null);
  const [presence, setPresence] = useState<import('./gen/worksurface/worksurface').HeyProps['presence']>(null);
  const [pulse, setPulse] = useState<L<PulseSnapshot>>(loading);
  const [classes, setClasses] = useState<L<SixClasses>>(loading);
  const [proposals, setProposals] = useState<L<readonly { id: string; description: string; status: 'pending' | 'approved' | 'rejected' }[]>>(loading);
  const [storeAtoms, setStoreAtoms] = useState<L<readonly { id: string; title: string; tier: 'hot' | 'warm' | 'cold'; indexedAt: string | null }[]>>(loading);

  useEffect(() => { initPerf(); }, []);

  // ── data loaders ──────────────────────────────────────────────────
  const loadStatus = useCallback(async () => {
    try {
      const s = await api.fetchStatus();
      setDaemon({ status: 'online', apiVersion: s.version ?? null });
    } catch {
      setDaemon({ status: 'offline' });
    }
  }, []);

  const loadGraph = useCallback(async (t: LoadTier) => {
    graphAbort.current?.abort();
    const ac = new AbortController();
    graphAbort.current = ac;
    const cap = LOAD_TIER_CAPS[t];
    try {
      const t0 = performance.now();
      const [graphAtoms, onto] = await Promise.all([
        api.fetchGraph({ limit: cap, excludeSource: 'repos' }, ac.signal),
        api.fetchOntology(ac.signal).catch(() => ({ ok: true, entities: [], links: [] })),
      ]);
      if (ac.signal.aborted) return;
      reportPerf('tier_load_ms', performance.now() - t0, t);
      setAtoms(graphAtoms.slice(0, cap));
      setAtomTotal(graphAtoms.length);
      setOntology(onto);
      setGraphLoaded(true);
    } catch { /* abort or offline — status poll reports it */ }
  }, []);

  const loadThreads = useCallback(async () => {
    try {
      const list = await api.fetchHeyThreads(20);
      setThreads(ready(list.map((t) => ({
        id: t.id,
        name: t.name,
        unread: false,
        lastActiveAt: t.updated_at ?? t.created_at ?? new Date(0).toISOString(),
      }))));
      setActiveThread((cur) => cur ?? list.find((t) => t.name === 'hey.md')?.id ?? list[0]?.id ?? null);
    } catch (e) { setThreads(failed(e)); }
  }, []);

  const loadMessages = useCallback(async (threadId: string) => {
    setConversations((c) => ({ ...c, [threadId]: loading }));
    try {
      const msgs = await api.fetchHeyMessages(threadId, 60);
      setConversations((c) => ({
        ...c,
        [threadId]: ready(msgs.filter((m) => m.kind !== 'reaction').map((m) => ({
          id: m.id,
          codename: m.agent_codename ?? m.agent_id,
          instanceId: m.instance_id ?? '',
          createdAt: m.created_at,
          body: m.content,
          replyToId: m.parent_id ?? undefined,
          canEdit: true,
          canDelete: true,
        }))),
      }));
    } catch (e) {
      setConversations((c) => ({ ...c, [threadId]: failed(e) }));
    }
  }, []);

  const loadPresence = useCallback(async () => {
    try {
      const seats = await api.fetchHeyPresence(50);
      const latest = seats[0];
      setPresence(latest ? {
        codename: latest.agent_codename,
        instanceId: latest.instance_id ?? '',
        status: 'online',
        workspace: latest.repo,
      } : null);
    } catch { setPresence(null); }
  }, []);

  const loadPulse = useCallback(async () => {
    try {
      const [s, activity] = await Promise.all([
        api.fetchStatus(),
        api.fetchActivity().catch(() => [] as import('../types').ActivityItem[]),
      ]);
      const events: MemoryEvent[] = activity.map((a) => ({
        id: a.id, event: a.event, title: a.title ?? '', source: a.source ?? '',
      }));
      setPulse(ready({
        stats: {
          loaded: atoms.length, total: atomTotal || s.atom_count || 0,
          hot: atoms.filter((a) => a.tier === 'hot').length,
          warm: atoms.filter((a) => a.tier === 'warm').length,
          cold: atoms.filter((a) => a.tier === 'cold').length,
          trusted: atoms.length,
        },
        events,
      }));
    } catch (e) { setPulse(failed(e)); }
  }, [atoms, atomTotal]);

  const loadOntologyClasses = useCallback(async () => {
    try {
      const onto = await api.fetchOntology();
      const classEntities = onto.entities.filter((e) => e.kind === 'class');
      const six = CLASS_IDS.map((cid, i) => {
        const match = classEntities[i];
        const instances = onto.entities.filter((e) => e.kind === 'instance' && match && e.attributes && JSON.stringify(e.attributes).includes(match.id));
        return {
          id: match?.id ?? cid,
          name: match?.name ?? cid,
          instanceCount: instances.length,
          instances: ready(instances.map((e) => ({ id: e.id, label: e.name })) as readonly { id: string; label: string }[]),
        };
      }) as unknown as SixClasses;
      setClasses(ready(six));
    } catch (e) { setClasses(failed(e)); }
  }, []);

  const loadProposals = useCallback(async () => {
    try {
      const list = await api.fetchOntologyProposals(undefined, 50);
      setProposals(ready(list.map((p) => ({
        id: p.id,
        description: describeProposal(p),
        status: (p.status === 'approved' || p.status === 'rejected' ? p.status : 'pending'),
      }))));
    } catch (e) { setProposals(failed(e)); }
  }, []);

  const loadStore = useCallback(async () => {
    try {
      const list = await api.fetchAtoms(50);
      setStoreAtoms(ready(list.map((a) => ({
        id: a.id, title: a.title, tier: a.tier, indexedAt: a.indexed_at,
      }))));
    } catch (e) { setStoreAtoms(failed(e)); }
  }, []);

  // ── polling ───────────────────────────────────────────────────────
  useEffect(() => {
    void loadStatus();
    const id = setInterval(loadStatus, 8000);
    return () => clearInterval(id);
  }, [loadStatus]);

  useEffect(() => { setPerfTier(tier); void loadGraph(tier); }, [tier, loadGraph]);

  useEffect(() => {
    void loadThreads(); void loadPresence(); void loadOntologyClasses(); void loadProposals(); void loadStore();
  }, [loadThreads, loadPresence, loadOntologyClasses, loadProposals, loadStore]);

  useEffect(() => { void loadPulse(); }, [loadPulse]);

  useEffect(() => {
    if (activeThread) void loadMessages(activeThread);
  }, [activeThread, loadMessages]);

  // ── search ────────────────────────────────────────────────────────
  useEffect(() => {
    if (!query.trim()) { setResults(null); setSearchState('idle'); return; }
    setSearchState('loading');
    searchAbort.current?.abort();
    const ac = new AbortController();
    searchAbort.current = ac;
    const id = setTimeout(async () => {
      try {
        const hits = await api.searchAtoms(query, ac.signal);
        if (!ac.signal.aborted) {
          setResults(hits.map((a) => ({ id: a.id, title: a.title, summary: a.summary })));
          setSearchState('idle');
        }
      } catch { if (!ac.signal.aborted) setSearchState('error'); }
    }, 250);
    return () => clearTimeout(id);
  }, [query]);

  // ── hey mutations ─────────────────────────────────────────────────
  const mutateThread = useCallback((threadId: string) => {
    void loadMessages(threadId);
    void loadThreads();
  }, [loadMessages, loadThreads]);

  const heyProps = {
    presence,
    threads,
    conversations,
    board: activeThread && conversations[activeThread]?.status === 'ready'
      ? ready((conversations[activeThread]!.data ?? [])
          .filter((m) => true)
          .map((m) => {
            const lane = laneOf(m.body);
            return {
              id: m.id,
              title: laneBody(m.body).slice(0, 120),
              status: LANE_TO_STATUS[lane] ?? 'queued',
              assignee: m.codename,
            } satisfies Task;
          }))
      : loading as L<readonly Task[]>,
    onSelectThread: (id: string) => setActiveThread(id),
    onCreateThread: async () => {
      const name = `thread-${Date.now().toString(36)}`;
      await api.postHeyMessage(name, '[inbox] thread opened', { thread_name: name });
      await loadThreads();
    },
    onCreateTask: async () => {
      if (!activeThread) return;
      await api.postHeyMessage(activeThread, '[todo] new task');
      mutateThread(activeThread);
    },
    onRetry: (section: string, threadId?: string) => {
      if (section === 'threads') void loadThreads();
      if (section === 'board' || section === 'conversation') void loadMessages(threadId ?? activeThread ?? '');
    },
    onSend: async (threadId: string, body: string, replyToId?: string) => {
      await api.postHeyMessage(threadId, body, replyToId ? { parent_id: replyToId } : {});
      mutateThread(threadId);
    },
    onEditMessage: async (threadId: string, messageId: string, body: string) => {
      await api.updateHeyMessage(messageId, body);
      mutateThread(threadId);
    },
    onDeleteMessage: async (threadId: string, messageId: string) => {
      await api.deleteHeyMessage(messageId);
      mutateThread(threadId);
    },
    onReact: async (threadId: string, messageId: string, emoji: string) => {
      await api.reactHeyMessage(messageId, emoji, threadId);
      mutateThread(threadId);
    },
    onMoveTask: async (taskId: string, status: TaskStatus) => {
      const conv = activeThread ? conversations[activeThread] : undefined;
      const msg = conv?.status === 'ready' ? conv.data.find((m) => m.id === taskId) : undefined;
      if (!msg) return;
      await api.updateHeyMessage(taskId, withLane(msg.body, STATUS_TO_LANE[status]));
      if (activeThread) mutateThread(activeThread);
    },
  };

  const ontologyProps = {
    classes,
    proposals,
    onExpandClass: () => undefined,
    onRetryClass: () => void loadOntologyClasses(),
    onRetry: () => { void loadOntologyClasses(); void loadProposals(); },
    onDecide: async (proposalId: string, decision: 'approved' | 'rejected') => {
      await api.decideOntologyProposal(proposalId, decision === 'approved' ? 'approve' : 'reject');
      await loadProposals();
    },
  };

  const storeProps = {
    atoms: storeAtoms,
    onRetry: () => void loadStore(),
  };

  // ── inspector adapter ─────────────────────────────────────────────
  const inspectorAtom: InspectorAtom | null = selected ? {
    id: selected.id,
    title: selected.title,
    summary: selected.summary,
    tier: selected.tier,
    score: selected.score,
    source: selected.source,
    sourceUrl: selected.file || null,
    tags: selected.tags,
    indexedAt: selected.indexed_at,
    links: null,
  } : null;
  const inspectorClasses: InspectorClass[] = ontology.entities
    .filter((e) => e.kind === 'class')
    .map((e) => ({ id: e.id, label: e.name }));

  return (
    <div className="next-app">
      <a className="skip-link" href="#ws">Skip to workspace</a>
      <TopBar
        version={`${UI_VERSION}·${typeof __UI_BUILD__ !== 'undefined' ? __UI_BUILD__ : 'dev'}`}
        active={nav}
        onNavigate={setNav}
        daemon={daemon}
        onOpenAccess={() => setSettingsOpen(true)}
      />
      <CommandStrip
        daemonStatus={daemon.status}
        query={query}
        onQueryChange={setQuery}
        results={results}
        searchState={searchState}
        onSelectResult={(r) => {
          const atom = atoms.find((a) => a.id === r.id);
          if (atom) { setSelected(atom); brainRef.current?.focusAtom(atom); }
        }}
        onResetSearch={() => { setQuery(''); setResults(null); setSearchState('idle'); }}
        tier={tier}
        onTierChange={setTier}
        layout={layout}
        onLayoutChange={setLayout}
      />
      <main id="ws" className="next-main">
        <section className="next-hero">
          <BrainHero
            atomCount={atoms.length}
            atomTotal={Math.max(atomTotal, atoms.length)}
            tier={tier}
            layout={layout === 'ontology' ? 'ontology' : 'force'}
            fps={null}
            synapses={null}
            empty={graphLoaded && atomTotal === 0 && atoms.length === 0}
          >
            <BrainStage
              ref={brainRef}
              atoms={atoms}
              renderCap={LOAD_TIER_CAPS[tier]}
              layout={layout}
              ontology={ontology}
              atomTotal={atomTotal}
              onSelect={setSelected}
              onHover={() => undefined}
              onOntologyChanged={setOntology}
              caption=""
            />
          </BrainHero>
        </section>
        <section className="next-worksurface">
          <WorkSurface
            hey={heyProps}
            pulse={{
              data: pulse,
              onRetry: () => void loadPulse(),
              subscribe: (onSnapshot) => {
                const id = setInterval(() => void loadPulse().then(() => {
                  // push latest ready snapshot into subscriber
                  setPulse((cur) => { if (cur.status === 'ready') onSnapshot(cur.data); return cur; });
                }), 10_000);
                return () => clearInterval(id);
              },
            }}
            ontology={ontologyProps}
            ask={{
              onAsk: (q, signal) => api.askBrain(q, signal),
            }}
            store={storeProps}
          />
        </section>
      </main>
      <div className="next-inspector">
        <Inspector
          atom={inspectorAtom}
          mode={selected ? 'pinned' : 'preview'}
          classes={inspectorClasses}
          onPromote={async (atomId, classId) => {
            await api.promoteAtomToOntology(atomId, classId);
            await loadOntologyClasses();
          }}
          onOpenSource={(a) => { if (a.sourceUrl) void api.openFile(a.sourceUrl); }}
          onClose={() => setSelected(null)}
        />
      </div>
      {settingsOpen && (
        <div
          role="dialog"
          aria-modal="true"
          onKeyDown={(e) => { if (e.key === 'Escape') setSettingsOpen(false); }}
          ref={(el) => el?.focus()}
          tabIndex={-1}
          style={{ position: 'fixed', inset: 0, zIndex: 300, background: 'var(--scrim)' }}
          onClick={(e) => { if (e.target === e.currentTarget) setSettingsOpen(false); }}
        >
          <div style={{ maxWidth: 720, margin: '8vh auto', maxHeight: '84vh', overflow: 'auto' }}>
            <SettingsPanel
              daemon={{ online: daemon.status === 'online', version: daemon.apiVersion ?? null }}
              presence={null}
              mode={(authMode === 'locked' ? 'token' : 'open') as AccessMode}
              onLogin={async () => setSettingsOpen(false)}
            />
          </div>
        </div>
      )}
    </div>
  );
}
