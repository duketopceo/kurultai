#!/usr/bin/env bash
# Kurultai one-line installer.
# Usage: curl -fsSL https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.sh | bash
#
# Prefers a GitHub Release binary when available for this OS/arch.
# Falls back to: cargo install --git … --tag <latest> --locked
set -euo pipefail

REPO="${KURULTAI_REPO:-duketopceo/kurultai}"
INSTALL_DIR="${KURULTAI_INSTALL_DIR:-$HOME/.local/bin}"
GIT_URL="https://github.com/${REPO}"
API="https://api.github.com/repos/${REPO}/releases/latest"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: required command not found: $1" >&2
    exit 1
  }
}

need_cmd curl
need_cmd tar
need_cmd uname
mkdir -p "$INSTALL_DIR"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
  darwin-arm64) ASSET_NAME="kurultai-macos-arm64.tar.gz" ;;
  darwin-x86_64) ASSET_NAME="kurultai-macos-amd64.tar.gz" ;;
  linux-x86_64|linux-amd64) ASSET_NAME="kurultai-linux-amd64.tar.gz" ;;
  *)
    ASSET_NAME=""
    ;;
esac

RELEASE_JSON=""
if RELEASE_JSON="$(curl -fsSL "$API" 2>/dev/null)"; then
  :
else
  RELEASE_JSON=""
fi

TAG=""
if [ -n "$RELEASE_JSON" ]; then
  TAG="$(printf '%s' "$RELEASE_JSON" | grep -oE '"tag_name":[[:space:]]*"[^"]+"' | head -n1 | sed -E 's/.*"([^"]+)".*/\1/')"
fi

DOWNLOAD_URL=""
if [ -n "$RELEASE_JSON" ] && [ -n "$ASSET_NAME" ]; then
  DOWNLOAD_URL="$(printf '%s' "$RELEASE_JSON" \
    | grep -oE "\"browser_download_url\":[[:space:]]*\"[^\"]+/${ASSET_NAME}\"" \
    | head -n1 \
    | sed -E 's/.*"([^"]+)".*/\1/')"
fi

install_via_cargo() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "error: no release binary for this platform and cargo was not found." >&2
    echo "Install Rust from https://rustup.rs then re-run this installer," >&2
    echo "or: cargo install --git ${GIT_URL} --locked" >&2
    exit 1
  fi
  if [ -n "${TAG}" ]; then
    echo "Installing kurultai ${TAG} via cargo…"
    cargo install --git "$GIT_URL" --tag "$TAG" --locked --force
  else
    echo "Installing kurultai via cargo (main)…"
    cargo install --git "$GIT_URL" --locked --force
  fi
  echo "Done. Ensure ~/.cargo/bin is on your PATH."
  echo "  kurultai --version"
  echo "  kurultai init && kurultai daemon --port 8421"
}

if [ -z "${DOWNLOAD_URL}" ]; then
  if [ -n "$ASSET_NAME" ]; then
    echo "No ${ASSET_NAME} on latest GitHub Release — falling back to cargo."
  else
    echo "No prebuilt binary for ${OS}-${ARCH} — falling back to cargo."
  fi
  install_via_cargo
  exit 0
fi

echo "Downloading ${ASSET_NAME}${TAG:+ (${TAG})}…"
TEMP_TAR="$(mktemp)"
cleanup() { rm -f "$TEMP_TAR"; }
trap cleanup EXIT

curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_TAR"
tar -xzf "$TEMP_TAR" -C "$INSTALL_DIR" kurultai
chmod +x "$INSTALL_DIR/kurultai"

echo "Installed $INSTALL_DIR/kurultai"
"$INSTALL_DIR/kurultai" --version 2>/dev/null || true
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Add to PATH: export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
echo "Next: kurultai init && kurultai daemon --port 8421"
