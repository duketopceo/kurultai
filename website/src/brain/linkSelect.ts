/**
 * Intentional-synapse selection: shared-tag derivation yields cliques —
 * N atoms sharing one tag produce C(N,2) near-identical links whose
 * individual edges carry no information. A flat strength-sort then fills
 * the edge budget with arbitrary clique members.
 *
 * selectIntentionalLinks keeps a link only when it is *nominated* by one
 * of its endpoints (among that node's top-K strongest links) or when an
 * endpoint is a low-degree node whose links are all load-bearing
 * (chains, bridges, leaves — dropping them erases structure).
 */

export type SelectableLink = { a: string; b: string; strength: number };

export type LinkSelectOptions = {
  /** Max links a node nominates as its strongest. */
  nominationsPerNode?: number;
  /** Nodes at/below this degree keep every link they have. */
  lowDegreeKeepAll?: number;
  /** Hard budget on returned links. */
  maxLinks?: number;
};

const linkKey = (a: string, b: string) => (a < b ? `${a}${b}` : `${b}${a}`);

export function selectIntentionalLinks(
  links: SelectableLink[],
  opts: LinkSelectOptions = {},
): SelectableLink[] {
  const k = opts.nominationsPerNode ?? 3;
  const lowDegree = opts.lowDegreeKeepAll ?? 2;
  const maxLinks = opts.maxLinks ?? 1200;

  const degree = new Map<string, number>();
  const incident = new Map<string, SelectableLink[]>();
  const bump = (id: string, link: SelectableLink) => {
    degree.set(id, (degree.get(id) ?? 0) + 1);
    const list = incident.get(id);
    if (list) list.push(link);
    else incident.set(id, [link]);
  };
  for (const link of links) {
    bump(link.a, link);
    bump(link.b, link);
  }

  // Each node nominates its top-K links: strength desc, key asc (stable).
  const nominated = new Set<string>();
  incident.forEach((list) => {
    list
      .slice()
      .sort(
        (x, y) =>
          y.strength - x.strength || linkKey(x.a, x.b).localeCompare(linkKey(y.a, y.b)),
      )
      .slice(0, k)
      .forEach((link) => nominated.add(linkKey(link.a, link.b)));
  });

  const byStrengthThenKey = (x: SelectableLink, y: SelectableLink) =>
    y.strength - x.strength || linkKey(x.a, x.b).localeCompare(linkKey(y.a, y.b));

  // Low-degree links are unconditional — they are the load-bearing wiring
  // (a leaf's only edge must never lose a strength contest to clique fill).
  const isLowDegree = (link: SelectableLink) =>
    (degree.get(link.a) ?? 0) <= lowDegree || (degree.get(link.b) ?? 0) <= lowDegree;
  const lowKept = links.filter(isLowDegree);
  const lowKeys = new Set(lowKept.map((l) => linkKey(l.a, l.b)));

  const nominatedLinks = links
    .filter((l) => !lowKeys.has(linkKey(l.a, l.b)) && nominated.has(linkKey(l.a, l.b)))
    .sort(byStrengthThenKey);
  const budget = Math.max(0, maxLinks - lowKept.length);
  const kept = [...lowKept, ...nominatedLinks.slice(0, budget)];
  kept.sort(byStrengthThenKey);
  return kept;
}
