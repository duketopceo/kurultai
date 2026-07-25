#!/usr/bin/env bash
# Kurultai installer — cargo-first until GitHub Releases exist.
# Usage: curl -fsSL …/scripts/install.sh | bash
set -euo pipefail

REPO="${KURULTAI_REPO:-duketopceo/kurultai}"
INSTALL_DIR="${KURULTAI_INSTALL_DIR:-$HOME/.local/bin}"
GIT_URL="https://github.com/${REPO}"

need_cargo() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found (needed until binary releases ship)." >&2
    echo "Install Rust: https://rustup.rs  then re-run this installer." >&2
    exit 1
  fi
}

install_via_cargo() {
  need_cargo
  echo "Installing kurultai via cargo (git)…"
  cargo install --git "$GIT_URL" --locked
  echo "Done. Ensure ~/.cargo/bin is on your PATH."
}

mkdir -p "$INSTALL_DIR"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
  darwin-arm64) ASSET_NAME="kurultai-macos-arm64.tar.gz" ;;
  darwin-x86_64) ASSET_NAME="kurultai-macos-amd64.tar.gz" ;;
  linux-x86_64|linux-amd64) ASSET_NAME="kurultai-linux-amd64.tar.gz" ;;
  *)
    install_via_cargo
    exit 0
    ;;
esac

DOWNLOAD_URL=""
if DOWNLOAD_URL="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
  | grep -oE "https://[^\"]+/${ASSET_NAME}" | head -n1)"; then
  :
else
  DOWNLOAD_URL=""
fi

if [ -z "${DOWNLOAD_URL}" ]; then
  # No published release asset yet — expected for pre-v0.1 public builds.
  install_via_cargo
  exit 0
fi

echo "Downloading ${ASSET_NAME}…"
TEMP_TAR="$(mktemp)"
cleanup() { rm -f "$TEMP_TAR"; }
trap cleanup EXIT

curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_TAR"
tar -xzf "$TEMP_TAR" -C "$INSTALL_DIR" kurultai
chmod +x "$INSTALL_DIR/kurultai"

echo "Installed $INSTALL_DIR/kurultai"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Add to PATH: export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
