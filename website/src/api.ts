import type {
  ApiAtomResult,
  Atom,
  StatusResponse,
  ActivityItem,
  GraphNode,
  OntologyResponse,
} from './types';

function text(value: unknown, fallback = '—'): string {
  if (value == null || value === '') return fallback;
  return String(value);
}

export function normalizeAtom(result: ApiAtomResult): Atom {
  const atom = result.atom ?? (result as RawFallback);
  const meta = (atom as { metadata?: Record<string, unknown> }).metadata ?? {};
  return {
    id: text(atom.id, crypto.randomUUID()),
    title: text(atom.title, 'Untitled memory'),
    summary: text(
      (atom as { summary?: string; content?: string }).summary ||
      (atom as { content?: string }).content ||
      (meta as { summary?: string }).summary,
      'No local summary available.'
    ),
    source: text(atom.source),
    source_id: text(atom.source_id),
    tags: Array.isArray(atom.tags) ? atom.tags : Array.isArray((meta as { tags?: string[] }).tags) ? (meta as { tags?: string[] }).tags! : [],
    file: (atom as { file_path?: string }).file_path || (meta as { file_path?: string }).file_path || '',
    indexed_at: (atom as { indexed_at?: string }).indexed_at || (meta as { indexed_at?: string }).indexed_at || '',
    last_accessed_at: (atom as { last_accessed_at?: string }).last_accessed_at || (meta as { last_accessed_at?: string }).last_accessed_at || '',
    score: Number(result.score) || 0,
    tier: ((atom as { tier?: string }).tier || (meta as { tier?: string }).tier || 'warm') as Atom['tier'],
    lean: false,
  };
}

type RawFallback = ApiAtomResult & { metadata?: Record<string, unknown> };

export function normalizeGraphNode(node: GraphNode): Atom {
  const n = node || {};
  return {
    id: text(n.id, crypto.randomUUID()),
    title: text(n.title, 'Untitled memory'),
    summary: text(n.summary, n.tier === 'hot' ? 'No local summary available.' : 'Lean stub — select to load details.'),
    source: text(n.source),
    source_id: text(n.source_id),
    tags: Array.isArray(n.tags) ? n.tags : [],
    file: '',
    indexed_at: n.indexed_at || '',
    last_accessed_at: n.last_accessed_at || '',
    score: n.tier === 'hot' ? 1 : n.tier === 'warm' ? 0.55 : 0.25,
    tier: (n.tier || 'warm') as Atom['tier'],
    lean: true,
  };
}

const dbg = (...args: unknown[]) => console.debug('[kurultai:api]', ...args);

const TOKEN_KEY = 'kurultai:token';

function getAuthHeaders(init?: HeadersInit): HeadersInit {
  const token = typeof window !== 'undefined' ? localStorage.getItem(TOKEN_KEY) : null;
  const headers: Record<string, string> = { Accept: 'application/json' };
  if (token && token.length > 0) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  if (init) {
    const h = new Headers(init);
    h.forEach((v, k) => { headers[k] = v; });
  }
  return headers;
}

async function getJson<T>(path: string, signal?: AbortSignal): Promise<T> {
  dbg('GET', path);
  const t0 = performance.now();
  const r = await fetch(path, {
    cache: 'no-store',
    headers: getAuthHeaders(),
    signal,
  });
  if (!r.ok) {
    dbg('GET failed', path, r.status);
    throw new Error(`${path} failed (${r.status})`);
  }
  const data = await r.json() as T;
  dbg('GET ok', path, `${(performance.now() - t0).toFixed(0)}ms`);
  return data;
}

export async function fetchStatus(signal?: AbortSignal): Promise<StatusResponse> {
  return getJson<StatusResponse>('/api/status', signal);
}

export async function fetchAtoms(limit: number, signal?: AbortSignal): Promise<Atom[]> {
  const data = await getJson<ApiAtomResult[]>(`/api/atoms?limit=${limit}`, signal);
  if (!Array.isArray(data)) return [];
  return data.map(normalizeAtom);
}

export async function fetchGraph(signal?: AbortSignal): Promise<Atom[]> {
  const data = await getJson<{ nodes?: GraphNode[]; hot?: GraphNode[]; warm?: GraphNode[]; cold?: GraphNode[] }>('/api/graph', signal);
  const nodes: GraphNode[] = data.nodes ?? [
    ...(data.hot ?? []),
    ...(data.warm ?? []),
    ...(data.cold ?? []),
  ];
  return nodes.map(normalizeGraphNode);
}

export async function fetchActivity(signal?: AbortSignal): Promise<ActivityItem[]> {
  const data = await getJson<ActivityItem[]>('/api/activity', signal);
  return Array.isArray(data) ? data : [];
}

export async function fetchOntology(signal?: AbortSignal): Promise<OntologyResponse> {
  const data = await getJson<Partial<OntologyResponse>>('/api/ontology', signal);
  return {
    ok: data.ok !== false,
    entities: Array.isArray(data.entities) ? data.entities : [],
    links: Array.isArray(data.links) ? data.links : [],
  };
}

export async function searchAtoms(query: string, signal?: AbortSignal): Promise<Atom[]> {
  const data = await getJson<ApiAtomResult[]>(`/api/search?q=${encodeURIComponent(query)}&limit=20`, signal);
  if (!Array.isArray(data)) return [];
  return data.map(normalizeAtom);
}

export async function askBrain(question: string, signal?: AbortSignal): Promise<string> {
  const r = await fetch('/api/ask', {
    method: 'POST',
    headers: getAuthHeaders({ 'Content-Type': 'application/json' }),
    body: JSON.stringify({ question }),
    signal,
  });
  if (!r.ok) throw new Error(`ask failed (${r.status})`);
  const data = await r.json();
  return data.answer ?? data.text ?? JSON.stringify(data);
}

export async function touchAtom(atomId: string): Promise<Atom | null> {
  const r = await fetch('/api/touch', {
    method: 'POST',
    headers: getAuthHeaders({ 'Content-Type': 'application/json' }),
    body: JSON.stringify({ atom_id: atomId }),
  });
  if (!r.ok) return null;
  const data = await r.json();
  return normalizeAtom(data);
}

export async function openFile(file: string): Promise<void> {
  await fetch(`/api/open?file=${encodeURIComponent(file)}`, {
    headers: getAuthHeaders(),
  });
}

export type HeyThread = {
  id: string;
  name: string;
  parent_thread_id?: string | null;
  turn_cap?: number;
  turns_used?: number;
  created_at?: string;
  updated_at?: string;
};

export type HeyMessage = {
  id: string;
  thread_id: string;
  agent_id: string;
  parent_id?: string | null;
  kind: string;
  content: string;
  request_reply?: boolean;
  turns_consumed?: number;
  created_at: string;
};

export async function fetchHeyThreads(limit = 20, signal?: AbortSignal): Promise<HeyThread[]> {
  const data = await getJson<HeyThread[]>(`/api/hey/threads?limit=${limit}`, signal);
  return Array.isArray(data) ? data : [];
}

export async function fetchHeyMessages(
  thread: string,
  limit = 50,
  signal?: AbortSignal,
): Promise<HeyMessage[]> {
  const data = await getJson<HeyMessage[]>(
    `/api/hey/threads/${encodeURIComponent(thread)}/messages?limit=${limit}`,
    signal,
  );
  return Array.isArray(data) ? data : [];
}

