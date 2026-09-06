export interface FdgNode {
  id: string;
  x: number;
  y: number;
  z: number;
  vx: number;
  vy: number;
  vz: number;
  tags: string[];
}

export interface FdgLink {
  a: number;
  b: number;
  strength: number;
}

export interface FdgParams {
  theta: number;
  repulsion: number;
  springK: number;
  springRest: number;
  centerK: number;
  tagK: number;
  hullK: number;
  damping: number;
  minTagMembers: number;
}

export const DEFAULT_FDG_PARAMS: FdgParams = {
  theta: 0.8,
  repulsion: 0.02,
  springK: 0.05,
  springRest: 0.35,
  centerK: 0.01,
  tagK: 0.02,
  // Soft hull bias during the tick; hard SDF project after integrate is authoritative.
  hullK: 0.12,
  damping: 0.85,
  minTagMembers: 3,
};

export interface SignedDistanceField {
  nx: number;
  ny: number;
  nz: number;
  originX: number;
  originY: number;
  originZ: number;
  cell: number;
  values: Float32Array;
}

export type FdgWorkerIn =
  | { type: 'init'; nodes: FdgNode[]; links: FdgLink[]; sdf: ArrayBuffer; aabb: number[] }
  | { type: 'tick'; steps: number }
  | { type: 'setLinks'; links: FdgLink[] };

export type FdgWorkerOut = { type: 'positions'; xyz: Float32Array; ids: string[] };
