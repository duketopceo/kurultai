/** 32-bit mode (for fun): persisted in localStorage (try/catch), forced by `?mode=32bit`, off by `?mode=normal`. */
import { useCallback, useEffect, useState } from 'react';

export const MODE32_KEY = 'kurultai.mode32';

export function parseMode32(search: string): boolean | null {
  const v = new URLSearchParams(search).get('mode');
  if (v === '32bit' || v === '32') return true;
  if (v === 'normal' || v === '64bit') return false;
  return null;
}

export function readMode32(): boolean {
  try {
    const fromUrl = parseMode32(window.location.search);
    if (fromUrl !== null) return fromUrl;
  } catch { /* no window */ }
  try {
    return window.localStorage.getItem(MODE32_KEY) === '1';
  } catch {
    return false;
  }
}

export function writeMode32(on: boolean): void {
  try { window.localStorage.setItem(MODE32_KEY, on ? '1' : '0'); } catch { /* private mode: keep in memory */ }
}

/** Hook: current mode + setter; mirrors the mode onto <html data-mode> so CSS can switch instantly. */
export function useMode32(): [boolean, (on: boolean) => void] {
  const [on, setOn] = useState<boolean>(() => readMode32());
  useEffect(() => { document.documentElement.dataset.mode = on ? '32bit' : 'normal'; }, [on]);
  const set = useCallback((v: boolean) => { writeMode32(v); setOn(v); }, []);
  return [on, set];
}
