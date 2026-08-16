export function createFdgWorker(): Worker {
  return new Worker(new URL('./fdg.worker.ts', import.meta.url), { type: 'module' });
}
