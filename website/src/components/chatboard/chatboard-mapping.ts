// Shared chatboard view-model contract (identical in mapping module + components):
export interface HeyAgentRef { codename?: string; name?: string; instance_id?: string }
export interface ThreadVM { id: string; title: string; peerLabel: string; agentKey: string; unread: number; updatedAtMs: number; updatedLabel: string }
export interface MessageVM { id: string; senderLabel: string; senderKind: 'agent' | 'human' | 'system'; body: string; createdAtMs: number; timeLabel: string; agentKey: string; reactions: Array<{ emoji: string; count: number }> }
export interface MapThreadsOptions { nowMs?: number; unreadByThread?: ReadonlyMap<string, number> }
export interface MapMessagesOptions { nowMs?: number; reactions?: ReadonlyMap<string, MessageVM['reactions']> }
export type PresenceStatus = 'online' | 'away' | 'offline';
type Fields = Record<string, unknown>;
const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const MAX_DATE_MS = 8_640_000_000_000_000;
const PRESENCE_ONLINE_MS = 15 * MINUTE;
const PRESENCE_AWAY_MS = 4 * HOUR;
function fields(value: unknown): Fields {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    ? value as Fields
    : {};
}
function text(value: unknown): string {
  return typeof value === 'string' ? value.trim() : '';
}
function identifier(value: unknown): string {
  return text(value) || (typeof value === 'number' && isFinite(value) ? String(value) : '');
}
function timestamp(value: unknown): number | undefined {
  const ms = typeof value === 'number'
    ? value
    : typeof value === 'string' && value.trim() ? Date.parse(value) : NaN;
  return isFinite(ms) && Math.abs(ms) <= MAX_DATE_MS ? Math.floor(ms) : undefined;
}
function clock(nowMs: number | undefined): number {
  return timestamp(nowMs) ?? Date.now();
}
function relativeLabel(atMs: number, nowMs: number): string {
  const elapsed = Math.max(0, nowMs - atMs);
  if (elapsed < MINUTE) return 'now';
  if (elapsed < HOUR) return `${Math.floor(elapsed / MINUTE)}m`;
  if (elapsed < DAY) return `${Math.floor(elapsed / HOUR)}h`;
  if (elapsed < 7 * DAY) return `${Math.floor(elapsed / DAY)}d`;
  return new Date(atMs).toISOString().split('T')[0] || '';
}
function count(value: unknown, fallback: number): number {
  return typeof value === 'number' && isFinite(value)
    ? Math.max(0, Math.floor(value))
    : fallback;
}
function senderKind(value: unknown): MessageVM['senderKind'] {
  return value === 'human' || value === 'system' ? value : 'agent';
}
function agentRef(value: Fields): HeyAgentRef {
  return {
    codename: text(value.agent_codename) || text(value.codename),
    name: text(value.name) || identifier(value.agent_id),
    instance_id: text(value.instance_id)
  };
}
export function identityLabel(a: HeyAgentRef | { kind: 'human' | 'system' }): string {
  const value = fields(a);
  if (value.kind === 'human') return 'Human';
  if (value.kind === 'system') return 'System';
  const codename = text(value.codename);
  const name = text(value.name);
  const instance = text(value.instance_id);
  if (codename) return instance ? `${codename}@${instance}` : codename;
  return name || instance || 'agent';
}
/** Aggregate `kind: 'reaction'` rows onto their parent message, first-seen emoji order. */
export function buildReactionIndex(raw: unknown[]): Map<string, MessageVM['reactions']> {
  const index = new Map<string, MessageVM['reactions']>();
  if (!Array.isArray(raw)) return index;
  for (const item of raw) {
    const row = fields(item);
    if (row.kind !== 'reaction') continue;
    const parentId = identifier(row.parent_id);
    const emoji = text(row.content) || text(row.emoji);
    if (!parentId || !emoji) continue;
    const list = index.get(parentId) ?? [];
    const existing = list.find((entry) => entry.emoji === emoji);
    if (existing) {
      existing.count += 1;
    } else {
      list.push({ emoji, count: 1 });
    }
    index.set(parentId, list);
  }
  return index;
}
/** Latest activity per agent_id bucketed into online/away/offline by recency. */
export function presenceMap(raw: unknown, nowMs?: number): Record<string, PresenceStatus> {
  const now = clock(nowMs);
  const out: Record<string, PresenceStatus> = {};
  if (!Array.isArray(raw)) return out;
  const rank: Record<PresenceStatus, number> = { offline: 0, away: 1, online: 2 };
  for (const item of raw) {
    const row = fields(item);
    const agentId = identifier(row.agent_id);
    if (!agentId) continue;
    const at = timestamp(row.created_at);
    const elapsed = at === undefined ? Infinity : Math.max(0, now - at);
    const status: PresenceStatus =
      elapsed < PRESENCE_ONLINE_MS ? 'online' : elapsed < PRESENCE_AWAY_MS ? 'away' : 'offline';
    if (rank[status] > (rank[out[agentId]] ?? -1)) {
      out[agentId] = status;
    }
  }
  return out;
}
export function mapThreads(raw: unknown[], opts?: MapThreadsOptions): ThreadVM[] {
  const now = clock(opts?.nowMs);
  if (!Array.isArray(raw)) return [];
  return raw.map((item: unknown, index: number): ThreadVM => {
    const thread = fields(item);
    const id = identifier(thread.id) || `thread-${index}`;
    const title = text(thread.name) || text(thread.title) || identifier(thread.id) || 'Untitled thread';
    const updatedAtMs = timestamp(thread.updated_at) ?? timestamp(thread.created_at) ?? 0;
    return {
      id,
      title,
      peerLabel: title,
      agentKey: identifier(thread.agent_id) || identifier(thread.last_agent_id),
      unread: count(opts?.unreadByThread?.get(id), 0),
      updatedAtMs,
      updatedLabel: relativeLabel(updatedAtMs, now)
    };
  }).sort((a, b) => b.updatedAtMs - a.updatedAtMs);
}
export function mapMessages(raw: unknown[], opts?: MapMessagesOptions): MessageVM[] {
  const now = clock(opts?.nowMs);
  if (!Array.isArray(raw)) return [];
  const out: MessageVM[] = [];
  raw.forEach((item: unknown, index: number) => {
    const message = fields(item);
    if (message.kind === 'reaction') return;
    const kind = senderKind(message.kind);
    const createdAtMs = timestamp(message.created_at) ?? 0;
    const id = identifier(message.id) || `message-${index}`;
    const reactions = opts?.reactions?.get(id);
    out.push({
      id,
      senderLabel: kind === 'agent'
        ? identityLabel(agentRef(message))
        : identityLabel({ kind }),
      senderKind: kind,
      body: typeof message.content === 'string' ? message.content : '',
      createdAtMs,
      timeLabel: relativeLabel(createdAtMs, now),
      agentKey: identifier(message.agent_id),
      reactions: reactions ? [...reactions] : []
    });
  });
  return out;
}
