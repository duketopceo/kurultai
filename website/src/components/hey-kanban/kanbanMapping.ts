// Lane mapping for the Hey kanban view (#331): a leading `[lane]` token in
// message content assigns the card to a column; untagged messages land in
// `inbox`. Moving a card = rewriting that token via PATCH — the store stays
// plain hey messages, so the board and the chatboard never diverge.

export const DEFAULT_LANE = 'inbox';

// Preferred column order; lanes seen in data but not listed here append after
// `done` in first-seen order.
export const KNOWN_LANES = ['inbox', 'todo', 'doing', 'review', 'done'] as const;

const LANE_RE = /^\[([a-z][a-z0-9_-]{0,23})\]\s*/i;

export type LaneMessage = {
  id: string;
  kind: string;
  content: string;
  agent_codename?: string;
  created_at: string;
};

export function laneOf(content: string): string {
  const m = LANE_RE.exec(content.trim());
  return m ? m[1].toLowerCase() : DEFAULT_LANE;
}

/** Strip the leading lane token for display; returns content unchanged when absent. */
export function laneBody(content: string): string {
  return content.trim().replace(LANE_RE, '').trim();
}

/** Rewrite (or add) the leading `[lane]` token. */
export function withLane(content: string, lane: string): string {
  const body = laneBody(content);
  return lane === DEFAULT_LANE ? body || `[${lane}]` : `[${lane}] ${body}`.trim();
}

export type KanbanColumnVM = { id: string; title: string; messages: LaneMessage[] };

/** Bucket non-reaction messages into ordered lane columns. */
export function messagesToColumns(messages: LaneMessage[]): KanbanColumnVM[] {
  const lanes = new Map<string, LaneMessage[]>();
  const order: string[] = [...KNOWN_LANES];
  for (const lane of order) lanes.set(lane, []);
  for (const m of messages) {
    if (m.kind === 'reaction') continue;
    const lane = laneOf(m.content);
    if (!lanes.has(lane)) {
      lanes.set(lane, []);
      order.push(lane);
    }
    lanes.get(lane)!.push(m);
  }
  return order
    .filter((lane) => lanes.get(lane)!.length > 0 || KNOWN_LANES.includes(lane as never))
    .map((lane) => ({
      id: lane,
      title: lane[0].toUpperCase() + lane.slice(1),
      // Oldest first inside a column — new work sinks to the bottom.
      messages: lanes
        .get(lane)!
        .slice()
        .sort((a, b) => a.created_at.localeCompare(b.created_at)),
    }));
}
