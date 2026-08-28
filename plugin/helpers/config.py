"""Kurultai plugin configuration resolution.

Merges plugin defaults with user settings stored via the plugin settings
modal (Settings > External > Kurultai). Never touches API keys.
"""
from __future__ import annotations

import os
from typing import Any

from helpers.plugins import get_plugin_config

PLUGIN_NAME = "kurultai"

DEFAULTS: dict[str, Any] = {
    "binary_path": "",
    "server_url": "",
    "prefer_http": True,
    "config_path": "",
    "kurultai_env": "",
    "db_path": "",
    "default_limit": 8,
    "timeout_secs": 60,
    "auto_memory": False,
    "auto_memory_min_length": 20,
}


def resolve_config(agent=None) -> dict[str, Any]:
    """Return the effective kurultai plugin configuration.

    User-saved settings override DEFAULTS. Values are coerced to their
    expected types; empty strings stay empty (meaning "unset").
    """
    try:
        user_cfg = get_plugin_config(PLUGIN_NAME, agent=agent) or {}
    except Exception:
        user_cfg = {}

    cfg = dict(DEFAULTS)
    for key, value in user_cfg.items():
        if key in cfg and value is not None:
            cfg[key] = value

    # Type coercion / sanitisation
    cfg["prefer_http"] = _coerce_bool(cfg["prefer_http"])
    cfg["auto_memory"] = _coerce_bool(cfg["auto_memory"])
    cfg["auto_memory_min_length"] = _coerce_int(cfg["auto_memory_min_length"], DEFAULTS["auto_memory_min_length"])
    cfg["default_limit"] = _coerce_int(cfg["default_limit"], DEFAULTS["default_limit"])
    cfg["timeout_secs"] = max(5, _coerce_int(cfg["timeout_secs"], DEFAULTS["timeout_secs"]))
    for key in ("binary_path", "server_url", "config_path", "kurultai_env", "db_path"):
        cfg[key] = str(cfg.get(key) or "").strip()
    cfg["server_url"] = cfg["server_url"].rstrip("/")
    return cfg


def build_env(cfg: dict[str, Any]) -> dict[str, str]:
    """Environment for kurultai CLI subprocesses.

    Inherits the parent environment verbatim (so OPENROUTER_API_KEY flows
    through untouched when the operator provides it) and applies the
    plugin's config path / env overrides. The plugin itself never reads,
    stores, or logs any API key.
    """
    env = dict(os.environ)
    if cfg.get("config_path"):
        env["KURULTAI_CONFIG"] = cfg["config_path"]
    if cfg.get("kurultai_env"):
        env["KURULTAI_ENV"] = cfg["kurultai_env"]
    return env


def _coerce_bool(value: Any) -> bool:
    if isinstance(value, bool):
        return value
    if isinstance(value, (int, float)):
        return bool(value)
    if isinstance(value, str):
        return value.strip().lower() in {"1", "true", "yes", "on"}
    return False


def _coerce_int(value: Any, fallback: int) -> int:
    try:
        return int(value)
    except (TypeError, ValueError):
        return fallback
