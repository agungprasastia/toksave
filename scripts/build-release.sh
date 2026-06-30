#!/usr/bin/env bash
set -euo pipefail

# Build release binaries for all platforms using bun compile.
# Requires: bun >= 1.1

TARGETS=(
  "bun-linux-x64"
  "bun-linux-arm64"
  "bun-darwin-x64"
  "bun-darwin-arm64"
  "bun-windows-x64"
)

OUT_DIR="dist"
mkdir -p "$OUT_DIR"

for target in "${TARGETS[@]}"; do
  echo "Building $target..."

  if [[ "$target" == *"windows"* ]]; then
    outfile="$OUT_DIR/toksave-${target#bun-}.exe"
  else
    outfile="$OUT_DIR/toksave-${target#bun-}"
  fi

  bun build ./src/index.ts --compile --target="$target" --outfile "$outfile"
  echo "  → $outfile"
done

echo ""
echo "All binaries built in $OUT_DIR/"
ls -lh "$OUT_DIR/"
