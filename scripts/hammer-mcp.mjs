#!/usr/bin/env node
/**
 * hammer-mcp.mjs — load test for the MCP stdio lane (`kurultai mcp`).
 *
 * Spawns the real MCP stdio server, performs the initialize handshake,
 * then fires concurrent JSON-RPC tools/call traffic and measures per-call
 * latency. This is the lane agents actually use — HTTP numbers don't cover
 * stdio serialization.
 *
 * Usage:
 *   node scripts/hammer-mcp.mjs [--bin ./target/debug/kurultai]
 *                               [--config ~/.config/kurultai/config.toml]
 *                               [--conc 50] [--calls 200]
 *                               [--tool search|remember|hey_post|who_knows]
 */

import { spawn } from 'node:child_process';

const args = process.argv.slice(2);
const opt = (n, d) => { const i = args.indexOf('--' + n); return i >= 0 ? args[i + 1] : d; };
const BIN = opt('bin', './target/debug/kurultai');
const CONFIG = opt('config', process.env.HOME + '/.config/kurultai/config.toml');
const CONC = parseInt(opt('conc', '50'), 10);
const CALLS = parseInt(opt('calls', '200'), 10);
const TOOL = opt('tool', 'search');

const TOOL_ARGS = {
  search: { query: 'memory', limit: 10 },
  who_knows: { topic: 'brain' },
  remember: { title: `hammer-mcp ${Date.now()}`, content: 'hammer-mcp write probe', tags: ['hammer-test'] },
  hey_post: { content: `hammer-mcp post ${Date.now()}`, thread: 'hammer-test-x', instance_id: 'hammer-mcp' },
};

const pct = (a, p) => {
  if (!a.length) return 0;
  const s = [...a].sort((x, y) => x - y);
  return s[Math.min(s.length - 1, (s.length * p) | 0)];
};

const child = spawn(BIN, ['--config', CONFIG, 'mcp'], { stdio: ['pipe', 'pipe', 'inherit'] });
let buf = '';
const pending = new Map();
let nextId = 1;

child.stdout.on('data', (d) => {
  buf += d.toString();
  let nl;
  while ((nl = buf.indexOf('\n')) >= 0) {
    const line = buf.slice(0, nl).trim();
    buf = buf.slice(nl + 1);
    if (!line) continue;
    try {
      const msg = JSON.parse(line);
      const p = pending.get(msg.id);
      if (p) { pending.delete(msg.id); p(msg); }
    } catch { /* notifications/log lines */ }
  }
});

function rpc(method, params) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    const t = performance.now();
    pending.set(id, (msg) => resolve({ ms: performance.now() - t, error: !!msg.error, msg }));
    child.stdin.write(JSON.stringify({ jsonrpc: '2.0', id, method, params }) + '\n');
    setTimeout(() => {
      if (pending.delete(id)) reject(new Error('timeout'));
    }, 30000);
  });
}

// handshake
const init = await rpc('initialize', {
  protocolVersion: '2024-11-05',
  capabilities: {},
  clientInfo: { name: 'hammer-mcp', version: '0.0.0' },
});
if (init.error) { console.error('initialize failed', init.msg); process.exit(1); }
child.stdin.write(JSON.stringify({ jsonrpc: '2.0', method: 'notifications/initialized' }) + '\n');

console.log(`mcp lane: tool=${TOOL} conc=${CONC} calls=${CALLS}`);

const lat = [];
let ok = 0, errs = 0;
let inflight = 0, sent = 0;
const t0 = Date.now();
let doneCount = 0;
await new Promise((done) => {
  const pump = () => {
    while (sent < CALLS && inflight < CONC) {
      sent++;
      inflight++;
      rpc('tools/call', { name: TOOL, arguments: TOOL_ARGS[TOOL] })
        .then((r) => { lat.push(r.ms); r.error ? errs++ : ok++; })
        .catch(() => errs++)
        .finally(() => {
          inflight--;
          doneCount++;
          if (doneCount >= CALLS) return done();
          pump();
        });
    }
  };
  pump();
});
const wall = (Date.now() - t0) / 1000;
console.log(`calls=${ok + errs} ok=${ok} err=${errs} rps=${((ok + errs) / wall).toFixed(1)} p50=${pct(lat, .5) | 0}ms p95=${pct(lat, .95) | 0}ms p99=${pct(lat, .99) | 0}ms max=${Math.max(0, ...lat) | 0}ms`);
child.kill();
process.exit(0);
