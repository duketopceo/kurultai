"""kurultai_who_knows tool — which sources know about a topic.

Scopes a topic before deep retrieval: reports which indexed sources
(notes, chats, code checkouts, ...) contain knowledge about it.
"""
from __future__ import annotations

from helpers.tool import Response, Tool

from usr.plugins.kurultai.helpers.client import call, format_hits
from usr.plugins.kurultai.helpers.config import resolve_config


class KurultaiWhoKnows(Tool):
    """Report which Kurultai sources know about a topic."""

    async def execute(self, **kwargs) -> Response:
        topic = (self.args.get("topic") or "").strip()
        if not topic:
            return Response(
                message="kurultai_who_knows requires a 'topic' argument.", break_loop=False
            )
        cfg = resolve_config(agent=self.agent)
        limit = self.args.get("limit") or cfg["default_limit"]
        try:
            limit = max(1, min(50, int(limit)))
        except (TypeError, ValueError):
            limit = cfg["default_limit"]

        result = await call(
            cfg,
            operation="who-knows",
            cli_args=["who-knows", topic, "--limit", str(limit)],
            http_method="POST",
            http_path="/who_knows",
            http_payload={"topic": topic, "limit": limit},
            formatter=format_hits,
        )
        if not result.ok:
            return Response(message=result.text, break_loop=False)
        header = f"Kurultai who-knows ({result.transport}) for: {topic}"
        return Response(message=f"{header}\n\n{result.text}", break_loop=False)
