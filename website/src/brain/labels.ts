export interface LabelInput {
  ids: Array<{ id: string; title: string }>;
  links: Array<{ a: string; b: string }>;
  mode: 'brain' | 'ontology';
  showLabels: boolean;
  hoverId: string | null;
  hoverConnected: string[];
  cameraDistance: number;
}
export interface LabelPlanEntry {
  id: string;
  title: string;
  opacity: number;
}
interface RankedLabel {
  id: string;
  title: string;
  degree: number;
  order: number;
}
export function computeLabelPlan(input: LabelInput): {
  labels: LabelPlanEntry[];
  cap: number;
} {
  const cap = 200;
  const labels: LabelPlanEntry[] = [];
  if (input.mode === 'brain' && !input.showLabels) {
    return { labels, cap };
  }
  const byId: Record<string, RankedLabel | undefined> = Object.create(null);
  const ranked: RankedLabel[] = [];
  for (const item of input.ids) {
    if (byId[item.id] !== undefined) {
      continue;
    }
    const node: RankedLabel = {
      id: item.id,
      title: item.title,
      degree: 0,
      order: ranked.length
    };
    byId[item.id] = node;
    ranked.push(node);
  }
  const included: Record<string, boolean | undefined> = Object.create(null);
  const add = (id: string, opacity: number): void => {
    const node = byId[id];
    if (node === undefined || included[id] || labels.length >= cap) {
      return;
    }
    included[id] = true;
    labels.push({ id: node.id, title: node.title, opacity });
  };
  if (input.mode === 'ontology') {
    for (const node of ranked) {
      add(node.id, 1);
      if (labels.length >= cap) {
        break;
      }
    }
    return { labels, cap };
  }
  for (const link of input.links) {
    const a = byId[link.a];
    const b = byId[link.b];
    if (a !== undefined) {
      a.degree += 1;
    }
    if (b !== undefined) {
      b.degree += 1;
    }
  }
  ranked.sort((a, b) => b.degree - a.degree || a.order - b.order);
  // Reserve capacity for hover labels before filling the degree-ranked tiers.
  if (input.hoverId !== null) {
    add(input.hoverId, 1);
  }
  for (const id of input.hoverConnected) {
    add(id, 1);
  }
  const tierOpacity = Math.max(0, Math.min(0.6, 1.6 - input.cameraDistance * 0.35));
  let rank = 0;
  for (const node of ranked) {
    if (labels.length >= cap) {
      break;
    }
    const opacity = rank < 24 ? 0.9 : tierOpacity;
    if (opacity > 0.05) {
      add(node.id, opacity);
    }
    rank += 1;
  }
  return { labels, cap };
}
