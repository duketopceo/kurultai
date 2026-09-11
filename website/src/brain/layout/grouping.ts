export interface GroupInputEntity {
  id: string;
  kind: 'class' | 'instance';
  name?: string;
}
export interface GroupInputLink {
  a: string;
  b: string;
  rel?: string;
}
export interface ClassGroup {
  classId: string;
  name: string;
  childIds: string[];
  nestedIds: string[];
  tintIndex: number;
}
export const GROUP_TINTS: number[] = [
  0x5b21b6,
  0x6d28d9,
  0x7c3aed,
  0x8b5cf6,
  0xa78bfa,
  0xc4b5fd
];
function compareText(a: string, b: string): number {
  return a < b ? -1 : a > b ? 1 : 0;
}
export function deriveGroups(
  entities: GroupInputEntity[],
  links: GroupInputLink[]
): ClassGroup[] {
  const entitiesById = new Map<string, GroupInputEntity>();
  for (const entity of entities) {
    entitiesById.set(entity.id, entity);
  }
  const classes = Array.from(entitiesById.values())
    .filter(entity => entity.kind === 'class')
    .sort((a, b) =>
      compareText(a.name ?? a.id, b.name ?? b.id) || compareText(a.id, b.id)
    );
  const instanceChildren = new Map<string, Set<string>>();
  const subclasses = new Map<string, Set<string>>();
  const hasParent = new Set<string>();
  for (const entity of classes) {
    instanceChildren.set(entity.id, new Set<string>());
    subclasses.set(entity.id, new Set<string>());
  }
  for (const link of links) {
    const a = entitiesById.get(link.a);
    const b = entitiesById.get(link.b);
    if (!a || !b) {
      continue;
    }
    if (link.rel === 'instance_of') {
      if (a.kind === 'instance' && b.kind === 'class') {
        instanceChildren.get(b.id)?.add(a.id);
      } else if (b.kind === 'instance' && a.kind === 'class') {
        instanceChildren.get(a.id)?.add(b.id);
      }
    } else if (
      link.rel === 'is_a' &&
      a.kind === 'class' &&
      b.kind === 'class'
    ) {
      subclasses.get(b.id)?.add(a.id);
      hasParent.add(a.id);
    }
  }
  return classes.map((entity, index) => {
    const nestedIds = new Set<string>();
    if (!hasParent.has(entity.id)) {
      const pending = Array.from(subclasses.get(entity.id) ?? []);
      const visited = new Set<string>([entity.id]);
      // Traverse only from roots; visited bounds traversal even with cycles.
      while (pending.length > 0) {
        const id = pending.pop();
        if (id === undefined || visited.has(id)) {
          continue;
        }
        visited.add(id);
        nestedIds.add(id);
        for (const childId of subclasses.get(id) ?? []) {
          pending.push(childId);
        }
      }
    }
    return {
      classId: entity.id,
      name: entity.name ?? entity.id,
      childIds: Array.from(instanceChildren.get(entity.id) ?? []).sort(compareText),
      nestedIds: Array.from(nestedIds).sort(compareText),
      tintIndex: index % GROUP_TINTS.length
    };
  });
}
