"""Passive auto-memory: store distilled facts into Kurultai after each monologue.

Write path: the kurultai binary's MCP stdio `remember` tool (structured:
title/summary/tags). The daemon HTTP surface is read-only by policy, so
writes never go through /api/recall — that endpoint is a retrieval query.

Fire-and-forget, non-blocking, sanitized. Only stores when:
- auto_memory is enabled in plugin config
- the last AI response contains substantive content (not just tool calls)
- content passes sanitization (no secrets, API keys, or raw file dumps)

Never blocks the agent loop. Never raises. All errors are swallowed.
"""
from __future__ import annotations

import asyncio
import re
import time
from typing import Any

from helpers.extension import Extension
from usr.plugins.kurultai.helpers.config import resolve_config
from usr.plugins.kurultai.helpers.mcp_client import mcp_remember
from usr.plugins.kurultai.helpers.security import sanitize_content

_MAX_SUMMARY = 500
_MIN_CONTENT = 20
_RECENT: dict[str, float] = {}
_DEDUP_WINDOW = 300

_STOPWORDS = {
    "the", "and", "for", "with", "that", "this", "from", "into", "your", "have",
    "will", "been", "when", "then", "than", "them", "they", "what", "which",
    "were", "are", "was", "not", "but", "all", "can", "has", "had", "out",
    "its", "also", "use", "using", "used", "after", "before", "about", "there",
    "their", "would", "could", "should", "more", "some", "only", "over", "just",
    "like", "make", "made", "need", "want", "agent", "zero",
}


def _derive_tags(summary: str) -> list[str]:
    """Derive topic keywords so the atom never lands untagged."""
    words = re.findall(r"[a-z][a-z0-9_-]{3,}", summary.lower())
    counts: dict[str, int] = {}
    for word in words:
        if word in _STOPWORDS:
            continue
        counts[word] = counts.get(word, 0) + 1
    top = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))[:3]
    return ["agent-zero", "auto"] + [word for word, _ in top]


def _extract_summary(agent) -> str | None:
    try:
        history = getattr(agent, 'history', None)
        if not history:
            return None
        topics = getattr(history, 'topics', []) or []
        current = getattr(history, 'current', []) or []
        all_msgs = list(current)
        for topic in topics:
            topic_msgs = getattr(topic, 'messages', []) or []
            all_msgs.extend(topic_msgs)
        for msg in reversed(all_msgs):
            role = getattr(msg, 'role', '') or ''
            if role.lower() not in ('assistant', 'ai'):
                continue
            content = getattr(msg, 'content', '') or ''
            if isinstance(content, list):
                parts = []
                for block in content:
                    if isinstance(block, dict) and block.get('type') == 'text':
                        parts.append(block.get('text', ''))
                content = ' '.join(parts)
            if isinstance(content, str) and len(content.strip()) >= _MIN_CONTENT:
                return content.strip()
        return None
    except Exception:
        return None


def _dedup_key(agent, summary: str) -> str:
    ctx_id = getattr(getattr(agent, 'context', None), 'id', 'unknown')
    return f"{ctx_id}:{summary[:80]}"


def _should_skip(agent, summary: str) -> bool:
    key = _dedup_key(agent, summary)
    now = time.time()
    last = _RECENT.get(key, 0)
    if now - last < _DEDUP_WINDOW:
        return True
    _RECENT[key] = now
    return False


class AutoRemember(Extension):

    async def execute(self, loop_data=None, **kwargs) -> None:
        try:
            agent = getattr(self, 'agent', None)
            if not agent:
                return
            cfg = resolve_config(agent=agent)
            if not cfg.get('auto_memory', False):
                return
            summary = _extract_summary(agent)
            if not summary:
                return
            summary = sanitize_content(summary, _MAX_SUMMARY)
            if len(summary) < int(cfg.get('auto_memory_min_length', _MIN_CONTENT)):
                return
            if _should_skip(agent, summary):
                return
            asyncio.create_task(_store_fact(cfg, agent, summary))
        except Exception:
            pass


async def _store_fact(cfg: dict[str, Any], agent: Any, summary: str) -> None:
    try:
        ctx_id = str(getattr(getattr(agent, 'context', None), 'id', 'unknown'))
        title = sanitize_content(f"Agent chat {ctx_id[:8]}: {summary[:60]}", 120)
        tags = _derive_tags(summary)
        await mcp_remember(cfg, title, summary, tags, agent_id="agent-zero-auto")
    except Exception:
        pass
