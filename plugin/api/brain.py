"""API handler: /api/plugins/kurultai/brain

Action-based JSON bridge to the loopback Kurultai daemon for the sidebar
launcher card and brain store:

  action=status  GET  /api/status  -> flattened {ok, version, atoms, trusted,
                        quarantine, hot, warm, cold}
  action=search  GET  /api/search?q=&limit= -> {results: [...]}
  action=ask     POST /api/ask {question} -> daemon answer payload

Accepts GET query params or POST JSON body. Never touches API keys.
"""
from __future__ import annotations

import asyncio
import json
import urllib.parse
import urllib.request
from typing import Any

from helpers.api import ApiHandler, Request, Response

KURULTAI_URL = "http://127.0.0.1:8421"
_TIMEOUT_SECS = 15


class Brain(ApiHandler):
    @classmethod
    def get_methods(cls) -> list[str]:
        return ["GET", "POST"]

    async def process(self, input: dict, request: Request) -> dict:
        input = input or {}
        action = str(input.get("action") or request.args.get("action") or "status").lower()
        loop = asyncio.get_running_loop()
        try:
            if action == "status":
                data = await loop.run_in_executor(
                    None, self._fetch_json, "GET", f"{KURULTAI_URL}/api/status", None
                )
                brain = data.get("brain") or {}
                memory = data.get("memory") or {}
                return {
                    "success": True,
                    "ok": bool(data.get("ok")),
                    "version": data.get("version"),
                    "atoms": data.get("atoms"),
                    "trusted": brain.get("trusted_count"),
                    "quarantine": brain.get("quarantine_count"),
                    "hot": memory.get("hot"),
                    "warm": memory.get("warm"),
                    "cold": memory.get("cold"),
                }
            if action == "search":
                query = str(input.get("query") or request.args.get("q") or "").strip()
                if not query:
                    return {"success": False, "error": "No query provided"}
                try:
                    limit = int(input.get("limit") or request.args.get("limit") or 8)
                except (TypeError, ValueError):
                    limit = 8
                limit = max(1, min(50, limit))
                params = urllib.parse.urlencode({"q": query, "limit": limit})
                results = await loop.run_in_executor(
                    None, self._fetch_json, "GET", f"{KURULTAI_URL}/api/search?{params}", None
                )
                items = results if isinstance(results, list) else []
                return {"success": True, "results": items, "count": len(items)}
            if action == "ask":
                question = str(input.get("question") or input.get("query") or "").strip()
                if not question:
                    return {"success": False, "error": "No question provided"}
                answer = await loop.run_in_executor(
                    None,
                    self._fetch_json,
                    "POST",
                    f"{KURULTAI_URL}/api/ask",
                    {"question": question},
                )
                if isinstance(answer, dict):
                    return {"success": True, **answer}
                return {"success": True, "answer": str(answer)}
            return {"success": False, "error": f"Unknown action: {action}"}
        except Exception as e:
            return {"success": False, "error": f"Kurultai daemon error: {e}"}

    @staticmethod
    def _fetch_json(method: str, url: str, payload: dict[str, Any] | None) -> Any:
        data = json.dumps(payload).encode("utf-8") if payload is not None else None
        req = urllib.request.Request(url, data=data, method=method)
        if data is not None:
            req.add_header("Content-Type", "application/json")
        req.add_header("Accept", "application/json")
        with urllib.request.urlopen(req, timeout=_TIMEOUT_SECS) as resp:
            return json.loads(resp.read().decode("utf-8", "replace"))
