/** `?brain=v2` selects the v2 renderer; anything else keeps v1 (`?brain=v1`). */
export type BrainVersion = 'v1' | 'v2';

export function parseBrainVersion(search: string): BrainVersion {
  const v = new URLSearchParams(search).get('brain');
  return v === 'v2' ? 'v2' : 'v1';
}

export function brainVersion(): BrainVersion {
  try {
    return parseBrainVersion(window.location.search);
  } catch {
    return 'v1';
  }
}
