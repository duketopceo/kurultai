#!/usr/bin/env python3
"""recall-harness — randomized recall/search load + correctness suite.

Exercises a Kurultai daemon the way an agent would: randomized shallow and
deep queries, volume, concurrency (including many agents hitting the same
pathway), correctness checks (did the atom we seeded/listed come back),
and failover (endpoint switch mid-run).

Stdlib only — runs anywhere python3 exists.

Usage:
  python3 scripts/recall-harness.py --base http://127.0.0.1:8421
  python3 scripts/recall-harness.py --base https://api-knowledge.shippedit.dev \
      --token kk_... --volume 200 --concurrency 8
  python3 scripts/recall-harness.py --base ... --failover-base https://127.0.0.1:9

Scenarios: corpus | shallow | deep | volume | same-path | mixed-path |
correctness | failover | all (default)
Output: console summary + optional --json report.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import random
import statistics
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass, field, asdict


@dataclass
class Sample:
    scenario: str
    query: str
    ok: bool
    ms: float
    status: int = 0
    hits: int = 0
    expected_hit: bool | None = None  # correctness: did the expected atom return
    error: str = ""


@dataclass
class Report:
    base: str
    started: float = field(default_factory=time.time)
    samples: list[Sample] = field(default_factory=list)

    def add(self, s: Sample) -> None:
        self.samples.append(s)


class Client:
    def __init__(self, base: str, token: str | None, timeout: float = 15.0):
        self.base = base.rstrip("/")
        self.token = token
        self.timeout = timeout

    def req(self, method: str, path: str, body: dict | None = None,
            base: str | None = None) -> tuple[int, object, float]:
        url = (base or self.base).rstrip("/") + path
        data = json.dumps(body).encode() if body is not None else None
        r = urllib.request.Request(url, data=data, method=method)
        # Cloudflare 1010s the default python-urllib UA — always send one.
        r.add_header("User-Agent", "recall-harness/1.0")
        if self.token:
            r.add_header("Authorization", f"Bearer {self.token}")
        if data:
            r.add_header("Content-Type", "application/json")
        if url.endswith("/mcp"):
            r.add_header("Accept", "application/json, text/event-stream")
        t0 = time.perf_counter()
        try:
            with urllib.request.urlopen(r, timeout=self.timeout) as resp:
                payload = json.loads(resp.read() or b"null")
                return resp.status, payload, (time.perf_counter() - t0) * 1000
        except urllib.error.HTTPError as e:
            return e.code, None, (time.perf_counter() - t0) * 1000
        except Exception as e:  # noqa: BLE001 — harness records, never throws
            return 0, None, (time.perf_counter() - t0) * 1000


class McpClient(Client):
    """Drives the agent path: POST /mcp JSON-RPC tools/call (search/recall)."""

    def __init__(self, base: str, token: str | None, timeout: float = 15.0):
        super().__init__(base, token, timeout)
        self._id = 0
        status, payload, _ = self.req("POST", "/mcp", {
            "jsonrpc": "2.0", "id": 0, "method": "initialize",
            "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                       "clientInfo": {"name": "recall-harness", "version": "0"}},
        })
        if status != 200:
            raise RuntimeError(f"mcp initialize failed: {status}")

    def call_tool(self, name: str, arguments: dict) -> tuple[int, list, float]:
        self._id += 1
        status, payload, ms = self.req("POST", "/mcp", {
            "jsonrpc": "2.0", "id": self._id, "method": "tools/call",
            "params": {"name": name, "arguments": arguments},
        })
        if status != 200 or not isinstance(payload, dict):
            return status, [], ms
        result = payload.get("result", {})
        if payload.get("error") or result.get("isError"):
            return 500, [], ms
        out: list = []
        for part in result.get("content", []):
            if part.get("type") == "text":
                try:
                    parsed = json.loads(part["text"])
                    if isinstance(parsed, list):
                        out.extend(parsed)
                except (json.JSONDecodeError, KeyError):
                    pass
        return status, out, ms


# ── corpus + query generation ────────────────────────────────────────────────

def build_corpus(c: Client, rep: Report, mcp: bool = False) -> list[dict]:
    if mcp and isinstance(c, McpClient):
        # REST corpus is gated on hosted lanes; harvest atoms from MCP search.
        atoms: dict[str, dict] = {}
        for probe in ("the", "brain", "note", "code", "agent"):
            status, rows, ms = c.call_tool("search", {"query": probe, "limit": 50})
            ok = status == 200
            for r in rows:
                if isinstance(r, dict) and r.get("id"):
                    atoms[r["id"]] = r
            rep.add(Sample("corpus", f"mcp search {probe!r}", ok, ms, status,
                           hits=len(atoms)))
        return list(atoms.values())
    status, payload, ms = c.req("GET", "/api/atoms?limit=500")
    if status != 200 or not isinstance(payload, list):
        rep.add(Sample("corpus", "GET /api/atoms", False, ms, status,
                       error="could not load atoms for corpus"))
        return []
    atoms = [x.get("atom", x) for x in payload if isinstance(x, dict)]
    rep.add(Sample("corpus", "GET /api/atoms", True, ms, status, hits=len(atoms)))
    return atoms


def queries(atoms: list[dict], rng: random.Random) -> dict[str, list[str]]:
    """Derive query pools from the real corpus: shallow = single tag/word,
    deep = multi-word from titles, expected = per-atom title probes."""
    shallow, deep, expected = [], [], []
    for a in atoms:
        title = str(a.get("title", "")).strip()
        tags = a.get("tags") or []
        if isinstance(tags, str):
            tags = [tags]
        for t in tags:
            shallow.append(str(t))
        words = [w for w in title.split() if len(w) > 3]
        if len(words) >= 2:
            deep.append(" ".join(rng.sample(words, min(3, len(words)))))
        elif words:
            shallow.extend(words)
        if title:
            expected.append(title)
    return {
        "shallow": shallow or ["test"],
        "deep": deep or ["knowledge brain"],
        "expected": expected,
    }


# ── scenarios ────────────────────────────────────────────────────────────────

def run_search(c: Client, rep: Report, scenario: str, q: str,
               expected_id: str | None = None, limit: int = 20) -> Sample:
    if isinstance(c, McpClient):
        status, payload, ms = c.call_tool("search", {"query": q, "limit": limit})
    else:
        status, payload, ms = c.req(
            "GET", f"/api/search?q={urllib.request.quote(q)}&limit={limit}")
    ok = status == 200 and isinstance(payload, list)
    hits = len(payload) if ok else 0
    exp = None
    if expected_id is not None:
        exp = ok and any(
            (r.get("atom") or r).get("id") == expected_id for r in payload
        )
    return Sample(scenario, q, ok, ms, status, hits, exp,
                  "" if ok else f"http {status}")


def run_recall(c: Client, rep: Report, scenario: str, q: str) -> Sample:
    if isinstance(c, McpClient):
        status, payload, ms = c.call_tool("recall", {"query": q, "limit": 10})
    else:
        status, payload, ms = c.req("POST", "/api/recall",
                                    {"query": q, "limit": 10})
    ok = status == 200 and isinstance(payload, list)
    return Sample(scenario, q, ok, ms, status,
                  len(payload) if ok else 0, None,
                  "" if ok else f"http {status}")


def scenario_volume(c: Client, rep: Report, pools: dict, rng: random.Random,
                    n: int) -> None:
    for _ in range(n):
        q = rng.choice(pools["shallow"])
        rep.add(run_search(c, rep, "volume", q))


def scenario_deep(c: Client, rep: Report, pools: dict, rng: random.Random,
                  n: int) -> None:
    for q in rng.sample(pools["deep"], min(n, len(pools["deep"]))):
        rep.add(run_recall(c, rep, "deep", q))


def scenario_same_path(c: Client, rep: Report, pools: dict,
                       rng: random.Random, k: int) -> None:
    """k concurrent agents on ONE query — same pathway."""
    q = rng.choice(pools["shallow"])
    with concurrent.futures.ThreadPoolExecutor(k) as ex:
        for s in ex.map(lambda _: run_search(c, rep, "same-path", q), range(k)):
            rep.add(s)


def scenario_mixed_path(c: Client, rep: Report, pools: dict,
                        rng: random.Random, k: int) -> None:
    qs = [rng.choice(pools["shallow"]) for _ in range(k)]
    with concurrent.futures.ThreadPoolExecutor(k) as ex:
        for s in ex.map(lambda q: run_search(c, rep, "mixed-path", q), qs):
            rep.add(s)


def scenario_correctness(c: Client, rep: Report, atoms: list[dict],
                         rng: random.Random, n: int) -> None:
    """Search an atom's own title; it should come back."""
    for a in rng.sample(atoms, min(n, len(atoms))):
        title = str(a.get("title", "")).strip()
        if not title:
            continue
        rep.add(run_search(c, rep, "correctness", title, expected_id=a.get("id")))


def scenario_failover(c: Client, rep: Report, dead_base: str, pools: dict,
                      rng: random.Random) -> None:
    """Try a dead endpoint, then the live one — measures recovery."""
    q = rng.choice(pools["shallow"])
    status, _, ms = c.req("GET", f"/api/search?q={urllib.request.quote(q)}&limit=5",
                          base=dead_base)
    rep.add(Sample("failover-dead", q, status == 200, ms, status,
                   error="" if status == 200 else f"unreachable ({status})"))
    t0 = time.perf_counter()
    rep.add(run_search(c, rep, "failover-live", q))
    # recovery time = time until live endpoint answers after dead one failed
    rep.samples[-1].error = f"recovered in {time.perf_counter() - t0:.0f}ms total"


# ── summary ──────────────────────────────────────────────────────────────────

def pct(xs: list[float], p: float) -> float:
    if not xs:
        return 0.0
    xs = sorted(xs)
    i = min(len(xs) - 1, int(len(xs) * p / 100))
    return xs[i]


def summarize(rep: Report) -> None:
    print(f"\n=== recall-harness — {rep.base} ===")
    groups: dict[str, list[Sample]] = {}
    for s in rep.samples:
        groups.setdefault(s.scenario, []).append(s)
    print(f"{'scenario':<16}{'n':>5}{'ok%':>7}{'hits':>6}{'p50ms':>8}{'p95ms':>8}{'p99ms':>8}{'exp%':>7}")
    for name, xs in groups.items():
        lat = [s.ms for s in xs]
        ok = sum(s.ok for s in xs) / len(xs) * 100
        hits = sum(s.hits for s in xs)
        exps = [s.expected_hit for s in xs if s.expected_hit is not None]
        exp_s = f"{sum(1 for e in exps if e) / len(exps) * 100:.0f}%" if exps else "—"
        print(f"{name:<16}{len(xs):>5}{ok:>6.0f}%{hits:>6}"
              f"{pct(lat,50):>8.0f}{pct(lat,95):>8.0f}{pct(lat,99):>8.0f}{exp_s:>7}")
    lat = [s.ms for s in rep.samples]
    errs = [s for s in rep.samples if not s.ok]
    if lat:
        print(f"\ntotal {len(lat)} reqs · mean {statistics.fmean(lat):.0f}ms "
              f"· p95 {pct(lat,95):.0f}ms · errors {len(errs)}")
    for s in errs[:10]:
        print(f"  err [{s.scenario}] {s.query!r} → {s.error}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--base", default="http://127.0.0.1:8421")
    ap.add_argument("--token", default=None, help="bearer key (hosted lanes)")
    ap.add_argument("--failover-base", default="http://127.0.0.1:9",
                    help="endpoint that should be dead/unreachable")
    ap.add_argument("--scenario", default="all",
                    choices=["all", "corpus", "shallow", "deep", "volume",
                             "same-path", "mixed-path", "correctness", "failover"])
    ap.add_argument("--volume", type=int, default=100, help="volume query count")
    ap.add_argument("--deep", type=int, default=20, help="deep recall count")
    ap.add_argument("--concurrency", "-c", type=int, default=8)
    ap.add_argument("--correctness", type=int, default=25)
    ap.add_argument("--seed", type=int, default=42)
    ap.add_argument("--timeout", type=float, default=15.0)
    ap.add_argument("--mcp", action="store_true",
                    help="drive tools/call over POST /mcp (the real agent path)")
    ap.add_argument("--json", default=None, help="write full report to path")
    args = ap.parse_args()

    rng = random.Random(args.seed)
    c: Client = (McpClient(args.base, args.token, args.timeout) if args.mcp
                 else Client(args.base, args.token, args.timeout))
    rep = Report(base=args.base + (" ·mcp" if args.mcp else ""))
    run = lambda name: args.scenario in ("all", name)

    atoms = build_corpus(c, rep, mcp=args.mcp) if run("corpus") or args.scenario == "all" else []
    pools = queries(atoms, rng) if atoms else {"shallow": ["test"], "deep": ["knowledge brain"], "expected": []}

    if run("shallow") or run("volume"):
        scenario_volume(c, rep, pools, rng, args.volume)
    if run("deep"):
        scenario_deep(c, rep, pools, rng, args.deep)
    if run("same-path"):
        scenario_same_path(c, rep, pools, rng, args.concurrency)
    if run("mixed-path"):
        scenario_mixed_path(c, rep, pools, rng, args.concurrency)
    if run("correctness") and atoms:
        scenario_correctness(c, rep, atoms, rng, args.correctness)
    if run("failover"):
        scenario_failover(c, rep, args.failover_base, pools, rng)

    summarize(rep)
    if args.json:
        with open(args.json, "w") as f:
            json.dump({"base": rep.base, "started": rep.started,
                       "samples": [asdict(s) for s in rep.samples]}, f, indent=2)
        print(f"report → {args.json}")
    return 0 if all(s.ok or s.scenario.startswith("failover-dead") for s in rep.samples) else 1


if __name__ == "__main__":
    sys.exit(main())
