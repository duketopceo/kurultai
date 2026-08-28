"""kurultai_search tool — query the local Kurultai knowledge brain.

Returns token-capped excerpts with citations from the user's indexed
notes, chats, and code checkouts. FTS-first: works without an API key.
"""
from __future__ import annotations

from helpers.tool import Response, Tool

from usr.plugins.kurultai.helpers.client import call, format_hits
from usr.plugins.kurultai.helpers.config import resolve_config


class KurultaiSearch(Tool):
    """Search the Kurultai brain for grounded excerpts and citations."""

    async def execute(self, **kwargs) -> Response:
        query = (self.args.get("query") or "").strip()
        if not query:
            return Response(
                message="kurultai_search requires a 'query' argument.", break_loop=False
            )
        cfg = resolve_config(agent=self.agent)
        limit = self.args.get("limit") or cfg["default_limit"]
        try:
            limit = max(1, min(50, int(limit)))
        except (TypeError, ValueError):
            limit = cfg["default_limit"]

        result = await call(
            cfg,
            operation="search",
            cli_args=["search", query, "--limit", str(limit)],
            http_method="POST",
            http_path="/api/search",
            http_payload={"query": query, "limit": limit},
            formatter=format_hits,
        )
        if not result.ok:
            return Response(message=result.text, break_loop=False)
        header = f"Kurultai search ({result.transport}) for: {query}"
        return Response(message=f"{header}\n\n{result.text}", break_loop=False)
