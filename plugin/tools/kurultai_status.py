"""kurultai_status tool — report Kurultai brain health and plugin wiring.

Shows binary resolution, daemon reachability, kurultai's own `status`
output, and (when configured) whether the expected SQLite store exists.
"""
from __future__ import annotations

import os

from helpers.tool import Response, Tool

from usr.plugins.kurultai.helpers.client import (
    call,
    find_binary,
    http_available,
)
from usr.plugins.kurultai.helpers.config import resolve_config


class KurultaiStatus(Tool):
    """Report Kurultai binary/daemon/store status for diagnostics."""

    async def execute(self, **kwargs) -> Response:
        cfg = resolve_config(agent=self.agent)
        lines = ["Kurultai status"]

        binary = find_binary(cfg)
        lines.append(
            f"- binary: {binary}" if binary else "- binary: NOT FOUND (set binary_path in plugin settings)"
        )

        if cfg.get("server_url"):
            reachable = await http_available(cfg)
            lines.append(
                f"- daemon: {cfg['server_url']} ({'reachable' if reachable else 'unreachable'})"
            )
        else:
            lines.append("- daemon: not configured (server_url empty)")

        db_path = cfg.get("db_path")
        if db_path:
            expanded = os.path.expanduser(db_path)
            exists = os.path.isfile(expanded)
            lines.append(
                f"- store db: {expanded} ({'present' if exists else 'missing — run `kurultai index --full`'})"
            )
        else:
            lines.append("- store db: path not configured (db_path empty)")

        result = await call(
            cfg,
            operation="status",
            cli_args=["status"],
            http_method="GET",
            http_path="/api/status",
        )
        lines.append("")
        lines.append(result.text if result.ok else f"kurultai status unavailable: {result.text}")
        return Response(message="\n".join(lines), break_loop=False)
