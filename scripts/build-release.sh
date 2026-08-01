#!/usr/bin/env bash
set -euo pipefail

# Build the release binary for the current platform and package it in the
# same format as CI (.github/workflows/release.yml).
# Cross-platform builds happen in CI.

OUT_DIR="dist"
mkdir -p "$OUT_DIR"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)  PLATFORM="linux" ;;
  darwin) PLATFORM="darwin" ;;
  mingw*|msys*|cygwin) PLATFORM="windows" ;;
  *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH="x64" ;;
  aarch64|arm64) ARCH="arm64" ;;
  *)             echo "Unsupported arch: $ARCH"; exit 1 ;;
esac

TARGET="${PLATFORM}-${ARCH}"
EXT=""
[ "$PLATFORM" = "windows" ] && EXT=".exe"

echo "Building toksave ($TARGET)..."
cargo build --release

if [[ "$PLATFORM" == "windows" ]]; then
  cp "target/release/toksave$EXT" "$OUT_DIR/toksave$EXT"
  7z a "$OUT_DIR/toksave-${TARGET}.zip" "$OUT_DIR/toksave$EXT"
else
  tar czf "$OUT_DIR/toksave-${TARGET}.tar.gz" -C target/release toksave
fi

echo ""
echo "Packaged in $OUT_DIR/"
ls -lh "$OUT_DIR/"
