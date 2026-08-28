"""kurultai_ask tool — synthesized answer from the Kurultai brain.

Higher cost than kurultai_search: runs synthesis across atoms and returns
an answer with citations. Falls back to extractive answers when no LLM
API key is configured for kurultai.
"""
from __future__ import annotations

from helpers.tool import Response, Tool

from usr.plugins.kurultai.helpers.client import call, format_answer
from usr.plugins.kurultai.helpers.config import resolve_config


class KurultaiAsk(Tool):
    """Ask the Kurultai brain a question and get a cited, synthesized answer."""

    async def execute(self, **kwargs) -> Response:
        question = (self.args.get("question") or "").strip()
        if not question:
            return Response(
                message="kurultai_ask requires a 'question' argument.", break_loop=False
            )
        cfg = resolve_config(agent=self.agent)
        result = await call(
            cfg,
            operation="ask",
            cli_args=["ask", question],
            http_method="POST",
            http_path="/api/ask",
            http_payload={"question": question},
            formatter=format_answer,
        )
        if not result.ok:
            return Response(message=result.text, break_loop=False)
        header = f"Kurultai ask ({result.transport}): {question}"
        return Response(message=f"{header}\n\n{result.text}", break_loop=False)
