/** Brain v2 theme + effects prefs. Theme: Light / Dark / System (key shared with v1: `kurultai-theme`; absent = system). */
import { useCallback, useEffect, useState } from 'react';

export type ThemePref = 'system' | 'light' | 'dark';
export const THEME_KEY = 'kurultai-theme';
export const EFFECTS_KEY = 'kurultai.v2.reduceEffects';

export function parseThemePref(raw: string | null): ThemePref {
  // Light is the default for new visitors; 'system' is stored explicitly.
  return raw === 'dark' || raw === 'system' ? raw : 'light';
}

export function resolveTheme(pref: ThemePref, systemLight: boolean): 'light' | 'dark' {
  return pref === 'system' ? (systemLight ? 'light' : 'dark') : pref;
}

const get = (k: string): string | null => { try { return window.localStorage.getItem(k); } catch { return null; } };
const set = (k: string, v: string | null) => { try { if (v === null) window.localStorage.removeItem(k); else window.localStorage.setItem(k, v); } catch { /* storage blocked */ } };
const systemLight = () => { try { return window.matchMedia('(prefers-color-scheme: light)').matches; } catch { return false; } };

export function useTheme(): { pref: ThemePref; setPref: (p: ThemePref) => void; theme: 'light' | 'dark' } {
  const [pref, setPrefState] = useState<ThemePref>(() => parseThemePref(get(THEME_KEY)));
  const [sys, setSys] = useState<boolean>(() => systemLight());
  useEffect(() => {
    let mq: MediaQueryList | null = null;
    try { mq = window.matchMedia('(prefers-color-scheme: light)'); } catch { /* none */ }
    if (!mq) return;
    const on = () => setSys(mq!.matches);
    mq.addEventListener('change', on);
    return () => mq!.removeEventListener('change', on);
  }, []);
  const theme = resolveTheme(pref, sys);
  useEffect(() => { document.documentElement.dataset.theme = theme; }, [theme]);
  const setPref = useCallback((p: ThemePref) => { set(THEME_KEY, p); setPrefState(p); }, []);
  return { pref, setPref, theme };
}

/** "Reduce effects": freezes the background flow + breathing field. */
export function useReduceEffects(): [boolean, (on: boolean) => void] {
  const [on, setOn] = useState<boolean>(() => get(EFFECTS_KEY) === '1');
  const update = useCallback((v: boolean) => { set(EFFECTS_KEY, v ? '1' : '0'); setOn(v); }, []);
  return [on, update];
}
