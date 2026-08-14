import { tickFdg } from './fdg.ts';
import { unpackSdf } from './sdf.ts';
import { DEFAULT_FDG_PARAMS } from './types.ts';
import type { FdgLink, FdgNode, FdgWorkerIn, FdgWorkerOut, SignedDistanceField } from './types.ts';

let nodes: FdgNode[] = [];
let links: FdgLink[] = [];
let sdf: SignedDistanceField | null = null;

self.onmessage = (event: MessageEvent<FdgWorkerIn>) => {
  const msg = event.data;
  switch (msg.type) {
    case 'init':
      nodes = msg.nodes;
      links = msg.links;
      sdf = msg.sdf.byteLength > 0 ? unpackSdf(msg.sdf) : null;
      return;
    case 'setLinks':
      links = msg.links;
      return;
    case 'tick': {
      const steps = Math.max(0, msg.steps | 0);
      for (let s = 0; s < steps; s++) tickFdg(nodes, links, sdf, DEFAULT_FDG_PARAMS);
      const xyz = new Float32Array(nodes.length * 3);
      const ids = new Array<string>(nodes.length);
      for (let i = 0; i < nodes.length; i++) {
        const node = nodes[i];
        xyz[i * 3] = node.x;
        xyz[i * 3 + 1] = node.y;
        xyz[i * 3 + 2] = node.z;
        ids[i] = node.id;
      }
      const out: FdgWorkerOut = { type: 'positions', xyz, ids };
      self.postMessage(out);
      return;
    }
    default: {
      const _never: never = msg;
      void _never;
    }
  }
};
