/** Brain v2 is the default; `?brain=v1` keeps the previous renderer reachable. */
export type BrainVersion = 'v1' | 'v2';

export function parseBrainVersion(search: string): BrainVersion {
  const v = new URLSearchParams(search).get('brain');
  return v === 'v1' ? 'v1' : 'v2';
}

export function brainVersion(): BrainVersion {
  try {
    return parseBrainVersion(window.location.search);
  } catch {
    return 'v2';
  }
}
