"""Security validation for Kurultai plugin.

Validates that server_url points to localhost or a private network address.
Sanitizes content before storing in the brain.
"""
from __future__ import annotations

import ipaddress
import re
from urllib.parse import urlparse

_ALLOWED_HOSTS = {"localhost", "127.0.0.1", "0.0.0.0", "::1"}
_PRIVATE_PREFIXES = ("10.", "172.16.", "172.17.", "172.18.", "172.19.", "172.20.",
                     "172.21.", "172.22.", "172.23.", "172.24.", "172.25.", "172.26.",
                     "172.27.", "172.28.", "172.29.", "172.30.", "172.31.", "192.168.")

_SECRET_PATTERNS = [
    re.compile(r'(?:sk-|pk-|Bearer\s+)[A-Za-z0-9_\-]{20,}', re.I),
    re.compile(r'(?:API_KEY|TOKEN|SECRET|PASSWORD)\s*[=:]\s*["\']?[A-Za-z0-9_\-]{8,}', re.I),
]


def is_safe_server_url(url: str) -> bool:
    """True when the URL host is localhost or a private/internal address."""
    if not url:
        return True  # empty = CLI-only mode, no daemon
    try:
        parsed = urlparse(url)
        host = parsed.hostname or ""
        if host in _ALLOWED_HOSTS:
            return True
        if host.startswith(_PRIVATE_PREFIXES):
            return True
        try:
            ip = ipaddress.ip_address(host)
            return ip.is_private or ip.is_loopback
        except ValueError:
            pass
        return False
    except Exception:
        return False


def sanitize_content(text: str, max_length: int = 500) -> str:
    """Strip secret-like substrings and truncate."""
    for pattern in _SECRET_PATTERNS:
        text = pattern.sub('[REDACTED]', text)
    return text.strip()[:max_length]
