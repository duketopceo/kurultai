export interface Atom {
  id: string;
  title: string;
  summary: string;
  source: string;
  source_id: string;
  tags: string[];
  file: string;
  indexed_at: string;
  last_accessed_at: string;
  score: number;
  tier: 'hot' | 'warm' | 'cold';
  lean: boolean;
}

export interface ApiAtomResult {
  atom?: RawAtom;
  score?: number;
  id?: string;
  title?: string;
  summary?: string;
  source?: string;
  source_id?: string;
  tags?: string[];
  file_path?: string;
  indexed_at?: string;
  last_accessed_at?: string;
  tier?: string;
  metadata?: Record<string, unknown>;
}

export interface RawAtom {
  id?: string;
  title?: string;
  summary?: string;
  content?: string;
  source?: string;
  source_id?: string;
  tags?: string[];
  file_path?: string;
  indexed_at?: string;
  last_accessed_at?: string;
  tier?: string;
  metadata?: Record<string, unknown>;
}

export interface GraphNode {
  id: string;
  title?: string;
  summary?: string;
  source?: string;
  source_id?: string;
  tags?: string[];
  indexed_at?: string;
  last_accessed_at?: string;
  tier?: string;
}

export interface Link {
  a: string;
  b: string;
  strength: number;
}

export interface StatusResponse {
  ok: boolean;
  version?: string;
  atom_count?: number;
  indexed_at?: string;
}

export interface ActivityItem {
  id: string;
  event: string;
  source?: string;
  title?: string;
  ts?: string;
}

export type LayoutMode = 'brain' | 'ontology';
export type LoadTier = 'low' | 'mid' | 'high' | 'max';
export const LOAD_TIER_CAPS: Record<LoadTier, number> = { low: 500, mid: 2000, high: 6000, max: 20000 };
export type Theme = 'dark' | 'light';
