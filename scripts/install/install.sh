#!/usr/bin/env bash
# Kurultai personal installer (v1 / #72)
# Usage: ./scripts/install/install.sh [flags]
#        curl -fsSL … | bash  is intentionally not the default path — clone first.
set -euo pipefail

REPO_URL_DEFAULT="https://github.com/duketopceo/kurultai.git"
REPO_URL="${KURULTAI_REPO_URL:-$REPO_URL_DEFAULT}"
SRC=""
AGENT="cursor"
DO_INIT=1
DO_BUILD=1
INSTALL_RUST=0
DRY_RUN=0
FORCE_CLONE=0

log() { echo "[kurultai-install] $*" >&2; }
die() { echo "[kurultai-install] ERROR: $*" >&2; exit 1; }

usage() {
  cat <<'EOF'
Kurultai personal installer (Linux / macOS)

Usage:
  ./scripts/install/install.sh [options]

Options:
  --help              Show this help
  --dry-run           Print steps without installing
  --skip-build        Skip cargo install (wire/init only; needs kurultai on PATH)
  --no-init           Skip config + MCP wire (kurultai init)
  --install-rust      Run rustup if cargo is missing (non-interactive)
  --agent <name>      Agent for MCP wire (default: cursor)
  --repo-url <url>    Git clone URL when not running from a checkout
  --src <path>        Explicit source tree (Cargo.toml must exist)
  --clone             Always clone REPO_URL into a temp dir (ignore cwd checkout)

Environment:
  KURULTAI_REPO_URL   Override default clone URL
  CARGO_HOME          Honored by cargo install

After install:
  kurultai index --full
  kurultai search "your query"
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --dry-run) DRY_RUN=1; shift ;;
    --skip-build) DO_BUILD=0; shift ;;
    --no-init) DO_INIT=0; shift ;;
    --install-rust) INSTALL_RUST=1; shift ;;
    --agent)
      [[ $# -ge 2 ]] || die "--agent requires a value"
      AGENT="$2"
      shift 2
      ;;
    --repo-url)
      [[ $# -ge 2 ]] || die "--repo-url requires a value"
      REPO_URL="$2"
      shift 2
      ;;
    --src)
      [[ $# -ge 2 ]] || die "--src requires a value"
      SRC="$2"
      shift 2
      ;;
    --clone) FORCE_CLONE=1; shift ;;
    *) die "unknown argument: $1 (try --help)" ;;
  esac
done

run() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    log "DRY-RUN: $*"
  else
    log "+ $*"
    "$@"
  fi
}

detect_os() {
  local u
  u="$(uname -s 2>/dev/null || echo unknown)"
  case "$u" in
    Darwin) echo "macos" ;;
    Linux) echo "linux" ;;
    *) echo "$u" ;;
  esac
}

ensure_cargo() {
  if command -v cargo >/dev/null 2>&1; then
    log "cargo: $(command -v cargo) ($(cargo --version 2>/dev/null || true))"
    return 0
  fi
  if [[ "$INSTALL_RUST" -eq 1 ]]; then
    log "cargo missing — installing Rust via rustup (local copy, not pipe-to-shell)"
    if [[ "$DRY_RUN" -eq 1 ]]; then
      log "DRY-RUN: download rustup-init, then sh <file> -s -- -y"
      return 0
    fi
    command -v curl >/dev/null 2>&1 || die "curl required to install rustup"
    local rs
    rs="$(mktemp "${TMPDIR:-/tmp}/rustup-init.XXXXXX")"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o "$rs"
    chmod +x "$rs"
    sh "$rs" -s -- -y
    rm -f "$rs"
    # shellcheck disable=SC1091
    source "${CARGO_HOME:-$HOME/.cargo}/env" 2>/dev/null || true
    command -v cargo >/dev/null 2>&1 || die "rustup finished but cargo still not on PATH"
    return 0
  fi
  die "cargo not found. Install Rust (https://rustup.rs) or re-run with --install-rust"
}

resolve_src() {
  if [[ -n "$SRC" ]]; then
    [[ -f "$SRC/Cargo.toml" ]] || die "--src $SRC has no Cargo.toml"
    echo "$(cd "$SRC" && pwd)"
    return 0
  fi
  if [[ "$FORCE_CLONE" -eq 0 ]]; then
    # Walk up from script location and from cwd for a Cargo.toml named kurultai
    local cand
    for cand in \
      "${BASH_SOURCE[0]%/*}/../.." \
      "$PWD" \
      "$PWD/.."
    do
      if [[ -f "$cand/Cargo.toml" ]] && grep -q 'name = "kurultai"' "$cand/Cargo.toml" 2>/dev/null; then
        echo "$(cd "$cand" && pwd)"
        return 0
      fi
    done
  fi
  command -v git >/dev/null 2>&1 || die "git required to clone $REPO_URL"
  local dest
  dest="${TMPDIR:-/tmp}/kurultai-src-$$"
  if [[ "$DRY_RUN" -eq 1 ]]; then
    log "DRY-RUN: git clone --depth 1 $REPO_URL $dest"
    echo "$dest"
    return 0
  fi
  rm -rf "$dest"
  git clone --depth 1 "$REPO_URL" "$dest"
  echo "$dest"
}

main() {
  local os
  os="$(detect_os)"
  log "personal install (os=$os dry_run=$DRY_RUN build=$DO_BUILD init=$DO_INIT agent=$AGENT)"

  case "$os" in
    macos|linux) ;;
    *) die "unsupported OS '$os' — personal installer supports Linux and macOS only" ;;
  esac

  if [[ "$DO_BUILD" -eq 1 ]]; then
    if [[ "$DRY_RUN" -eq 1 ]]; then
      if command -v cargo >/dev/null 2>&1; then
        log "cargo: $(command -v cargo)"
      else
        log "cargo: (missing — would fail without --install-rust on a real run)"
      fi
    else
      ensure_cargo
    fi
  else
    if [[ "$DRY_RUN" -eq 0 ]]; then
      command -v kurultai >/dev/null 2>&1 || die "--skip-build requires kurultai on PATH"
    fi
  fi

  local src
  src="$(resolve_src)"
  log "source: $src"

  if [[ "$DO_BUILD" -eq 1 ]]; then
    if [[ "$DRY_RUN" -eq 1 ]]; then
      log "DRY-RUN: cargo install --path $src --locked --force"
    else
      (
        cd "$src"
        cargo install --path . --locked --force
      )
    fi
  fi

  if [[ "$DO_INIT" -eq 1 ]]; then
    local kbin
    kbin="$(command -v kurultai 2>/dev/null || true)"
    if [[ -z "$kbin" && "$DRY_RUN" -eq 0 ]]; then
      # cargo install may have placed it under CARGO_HOME/bin
      kbin="${CARGO_HOME:-$HOME/.cargo}/bin/kurultai"
    fi
    if [[ "$DRY_RUN" -eq 1 ]]; then
      log "DRY-RUN: kurultai init --agent $AGENT"
    else
      [[ -x "$kbin" ]] || die "kurultai binary not found after install (looked for $kbin)"
      "$kbin" init --agent "$AGENT"
    fi
  else
    log "skipping init (--no-init)"
  fi

  cat <<EOF

✅ Kurultai personal install finished$( [[ "$DRY_RUN" -eq 1 ]] && echo " (dry-run)" ).

Quick start:
  kurultai --help
  kurultai index --full
  kurultai search "your query"

Config: ~/.config/kurultai/config.toml (or XDG_CONFIG_HOME)
MCP:    kurultai init --agent cursor

Need help? https://github.com/duketopceo/kurultai/issues/72
EOF
}

main
