"""Kurultai structured-write client via the binary's MCP stdio surface.

The daemon's HTTP surface is read-only by design (write containment policy):
`POST /api/recall` is a retrieval endpoint, and `POST /ingest` is disabled
without KURULTAI_INGEST_SECRET and forces quarantine. The only structured
write is the `remember` tool on the binary's MCP stdio server:

    kurultai --plain mcp [--agent-id ID] [--namespace NS]

Framing is newline-delimited JSON-RPC 2.0 (verified against kurultai 0.4.1):
    {"jsonrpc":"2.0","id":1,"method":"initialize",...}
    {"jsonrpc":"2.0","method":"notifications/initialized"}
    {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"remember",
     "arguments":{"title":..., "summary":..., "tags":[...]}}}

Returns {"ok": bool, "atom_id": str|None, "lane": str, "raw": str} where lane
is the trust lane (e.g. "trusted" or "quarantined:<reason>") reported by the
brain's write policy.
"""
from __future__ import annotations

import asyncio
import json
import re
from typing import Any

_RESULT_RE = re.compile(
    r"remembered atom id=(?P<id>\S+) lane=(?P<lane>\S+)(?: project=(?P<project>\S+))?"
)


def _child_env(cfg: dict[str, Any]) -> dict[str, str]:
    from usr.plugins.kurultai.helpers.config import build_env

    return build_env(cfg)


async def mcp_remember(
    cfg: dict[str, Any],
    title: str,
    summary: str,
    tags: list[str],
    project: str | None = None,
    agent_id: str = "agent-zero",
    timeout: float | None = None,
) -> dict[str, Any]:
    """Store one distilled atom through kurultai MCP stdio and report the lane."""
    from usr.plugins.kurultai.helpers.client import find_binary

    binary = find_binary(cfg)
    if not binary:
        return {
            "ok": False,
            "atom_id": None,
            "lane": "error:binary-missing",
            "raw": "kurultai binary not found (set binary_path in plugin settings)",
        }

    argv = [binary, "--plain", "mcp", "--agent-id", agent_id]
    if project:
        argv += ["--namespace", project]

    try:
        timeout = float(timeout or min(float(cfg.get("timeout_secs") or 60), 90.0))
    except (TypeError, ValueError):
        timeout = 60.0

    call_params = {
        "name": "remember",
        "arguments": {
            "title": title,
            "summary": summary,
            "tags": [str(t) for t in tags if str(t).strip()],
        },
    }
    lines = [
        json.dumps(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "agent-zero-kurultai", "version": "1.0"},
                },
            }
        ),
        json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        json.dumps(
            {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": call_params}
        ),
    ]

    try:
        proc = await asyncio.create_subprocess_exec(
            *argv,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            env=_child_env(cfg),
        )
    except Exception as exc:
        return {"ok": False, "atom_id": None, "lane": "error:spawn", "raw": str(exc)}

    payload = ("\n".join(lines) + "\n").encode("utf-8")

    async def _pump():
        return await proc.communicate(payload)

    try:
        stdout, stderr = await asyncio.wait_for(_pump(), timeout=timeout)
    except asyncio.TimeoutError:
        try:
            proc.kill()
        except Exception:
            pass
        return {
            "ok": False,
            "atom_id": None,
            "lane": "error:timeout",
            "raw": f"mcp remember timed out after {timeout:.0f}s",
        }

    raw = stdout.decode("utf-8", "replace")
    err = stderr.decode("utf-8", "replace")
    response = _extract_id_response(raw, 3)
    if response is None:
        return {
            "ok": False,
            "atom_id": None,
            "lane": "error:no-response",
            "raw": (raw or err)[:400],
        }
    if "error" in response:
        return {
            "ok": False,
            "atom_id": None,
            "lane": "error:rpc",
            "raw": json.dumps(response.get("error"))[:400],
        }

    text = ""
    try:
        content = (response.get("result") or {}).get("content") or []
        if content and isinstance(content[0], dict):
            text = str(content[0].get("text", ""))
    except Exception:
        text = json.dumps(response)[:400]

    if (response.get("result") or {}).get("isError"):
        return {"ok": False, "atom_id": None, "lane": "error:tool", "raw": text[:400]}

    match = _RESULT_RE.search(text)
    if not match:
        return {"ok": False, "atom_id": None, "lane": "error:parse", "raw": text[:400]}
    return {
        "ok": True,
        "atom_id": match.group("id"),
        "lane": match.group("lane"),
        "raw": text,
    }


def _extract_id_response(raw: str, request_id: int) -> dict[str, Any] | None:
    for line in raw.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            msg = json.loads(line)
        except Exception:
            continue
        if msg.get("id") == request_id and ("result" in msg or "error" in msg):
            return msg
    return None
