#!/usr/bin/env bash
set -euo pipefail

# TokSave installer for macOS and Linux
# Usage: curl -fsSL https://raw.githubusercontent.com/agungprasastia/toksave/main/scripts/install.sh | bash

REPO="agungprasastia/toksave"
INSTALL_DIR="${TOKSAVE_INSTALL_DIR:-$HOME/.local/bin}"

# Detect platform
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)  PLATFORM="linux" ;;
  darwin) PLATFORM="darwin" ;;
  *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH="x64" ;;
  aarch64|arm64) ARCH="arm64" ;;
  *)             echo "Unsupported arch: $ARCH"; exit 1 ;;
esac

TARGET="${PLATFORM}-${ARCH}"
ASSET="toksave-${TARGET}.tar.gz"

echo "Installing toksave (${TARGET})..."

# Fetch latest release URL
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

# Download and extract
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

echo "  Downloading ${DOWNLOAD_URL}..."
curl -fsSL "$DOWNLOAD_URL" -o "$TMPDIR/$ASSET"

echo "  Extracting..."
tar xzf "$TMPDIR/$ASSET" -C "$TMPDIR"

# Install
mkdir -p "$INSTALL_DIR"
mv "$TMPDIR/toksave-${TARGET}" "$INSTALL_DIR/toksave"
chmod +x "$INSTALL_DIR/toksave"

echo ""
echo "  ✔ Installed to $INSTALL_DIR/toksave"

# Check PATH
if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
  echo ""
  echo "  ⚠ $INSTALL_DIR is not in your PATH."
  echo "  Add this to your shell profile:"
  echo ""
  echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
  echo ""
fi

echo "  Run 'toksave' to get started."
