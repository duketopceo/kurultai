"""API handler: /api/plugins/kurultai/kproxy

Same-origin, whitelisting proxy that lets the embedded Kurultai Brain app
(webui/brainapp/) reach the loopback daemon (127.0.0.1:8421) without CORS
or direct browser access to the daemon.

Only the endpoints below are reachable, and only whitelisted query/body
fields are forwarded. POSTs require the standard A0 CSRF token (injected
by webui/brainapp/assets/bridge.js).

  ep=status    GET  /api/status
  ep=graph     GET  /api/graph
  ep=activity  GET  /api/activity
  ep=ontology  GET  /api/ontology
  ep=search    GET  /api/search?q=&limit=&source=&include_quarantine=
  ep=ask       POST /api/ask            body: {question}
  ep=touch     POST /api/touch          body: {atom_id}

  /api/open is intentionally excluded: it spawns a host process and must not
  be re-exported through the A0 plugin HTTP surface (CSRF / path injection risk).
"""
from __future__ import annotations

import asyncio
import json
import urllib.error
import urllib.parse
import urllib.request
from typing import Any

from flask import jsonify

from helpers.api import ApiHandler, Request, Response

DAEMON_URL = "http://127.0.0.1:8421"
PROXY_TIMEOUT_SECS = 20

# ep -> (method, daemon_path, allowed_query_keys, allowed_body_keys)
ROUTES: dict[str, tuple[str, str, tuple[str, ...], tuple[str, ...]]] = {
    "status": ("GET", "/api/status", (), ()),
    "graph": ("GET", "/api/graph", (), ()),
    "activity": ("GET", "/api/activity", (), ()),
    "ontology": ("GET", "/api/ontology", (), ()),
    "search": ("GET", "/api/search", ("q", "limit", "source", "include_quarantine"), ()),
    "ask": ("POST", "/api/ask", (), ("question",)),
    "touch": ("POST", "/api/touch", (), ("atom_id",)),
}


class Kproxy(ApiHandler):
    @classmethod
    def get_methods(cls) -> list[str]:
        return ["GET", "POST"]

    async def process(self, input: dict, request: Request) -> Response:
        ep = str(request.args.get("ep") or input.get("ep") or "").strip().lower()
        route = ROUTES.get(ep)
        if route is None:
            resp = jsonify(
                {"success": False, "error": f"Unknown kproxy endpoint: {ep or '(missing ep)'}"}
            )
            resp.status_code = 404
            return resp

        method, path, query_keys, body_keys = route
        params: dict[str, str] = {}
        for key in query_keys:
            value = request.args.get(key)
            if value is not None and str(value).strip() != "":
                params[key] = str(value)
        url = f"{DAEMON_URL}{path}"
        if params:
            url += "?" + urllib.parse.urlencode(params)

        body: bytes | None = None
        if method == "POST":
            payload = {key: input[key] for key in body_keys if key in input}
            body = json.dumps(payload).encode("utf-8")

        loop = asyncio.get_running_loop()
        status, payload_out = await loop.run_in_executor(None, self._fetch, method, url, body)
        resp = jsonify(payload_out)
        resp.status_code = status
        return resp

    @staticmethod
    def _fetch(method: str, url: str, body: bytes | None) -> tuple[int, Any]:
        req = urllib.request.Request(url, data=body, method=method)
        if body is not None:
            req.add_header("Content-Type", "application/json")
        req.add_header("Accept", "application/json")
        try:
            with urllib.request.urlopen(req, timeout=PROXY_TIMEOUT_SECS) as resp:
                return resp.status, json.loads(resp.read().decode("utf-8", "replace"))
        except urllib.error.HTTPError as exc:
            try:
                detail = json.loads(exc.read().decode("utf-8", "replace"))
            except Exception:
                detail = {"error": f"daemon HTTP {exc.code}"}
            return exc.code, detail
        except Exception as exc:
            return 502, {"success": False, "error": f"Kurultai daemon unreachable: {exc}"}
