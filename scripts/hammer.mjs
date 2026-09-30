#!/usr/bin/env node
/**
 * hammer.mjs — load/chaos harness for the kurultai daemon (and peers).
 *
 * Zero-dep Node runner: concurrency ramp, weighted route pools, latency
 * histograms, error buckets, optional RSS watcher, JSON artifacts.
 *
 * Usage:
 *   node scripts/hammer.mjs [--target http://127.0.0.1:8421]
 *                           [--auth "Authorization: Bearer x"]
 *                           [--suite read|search-adversarial|payload-ceiling|
 *                                    write-burst|write-race|turn-cap|consistency|soak]
 *                           [--conc 10,50,100,200] [--dur 8000]
 *                           [--json artifacts/hammer.json] [--keep]
 *                           [--thread hammer-test-x]
 *
 * Guardrails: write suites only touch `hammer-test*` namespaces and print a
 * cleanup recipe; hosted targets need --auth (or run via browser page-eval).
 */

const args = process.argv.slice(2);
function opt(name, dflt) {
  const i = args.indexOf('--' + name);
  return i >= 0 ? args[i + 1] : dflt;
}
const TARGET = opt('target', 'http://127.0.0.1:8421').replace(/\/$/, '');
const AUTH = opt('auth', null); // e.g. "Authorization: Bearer tok"
const SUITE = opt('suite', 'read');
const CONC = opt('conc', '10,50,100,200').split(',').map(Number);
const DUR = parseInt(opt('dur', '8000'), 10);
const JSON_OUT = opt('json', null);
const KEEP = args.includes('--keep');
const THREAD = opt('thread', 'hammer-test-x');

const AUTH_HEADERS = AUTH
  ? Object.fromEntries([AUTH.split(/:\s*(.*)/s).slice(0, 2)])
  : {};

const pct = (a, p) => {
  if (!a.length) return 0;
  const s = [...a].sort((x, y) => x - y);
  return s[Math.min(s.length - 1, (s.length * p) | 0)];
};

async function hit(route, init = {}) {
  const t = performance.now();
  try {
    const res = await fetch(TARGET + route, {
      ...init,
      headers: { ...AUTH_HEADERS, ...(init.headers || {}) },
      signal: AbortSignal.timeout(30000),
    });
    const body = await res.arrayBuffer();
    return { ok: res.ok, status: res.status, ms: performance.now() - t, bytes: body.byteLength };
  } catch {
    return { ok: false, status: 0, ms: performance.now() - t, err: true };
  }
}

async function worker(dur, fn, st, latKey) {
  const end = Date.now() + dur;
  while (Date.now() < end) {
    const r = await fn();
    st[latKey + 'N']++;
    st[latKey + 'Lat'].push(r.ms);
    if (r.err) st[latKey + 'Err']++;
    else if (!r.ok) st[latKey + 'Bad']++;
  }
}

function pool(routes) {
  const p = [];
  for (const [r, w] of routes) for (let i = 0; i < w; i++) p.push(r);
  return () => p[(Math.random() * p.length) | 0];
}

const READ_ROUTES = [
  ['/api/graph', 3],
  ['/api/graph?tier=hot', 2],
  ['/api/search?q=memory&limit=20', 3],
  ['/api/search?q=brain&limit=5', 2],
  ['/api/atoms?limit=50', 2],
  ['/api/status', 3],
  ['/api/hey/threads', 2],
  ['/api/ontology', 1],
  ['/api/metrics', 1],
  ['/ui/', 1],
];

const ADVERSARIAL_QUERIES = [
  '" OR "', 'NEAR/0', 'a*', '*', '""', 'a'.repeat(10000),
  '日本語 テスト 🧠', "'; DROP TABLE atoms;--", 'title:("x" OR "y"',
  '((((((((((', 'memory NOT memory NOT memory', 'x AND ',
].map((q) => '/api/search?q=' + encodeURIComponent(q) + '&limit=20');
ADVERSARIAL_QUERIES.push('/api/atoms?limit=100000', '/api/search?q=test&limit=-1', '/api/search?q=test&limit=0');

async function suiteRead(conc) {
  const pick = pool(READ_ROUTES);
  const st = { rN: 0, rErr: 0, rBad: 0, rLat: [] };
  const t0 = Date.now();
  await Promise.all(Array.from({ length: conc }, () => worker(DUR, () => hit(pick()), st, 'r')));
  return fmt('read', conc, st, (Date.now() - t0) / 1000);
}

async function suiteAdversarial(conc) {
  const pick = pool(ADVERSARIAL_QUERIES.map((q) => [q, 1]));
  const st = { rN: 0, rErr: 0, rBad: 0, rLat: [] };
  const t0 = Date.now();
  await Promise.all(Array.from({ length: conc }, () => worker(DUR, () => hit(pick()), st, 'r')));
  // health check — adversarial must not wedge the daemon
  const h = await hit('/health');
  return fmt('adversarial', conc, st, (Date.now() - t0) / 1000) + `  post-health=${h.status}`;
}

async function suitePayload(conc) {
  const sizes = [1024, 100 * 1024, 1024 * 1024, 10 * 1024 * 1024];
  const results = [];
  for (const sz of sizes) {
    const body = 'x'.repeat(sz);
    const r = await hit('/api/hey/threads', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: `hammer-payload-${sz}`, turn_cap: 1, _pad: body }),
    });
    results.push(`${sz}B→${r.status}@${r.ms | 0}ms`);
  }
  return `payload  ${results.join('  ')}`;
}

let hammerSeq = Math.floor(Math.random() * 1e6);
let writeIds = [];

async function postMsg(threadId) {
  const r = await hit(`/api/hey/threads/${threadId}/messages`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      content: `hammer-test write ${hammerSeq++} ${Date.now()}`,
      instance_id: 'hammer-agent',
    }),
  });
  return r;
}

async function ensureThread(name, cap = 10000) {
  const r = await hit('/api/hey/threads', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, turn_cap: cap }),
  });
  if (!r.ok) return null;
  // re-fetch to get id
  const list = await fetch(TARGET + '/api/hey/threads', { headers: AUTH_HEADERS }).then((x) => x.json());
  return (list.find((t) => t.name === name) || {}).id || null;
}

async function suiteWriteBurst(conc) {
  const tid = await ensureThread(THREAD);
  if (!tid) return `write-burst  ERROR: no auth for ${THREAD} thread`;
  const readers = pool(READ_ROUTES.slice(0, 7));
  const st = { wN: 0, wErr: 0, wBad: 0, wLat: [], rN: 0, rErr: 0, rBad: 0, rLat: [] };
  const t0 = Date.now();
  await Promise.all([
    ...Array.from({ length: conc }, () => worker(DUR, () => postMsg(tid), st, 'w')),
    ...Array.from({ length: conc }, () => worker(DUR, () => hit(readers()), st, 'r')),
  ]);
  return `write-burst conc=${conc}x2  w=${st.wN}(${st.wErr}e/${st.wBad}b) p50=${pct(st.wLat, .5) | 0} p99=${pct(st.wLat, .99) | 0}  r=${st.rN}(${st.rErr}e/${st.rBad}b) p50=${pct(st.rLat, .5) | 0} p99=${pct(st.rLat, .99) | 0}`;
}

async function suiteWriteRace() {
  const tid = await ensureThread('hammer-test-race', 10000);
  if (!tid) return 'write-race  ERROR: no auth';
  // N writers post to same thread simultaneously, then verify unique ids
  const N = 50;
  const results = await Promise.all(Array.from({ length: N }, () => postMsg(tid)));
  const okCount = results.filter((r) => r.ok).length;
  const list = await fetch(`${TARGET}/api/hey/threads/${tid}/messages?limit=500`, { headers: AUTH_HEADERS }).then((x) => x.json());
  const ids = new Set(list.map((m) => m.id));
  return `write-race  sent=${N} ok=${okCount} landed=${list.length} uniqueIds=${ids.size} ${ids.size === list.length ? 'NO-DUPES' : 'DUPES!'}`;
}

async function suiteTurnCap() {
  // small-cap thread; hammer past the cap
  const tid = await ensureThread('hammer-test-cap', 5);
  if (!tid) return 'turn-cap  ERROR: no auth';
  const results = [];
  for (let i = 0; i < 12; i++) results.push(await postMsg(tid));
  const codes = results.map((r) => r.status).join(',');
  const rejects = results.filter((r) => !r.ok).length;
  return `turn-cap  12 posts vs cap=5 → codes=[${codes}] rejects=${rejects} ${rejects > 0 ? 'CAP-BINDS' : 'CAP-BYPASSED(known gap for admin)'}`;
}

async function suiteConsistency(conc) {
  const tid = await ensureThread('hammer-test-consist', 10000);
  if (!tid) return 'consistency  ERROR: no auth';
  const st = { torn: 0, checks: 0 };
  const end = Date.now() + DUR;
  const write = async () => {
    while (Date.now() < end) await postMsg(tid);
  };
  const read = async () => {
    while (Date.now() < end) {
      const list = await fetch(`${TARGET}/api/hey/threads/${tid}/messages?limit=500`, { headers: AUTH_HEADERS })
        .then((x) => x.json())
        .catch(() => null);
      if (Array.isArray(list)) {
        st.checks++;
        const ids = list.map((m) => m.id);
        if (new Set(ids).size !== ids.length) st.torn++;
      }
    }
  };
  await Promise.all([...Array.from({ length: conc }, write), ...Array.from({ length: conc }, read)]);
  return `consistency  checks=${st.checks} torn=${st.torn} ${st.torn === 0 ? 'CLEAN' : 'TORN READS!'}`;
}

async function rssMb() {
  try {
    const { execSync } = await import('node:child_process');
    const pid = execSync("pgrep -f 'kurultai daemon' | head -1").toString().trim();
    return parseInt(execSync(`ps -o rss= -p ${pid}`).toString().trim()) / 1024;
  } catch {
    return null;
  }
}

async function suiteSoak(conc) {
  const pick = pool(READ_ROUTES);
  const st = { rN: 0, rErr: 0, rBad: 0, rLat: [] };
  const rssSamples = [];
  const r0 = await rssMb();
  if (r0 != null) rssSamples.push(r0);
  const t0 = Date.now();
  const watcher = setInterval(async () => {
    const r = await rssMb();
    if (r != null) rssSamples.push(r);
  }, 30000);
  await Promise.all(Array.from({ length: conc }, () => worker(DUR, () => hit(pick()), st, 'r')));
  clearInterval(watcher);
  const rEnd = await rssMb();
  if (rEnd != null) rssSamples.push(rEnd);
  const growth = rssSamples.length > 1 ? (((rssSamples.at(-1) - rssSamples[0]) / rssSamples[0]) * 100).toFixed(1) : '?';
  return fmt('soak', conc, st, (Date.now() - t0) / 1000) + `  rss=${rssSamples[0] ?? '?'}→${rssSamples.at(-1) ?? '?'}MB (${growth}%)`;
}

function fmt(name, conc, st, wall) {
  const n = st.rN;
  return `${name} conc=${conc}  reqs=${n} err=${st.rErr} bad=${st.rBad} rps=${(n / wall).toFixed(1)} p50=${pct(st.rLat, 0.5) | 0}ms p95=${pct(st.rLat, 0.95) | 0}ms p99=${pct(st.rLat, 0.99) | 0}ms max=${Math.max(0, ...st.rLat) | 0}ms`;
}

const SUITES = {
  read: { fn: suiteRead, ramp: true },
  'search-adversarial': { fn: suiteAdversarial, ramp: true },
  'payload-ceiling': { fn: suitePayload, ramp: false },
  'write-burst': { fn: suiteWriteBurst, ramp: true },
  'write-race': { fn: suiteWriteRace, ramp: false },
  'turn-cap': { fn: suiteTurnCap, ramp: false },
  consistency: { fn: suiteConsistency, ramp: false, conc: 5 },
  soak: { fn: suiteSoak, ramp: true },
};

const suite = SUITES[SUITE];
if (!suite) {
  console.error(`unknown suite '${SUITE}'; have: ${Object.keys(SUITES).join(', ')}`);
  process.exit(2);
}

const health = await hit('/health');
console.log(`target=${TARGET} suite=${SUITE} health=${health.status}`);
if (!health.ok) process.exit(1);

const results = [];
if (suite.ramp) {
  for (const c of CONC) {
    const line = await suite.fn(c);
    console.log(line);
    results.push(line);
  }
} else {
  const line = await suite.fn(suite.conc ?? CONC[0]);
  console.log(line);
  results.push(line);
}

if (!KEEP && SUITE.startsWith('write') || SUITE === 'consistency' || SUITE === 'turn-cap') {
  console.log('\ncleanup: hosted → SQL on server: DELETE FROM messages WHERE thread_id IN (SELECT id FROM threads WHERE name LIKE \'hammer-test%\');');
  console.log('         or per-message DELETE /api/hey/messages/{id}');
}

if (JSON_OUT) {
  const { writeFileSync, mkdirSync } = await import('node:fs');
  const { dirname } = await import('node:path');
  mkdirSync(dirname(JSON_OUT), { recursive: true });
  writeFileSync(JSON_OUT, JSON.stringify({ target: TARGET, suite: SUITE, ts: new Date().toISOString(), results }, null, 2));
  console.log(`json → ${JSON_OUT}`);
}
