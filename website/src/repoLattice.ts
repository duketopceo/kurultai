/** Group graph atoms into a handful of lattices: connector source, or a real git repo. */

const UUID_HEAD =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/i;
const NUMERIC = /^\d+$/;
const GENERIC_DIR = new Set([
  'src',
  'lib',
  'pkg',
  'crates',
  'apps',
  'packages',
  'website',
  'ui',
  'docs',
  'test',
  'tests',
  'bin',
  'scripts',
]);

export function isJunkLatticeName(name: string): boolean {
  const s = name.trim();
  if (!s) return true;
  if (NUMERIC.test(s)) return true;
  if (UUID_HEAD.test(s)) return true;
  if (s.includes('#')) return true;
  if (s.includes(':') && /\d+$/.test(s)) return true;
  if (/\.(md|txt|json)$/i.test(s)) return true;
  return false;
}

export function isCodeSource(source: string): boolean {
  // `repos` is the hosted github-connector source name (dogfood); `repo` alone
  // must not miss the plural — `\brepo\b` does not match `repos`.
  return /^(code|github|git|repos?)\b/i.test(source) || /github/i.test(source);
}

function looksLikeRepoSegment(seg: string): boolean {
  if (isJunkLatticeName(seg)) return false;
  if (GENERIC_DIR.has(seg.toLowerCase())) return false;
  if (seg.startsWith('_') || seg.startsWith('.')) return false;
  if (/\.[a-z0-9]{1,8}$/i.test(seg)) return false;
  return /^[A-Za-z][\w.-]{1,80}$/.test(seg);
}

/** `owner/repo` when both sides look like GitHub names. */
function ownerRepoFromPath(path: string): string | null {
  const segs = path.split('/').filter(Boolean);
  for (let i = 0; i < segs.length - 1; i++) {
    const a = segs[i];
    const b = segs[i + 1];
    if (looksLikeRepoSegment(a) && looksLikeRepoSegment(b) && /[-_]/.test(b)) {
      return `${a}/${b}`;
    }
    if (
      looksLikeRepoSegment(a) &&
      looksLikeRepoSegment(b) &&
      /^[A-Z]/.test(a) === false &&
      a.length >= 2 &&
      b.length >= 2 &&
      !GENERIC_DIR.has(b.toLowerCase())
    ) {
      // Bartlett-Roofs/yam, duketopceo/kurultai
      if (/^[A-Za-z0-9-]+$/.test(a) && /^[A-Za-z0-9._-]+$/.test(b)) {
        return `${a}/${b}`;
      }
    }
  }
  return null;
}

export function latticeOf(atom: { source: string; source_id: string }): string {
  const source = (atom.source || '').trim() || 'uncategorized';
  const sid = (atom.source_id || '').replace(/\\/g, '/').split('#')[0].trim();

  if (isCodeSource(source) && sid.includes('/')) {
    const ownerRepo = ownerRepoFromPath(sid);
    if (ownerRepo) return ownerRepo;
  }

  return source;
}

/** Code-connector lattice only. Notes/pond/dayflow are not repos. */
export function codeLatticeOf(atom: { source: string; source_id: string }): string | null {
  if (!isCodeSource(atom.source || '')) return null;
  return latticeOf(atom);
}
