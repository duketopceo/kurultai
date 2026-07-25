#!/usr/bin/env bash
# One-line installer (npm-style): curl -fsSL …/install.sh | bash
# Prefers a published GitHub Release binary; falls back to cargo install --git.
set -euo pipefail

REPO="${KURULTAI_REPO:-duketopceo/kurultai}"
INSTALL_DIR="${KURULTAI_INSTALL_DIR:-$HOME/.local/bin}"

mkdir -p "$INSTALL_DIR"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
  darwin-arm64) ASSET_NAME="kurultai-macos-arm64.tar.gz" ;;
  darwin-x86_64) ASSET_NAME="kurultai-macos-amd64.tar.gz" ;;
  linux-x86_64|linux-amd64) ASSET_NAME="kurultai-linux-amd64.tar.gz" ;;
  *)
    echo "No prebuilt asset for $OS/$ARCH — trying cargo install --git…"
    cargo install --git "https://github.com/${REPO}" --locked
    echo "Installed via cargo. Ensure ~/.cargo/bin is on PATH."
    exit 0
    ;;
esac

echo "Fetching latest Kurultai release for $OS ($ARCH)…"
RELEASE_URL="https://api.github.com/repos/${REPO}/releases/latest"
DOWNLOAD_URL="$(curl -fsSL "$RELEASE_URL" | grep -oE "https://[^\"]+/${ASSET_NAME}" | head -n1 || true)"

if [ -z "${DOWNLOAD_URL:-}" ]; then
  echo "No release asset yet — falling back to: cargo install --git https://github.com/${REPO} --locked"
  cargo install --git "https://github.com/${REPO}" --locked
  echo "Installed via cargo. Ensure ~/.cargo/bin is on PATH."
  exit 0
fi

TEMP_TAR="$(mktemp)"
cleanup() { rm -f "$TEMP_TAR"; }
trap cleanup EXIT

curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_TAR"
tar -xzf "$TEMP_TAR" -C "$INSTALL_DIR" kurultai
chmod +x "$INSTALL_DIR/kurultai"

echo "Kurultai installed to $INSTALL_DIR/kurultai"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Add to PATH: export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
