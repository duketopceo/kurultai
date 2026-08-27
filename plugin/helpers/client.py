"""Kurultai transport client — CLI subprocess and HTTP daemon modes.

Mode selection:
- CLI mode (always available when the binary exists): shells out to
  `kurultai search|ask|who-knows|status` and returns the human-readable
  stdout. Works with no daemon and no API key (FTS-only mode).
- HTTP mode (when server_url is configured and reachable): calls the
  `kurultai daemon` JSON API (`/api/search`, `/api/ask`, `/api/status`,
  `/api/recall`) and formats structured results with citations.

Stdlib only — no third-party packages required.
"""
from __future__ import annotations

import asyncio
import json
import shutil
import urllib.error
import urllib.request
from dataclasses import dataclass
from typing import Any

from usr.plugins.kurultai.helpers.config import build_env

import time

# Health-check TTL cache: avoid repeated /health round-trips within 5s
_health_cache: dict[str, tuple[bool, float]] = {}
_HEALTH_TTL = 5.0


@dataclass
class KurultaiResult:
    ok: bool
    text: str
    transport: str = "cli"  # "cli" | "http"
    detail: str = ""


def find_binary(cfg: dict[str, Any]) -> str | None:
    """Resolve the kurultai binary path, or None if not installed."""
    explicit = (cfg.get("binary_path") or "").strip()
    if explicit:
        return explicit if shutil.which(explicit) or _is_executable(explicit) else None
    return shutil.which("kurultai")


def _is_executable(path: str) -> bool:
    import os

    return os.path.isfile(os.path.expanduser(path)) and os.access(
        os.path.expanduser(path), os.X_OK
    )


async def _run_cli(cfg: dict[str, Any], args: list[str]) -> KurultaiResult:
    binary = find_binary(cfg)
    if not binary:
        return KurultaiResult(
            ok=False,
            text="kurultai binary not found. Install it (see plugin README) "
            "and/or set the binary path in the Kurultai plugin settings.",
            detail="binary-missing",
        )
    cmd = [binary, "--plain", *args]
    try:
        proc = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            env=build_env(cfg),
        )
        stdout, stderr = await asyncio.wait_for(
            proc.communicate(), timeout=cfg["timeout_secs"]
        )
    except asyncio.TimeoutError:
        return KurultaiResult(
            ok=False,
            text=f"kurultai {' '.join(args)} timed out after {cfg['timeout_secs']}s.",
            detail="timeout",
        )
    except Exception as exc:  # pragma: no cover - defensive
        return KurultaiResult(ok=False, text=f"failed to run kurultai: {exc}", detail="spawn-error")

    out = stdout.decode("utf-8", "replace").strip()
    err = stderr.decode("utf-8", "replace").strip()
    if proc.returncode != 0:
        detail = err or out or f"exit code {proc.returncode}"
        return KurultaiResult(ok=False, text=f"kurultai error: {detail}", detail="exit-nonzero")
    return KurultaiResult(ok=True, text=out or "(no output)", transport="cli", detail=err)


def _http_request(
    cfg: dict[str, Any], method: str, path: str, payload: dict[str, Any] | None = None
) -> dict[str, Any]:
    url = f"{cfg['server_url']}{path}"
    data = json.dumps(payload).encode("utf-8") if payload is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    if data is not None:
        req.add_header("Content-Type", "application/json")
    req.add_header("Accept", "application/json")
    with urllib.request.urlopen(req, timeout=cfg["timeout_secs"]) as resp:
        return json.loads(resp.read().decode("utf-8", "replace"))


async def _http_json(
    cfg: dict[str, Any], method: str, path: str, payload: dict[str, Any] | None = None
) -> dict[str, Any]:
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(None, _http_request, cfg, method, path, payload)


async def http_available(cfg: dict[str, Any]) -> bool:
    """True when a daemon base URL is configured and answers /health.

    Results are cached for _HEALTH_TTL seconds to eliminate redundant
    round-trips during rapid tool calls.
    """
    url = cfg.get("server_url") or ""
    if not url:
        return False
    now = time.time()
    cached = _health_cache.get(url)
    if cached and (now - cached[1]) < _HEALTH_TTL:
        return cached[0]
    try:
        # Use a short 3s timeout for health pings to avoid blocking the UI
        loop = asyncio.get_running_loop()
        await loop.run_in_executor(None, _http_request_timeout, cfg, "GET", "/health", 3)
        _health_cache[url] = (True, now)
        return True
    except Exception:
        _health_cache[url] = (False, now)
        return False


def _http_request_timeout(cfg: dict[str, Any], method: str, path: str, timeout: float) -> dict[str, Any]:
    """Like _http_request but with a custom timeout (for health pings)."""
    url = f"{cfg['server_url']}{path}"
    req = urllib.request.Request(url, method=method)
    req.add_header("Accept", "application/json")
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8", "replace"))


async def call(
    cfg: dict[str, Any],
    operation: str,
    cli_args: list[str],
    http_method: str | None = None,
    http_path: str | None = None,
    http_payload: dict[str, Any] | None = None,
    formatter=None,
) -> KurultaiResult:
    """Run one kurultai operation, preferring the daemon when configured.

    operation: human name for error messages ("search", "ask", ...)
    cli_args: CLI fallback argv after the binary (e.g. ["search", q, "--limit", "8"])
    http_*: daemon endpoint; when None the operation is CLI-only
    formatter: callable(dict) -> str for JSON responses
    """
    if cfg.get("prefer_http") and http_path and await http_available(cfg):
        try:
            data = await _http_json(cfg, http_method or "GET", http_path, http_payload)
            text = formatter(data) if formatter else json.dumps(data, indent=2)
            return KurultaiResult(ok=True, text=text, transport="http")
        except urllib.error.HTTPError as exc:
            body = ""
            try:
                body = exc.read().decode("utf-8", "replace")[:400]
            except Exception:
                pass
            return KurultaiResult(
                ok=False,
                text=f"kurultai daemon {operation} failed: HTTP {exc.code} {body}".strip(),
                transport="http",
                detail="http-error",
            )
        except Exception as exc:
            return KurultaiResult(
                ok=False,
                text=f"kurultai daemon {operation} failed: {exc}",
                transport="http",
                detail="http-error",
            )
    return await _run_cli(cfg, cli_args)


def format_hits(data: Any) -> str:
    """Format search/who_knows JSON into concise cited excerpts."""
    items = _extract_items(data)
    if not items:
        return "No results."
    lines = []
    for i, item in enumerate(items, 1):
        if not isinstance(item, dict):
            lines.append(f"{i}. {item}")
            continue
        title = item.get("title") or item.get("heading") or item.get("id") or ""
        excerpt = (item.get("excerpt") or item.get("summary") or item.get("content") or "").strip()
        source = item.get("source") or item.get("source_id") or ""
        cite = f" [source: {source}]" if source else ""
        head = f"{i}. {title}".rstrip()
        lines.append(f"{head}{cite}\n   {excerpt}" if excerpt else head + cite)
    return "\n".join(lines)


def format_answer(data: Any) -> str:
    """Format an /api/ask response: answer text plus citation list."""
    if isinstance(data, dict):
        answer = data.get("answer") or data.get("text") or ""
        citations = data.get("citations") or data.get("sources") or []
        if not answer and citations:
            return format_hits(citations)
        out = answer.strip() or json.dumps(data, indent=2)
        if citations:
            refs = []
            for c in citations:
                if isinstance(c, dict):
                    refs.append(str(c.get("source") or c.get("source_id") or c.get("id") or c))
                else:
                    refs.append(str(c))
            out += "\n\nCitations:\n" + "\n".join(f"- {r}" for r in refs)
        return out
    return str(data)


def _extract_items(data: Any) -> list[Any]:
    if isinstance(data, list):
        return data
    if isinstance(data, dict):
        for key in ("results", "hits", "items", "atoms", "entries", "sources"):
            value = data.get(key)
            if isinstance(value, list):
                return value
    return []
