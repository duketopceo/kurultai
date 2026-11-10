/** Brain v2 theme + effects prefs. Theme: Light / Dark / System (key shared with v1: `kurultai-theme`; absent = system). */
import { useCallback, useEffect, useState } from 'react';

export type ThemePref = 'system' | 'light' | 'dark';
export const THEME_KEY = 'kurultai-theme';
export const EFFECTS_KEY = 'kurultai.v2.reduceEffects';

/** Lane default for a visitor with no stored choice: work.* opens light, knowledge.* opens dark, anything else light. */
export function laneDefaultTheme(hostname: string): 'light' | 'dark' {
  const h = hostname.toLowerCase();
  return h === 'knowledge' || h.startsWith('knowledge.') ? 'dark' : 'light';
}

export function parseThemePref(raw: string | null, hostname = ''): ThemePref {
  // A stored choice wins ('system' is stored explicitly); otherwise the lane decides.
  return raw === 'light' || raw === 'dark' || raw === 'system' ? raw : laneDefaultTheme(hostname);
}

export function resolveTheme(pref: ThemePref, systemLight: boolean): 'light' | 'dark' {
  return pref === 'system' ? (systemLight ? 'light' : 'dark') : pref;
}

const get = (k: string): string | null => { try { return window.localStorage.getItem(k); } catch { return null; } };
const set = (k: string, v: string | null) => { try { if (v === null) window.localStorage.removeItem(k); else window.localStorage.setItem(k, v); } catch { /* storage blocked */ } };
const systemLight = () => { try { return window.matchMedia('(prefers-color-scheme: light)').matches; } catch { return false; } };

export function useTheme(): { pref: ThemePref; setPref: (p: ThemePref) => void; theme: 'light' | 'dark' } {
  const [pref, setPrefState] = useState<ThemePref>(() => parseThemePref(get(THEME_KEY), typeof location === 'undefined' ? '' : location.hostname));
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
