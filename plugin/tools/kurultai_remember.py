"""kurultai_remember tool — store a distilled, structured fact in the brain.

Write path: the kurultai binary's MCP stdio `remember` tool (the daemon HTTP
surface is read-only by policy). Structure is enforced here: short title,
distilled summary (<=500 chars, secrets redacted), 1-8 explicit tags.
Never dump raw transcripts or whole files — unstructured dumps land in
quarantine and pollute the brain.
"""
from __future__ import annotations

from helpers.tool import Response, Tool

from usr.plugins.kurultai.helpers.client import find_binary
from usr.plugins.kurultai.helpers.config import resolve_config
from usr.plugins.kurultai.helpers.mcp_client import mcp_remember
from usr.plugins.kurultai.helpers.security import sanitize_content

_MAX_TAGS = 8
_MAX_TAG_LEN = 40


class KurultaiRemember(Tool):
    """Store a distilled fact (title + summary + tags) into Kurultai."""

    async def execute(self, **kwargs) -> Response:
        title = sanitize_content(str(self.args.get("title") or ""), 120)
        summary = sanitize_content(str(self.args.get("summary") or ""), 500)
        raw_tags = self.args.get("tags") or []
        if isinstance(raw_tags, str):
            raw_tags = raw_tags.split(",")
        tags: list[str] = []
        for tag in raw_tags:
            clean = str(tag).strip().lstrip("#").lower()[:_MAX_TAG_LEN]
            if clean and clean not in tags:
                tags.append(clean)
        tags = tags[:_MAX_TAGS]
        project = str(self.args.get("project") or "").strip() or None

        if not title or not summary:
            return Response(
                message="kurultai_remember requires 'title' and 'summary'. Store "
                        "distilled facts only — no raw transcripts or file dumps.",
                break_loop=False,
            )
        if not tags:
            return Response(
                message="kurultai_remember requires 'tags' (comma-separated, at "
                        "least one) — untagged atoms are quarantined by the brain's "
                        "write policy.",
                break_loop=False,
            )

        cfg = resolve_config(agent=self.agent)
        if not find_binary(cfg):
            return Response(
                message="kurultai binary not found — set binary_path in the "
                        "Kurultai plugin settings.",
                break_loop=False,
            )

        result = await mcp_remember(
            cfg, title, summary, tags, project=project, agent_id="agent-zero"
        )
        if not result.get("ok"):
            return Response(
                message=f"Kurultai remember failed: {result.get('lane')} — "
                        f"{result.get('raw', '')}",
                break_loop=False,
            )

        lane = str(result.get("lane") or "")
        lines = [
            f"Kurultai remember: stored '{title}'",
            f"- atom_id: {result.get('atom_id')}",
            f"- lane: {lane}",
            f"- tags: {', '.join(tags)}" + (f" | project: {project}" if project else ""),
        ]
        if lane.startswith("quarantined"):
            lines.append(
                "Note: the atom was quarantined by the write policy — improve the "
                "structure/tags and re-store, or promote it explicitly with "
                "`kurultai promote <atom_id>` after review."
            )
        return Response(message="\n".join(lines), break_loop=False)
