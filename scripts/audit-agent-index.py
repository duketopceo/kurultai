#!/usr/bin/env python3
"""Fail if the hierarchical agent INDEX.md map drifts from git-tracked files."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SKIP_DIR_PREFIXES = (
    "ui/assets/",
    "tests/fixtures/vault/",
    "tests/fixtures/code_repo/",
)

SKIP_DIR_NAMES = {
    ".git",
    "target",
    "node_modules",
    ".next",
    "dist",
    ".harness",
    ".cursor",
}

INDEX_NAME = "INDEX.md"
SCHEMA = "index: kurultai/v1"


def git_tracked() -> list[str]:
    out = subprocess.check_output(
        ["git", "ls-files", "-z"], cwd=ROOT, text=True
    )
    return [p for p in out.split("\0") if p]


def under_skip_dir(path: str) -> bool:
    parts = Path(path).parts
    if any(p in SKIP_DIR_NAMES for p in parts):
        return True
    posix = path.replace("\\", "/")
    if posix == "ui/assets" or posix.startswith("ui/assets/"):
        return True
    for prefix in SKIP_DIR_PREFIXES:
        if posix.startswith(prefix) or posix + "/" == prefix:
            return True
    return False


def parent_posix(path: str) -> str:
    p = Path(path)
    parent = p.parent.as_posix()
    return "" if parent == "." else parent


def nearest_in_scope_dir(file_path: str, in_scope_dirs: set[str]) -> str:
    """Folder whose INDEX.md must mention this skip-interior file."""
    cur = parent_posix(file_path)
    while True:
        if cur in in_scope_dirs:
            return cur
        if not cur:
            return ""
        parent = str(Path(cur).parent)
        cur = "" if parent == "." else parent.replace("\\", "/")


def load_index(dir_key: str) -> str:
    path = ROOT / INDEX_NAME if dir_key == "" else ROOT / dir_key / INDEX_NAME
    if not path.is_file():
        return ""
    return path.read_text(encoding="utf-8")


def main() -> int:
    files = git_tracked()
    errors: list[str] = []

    all_dirs: set[str] = {""}
    for f in files:
        p = parent_posix(f)
        while True:
            all_dirs.add(p)
            if not p:
                break
            parent = str(Path(p).parent)
            p = "" if parent == "." else parent.replace("\\", "/")

    in_scope_dirs = {d for d in all_dirs if d == "" or not under_skip_dir(d + "/")}

    for d in sorted(in_scope_dirs, key=lambda x: x.split("/")):
        text = load_index(d)
        loc = INDEX_NAME if d == "" else f"{d}/{INDEX_NAME}"
        if not text:
            errors.append(f"missing {loc}")
            continue
        if SCHEMA not in text[:1200]:
            errors.append(f"{loc}: missing `{SCHEMA}` frontmatter")

    # Files in in-scope dirs must be named in that dir's INDEX.md
    for f in files:
        if f.endswith("/" + INDEX_NAME) or f == INDEX_NAME:
            continue
        if under_skip_dir(f):
            owner = nearest_in_scope_dir(f, in_scope_dirs)
            text = load_index(owner)
            loc = INDEX_NAME if owner == "" else f"{owner}/{INDEX_NAME}"
            # hashed assets: require the assets/ prefix mention
            posix = f.replace("\\", "/")
            if posix.startswith("ui/assets/"):
                if "assets/" not in text and "assets/*" not in text:
                    errors.append(f"{loc}: does not catalog ui/assets/ (need `assets/`)")
                continue
            needle = Path(f).name
            rel = posix[len(owner) + 1 :] if owner else posix
            if needle not in text and rel not in text:
                errors.append(f"{loc}: missing skip-interior file `{rel}`")
            continue

        d = parent_posix(f)
        text = load_index(d)
        loc = INDEX_NAME if d == "" else f"{d}/{INDEX_NAME}"
        name = Path(f).name
        if name not in text:
            errors.append(f"{loc}: missing file `{name}`")

    # Child in-scope dirs must be mentioned on the parent
    children: dict[str, set[str]] = {}
    for d in in_scope_dirs:
        if d == "":
            continue
        parent = parent_posix(d)
        children.setdefault(parent, set()).add(Path(d).name)

    for parent, kids in children.items():
        text = load_index(parent)
        loc = INDEX_NAME if parent == "" else f"{parent}/{INDEX_NAME}"
        for kid in sorted(kids):
            if kid not in text:
                errors.append(f"{loc}: missing child folder `{kid}/`")

    # Skip interiors should still be named on the nearest in-scope parent
    if "assets/" not in load_index("ui") and "assets/*" not in load_index("ui"):
        errors.append("ui/INDEX.md: missing generated `assets/` catalog")
    fixtures = load_index("tests/fixtures")
    if "vault" not in fixtures:
        errors.append("tests/fixtures/INDEX.md: missing `vault` catalog")
    if "code_repo" not in fixtures:
        errors.append("tests/fixtures/INDEX.md: missing `code_repo` catalog")

    if errors:
        # unique, stable
        seen: set[str] = set()
        uniq: list[str] = []
        for e in errors:
            if e not in seen:
                seen.add(e)
                uniq.append(e)
        print(f"agent-index audit failed ({len(uniq)}):", file=sys.stderr)
        for e in uniq:
            print(f"  {e}", file=sys.stderr)
        return 1

    print(
        f"agent-index audit ok: {len(in_scope_dirs)} folders, {len(files)} tracked files"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
