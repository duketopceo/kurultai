import { authHeaders } from './auth';

/** Client perf telemetry (#102): felt-perf samples reported to the daemon so
 *  /api/metrics can show p50/p99 of real load times and FPS per tier.
 *  Numbers + enum labels only — never queries, ids, or URLs. */

export type PerfTier = 'low' | 'mid' | 'high' | 'max';

type Sample = { metric: string; tier?: PerfTier; value: number };

const FLUSH_MS = 30_000;
const MAX_QUEUE = 40;

let queue: Sample[] = [];
let flushTimer: number | null = null;
let currentTier: PerfTier | undefined;
let lastFps: number | null = null;
let longTasks = 0;
let started = false;

export function reportPerf(metric: string, value: number, tier?: PerfTier): void {
  if (!Number.isFinite(value) || value < 0) return;
  queue.push({ metric, value, tier: tier ?? currentTier });
  if (queue.length >= MAX_QUEUE) void flush();
}

/** Latest Brain FPS sample — held until the next flush emits it once. */
export function reportFps(fps: number): void {
  if (Number.isFinite(fps) && fps >= 0) lastFps = fps;
}

export function setPerfTier(tier: PerfTier): void {
  currentTier = tier;
}

async function flush(): Promise<void> {
  if (flushTimer !== null) {
    window.clearTimeout(flushTimer);
    flushTimer = null;
  }
  const samples = queue;
  queue = [];
  if (lastFps !== null) {
    samples.push({ metric: 'fps', tier: currentTier, value: lastFps });
    lastFps = null;
  }
  if (longTasks > 0) {
    samples.push({ metric: 'long_tasks', tier: currentTier, value: longTasks });
    longTasks = 0;
  }
  const heap = (performance as { memory?: { usedJSHeapSize: number } }).memory;
  if (heap) {
    samples.push({ metric: 'heap_mb', tier: currentTier, value: heap.usedJSHeapSize / 1e6 });
  }
  // Reschedule before awaiting so an idle tick or a hung POST can't kill the cadence.
  schedule();
  if (!samples.length) return;
  try {
    await fetch('/api/metrics/client', {
      method: 'POST',
      headers: authHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({ samples }),
      keepalive: true,
      signal: AbortSignal.timeout(15_000),
    });
  } catch {
    // Telemetry must never break the app — drop the batch.
  }
}

function schedule(): void {
  if (flushTimer === null) {
    flushTimer = window.setTimeout(() => void flush(), FLUSH_MS);
  }
}

/** Wire collection once per page load: nav timing, long tasks, flush cadence. */
export function initPerf(): void {
  if (started) return;
  started = true;

  const reportNav = () => {
    const nav = performance.getEntriesByType('navigation')[0] as
      | PerformanceNavigationTiming
      | undefined;
    if (nav?.duration) reportPerf('nav_ms', nav.duration);
  };
  if (document.readyState === 'complete') reportNav();
  else window.addEventListener('load', reportNav, { once: true });

  try {
    new PerformanceObserver((list) => {
      longTasks += list.getEntries().length;
    }).observe({ type: 'longtask', buffered: true });
  } catch {
    // longtask unsupported (Firefox) — skip.
  }

  schedule();
  window.addEventListener('pagehide', () => void flush(), { once: true });
}
