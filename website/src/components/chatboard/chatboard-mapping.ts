// Shared chatboard view-model contract (identical in mapping module + components):
export interface HeyAgentRef { codename?: string; name?: string; instance_id?: string }
export interface ThreadVM { id: string; title: string; peerLabel: string; unread: number; updatedAtMs: number; updatedLabel: string }
export interface MessageVM { id: string; senderLabel: string; senderKind: 'agent' | 'human' | 'system'; body: string; createdAtMs: number; timeLabel: string; reactions: Array<{ emoji: string; count: number }> }
type Fields = Record<string, unknown>;
const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const MAX_DATE_MS = 8_640_000_000_000_000;
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
function peerLabel(value: unknown, title: string): string {
  if (!Array.isArray(value)) return title;
  for (const participant of value) {
    if (participant === null || typeof participant !== 'object' || Array.isArray(participant)) {
      continue;
    }
    const peer = fields(participant);
    const kind = senderKind(peer.kind);
    if (kind === 'human') continue;
    return kind === 'system' ? identityLabel({ kind }) : identityLabel(agentRef(peer));
  }
  return title;
}
function mapReactions(value: unknown): MessageVM['reactions'] {
  const result: MessageVM['reactions'] = [];
  if (!Array.isArray(value)) return result;
  for (const item of value) {
    const reaction = fields(item);
    const emoji = typeof item === 'string' ? text(item) : text(reaction.emoji);
    if (!emoji) continue;
    const amount = count(reaction.count, 1);
    let existing: MessageVM['reactions'][number] | undefined;
    for (const entry of result) {
      if (entry.emoji === emoji) {
        existing = entry;
        break;
      }
    }
    if (existing) {
      existing.count = Math.min(Number.MAX_VALUE, existing.count + amount);
    } else {
      result.push({ emoji, count: amount });
    }
  }
  return result;
}
export function mapThreads(raw: unknown[], nowMs?: number): ThreadVM[] {
  const now = clock(nowMs);
  if (!Array.isArray(raw)) return [];
  return raw.map((item: unknown, index: number): ThreadVM => {
    const thread = fields(item);
    const id = identifier(thread.id) || `thread-${index}`;
    const title = text(thread.name) || text(thread.title) || identifier(thread.id) || 'Untitled thread';
    const updatedAtMs = timestamp(thread.updated_at) ?? timestamp(thread.created_at) ?? 0;
    return {
      id,
      title,
      peerLabel: peerLabel(thread.participants, title),
      unread: count(thread.unread, count(thread.unread_count, 0)),
      updatedAtMs,
      updatedLabel: relativeLabel(updatedAtMs, now)
    };
  }).sort((a, b) => b.updatedAtMs - a.updatedAtMs);
}
export function mapMessages(raw: unknown[], nowMs?: number): MessageVM[] {
  const now = clock(nowMs);
  if (!Array.isArray(raw)) return [];
  return raw.map((item: unknown, index: number): MessageVM => {
    const message = fields(item);
    const kind = senderKind(message.kind);
    const createdAtMs = timestamp(message.created_at) ?? 0;
    return {
      id: identifier(message.id) || `message-${index}`,
      senderLabel: kind === 'agent'
        ? identityLabel(agentRef(message))
        : identityLabel({ kind }),
      senderKind: kind,
      body: typeof message.content === 'string' ? message.content : '',
      createdAtMs,
      timeLabel: relativeLabel(createdAtMs, now),
      reactions: mapReactions(message.reactions)
    };
  });
}
