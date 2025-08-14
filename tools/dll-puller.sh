#!/usr/bin/env bash
set -euo pipefail

_uname=$(uname -s 2>/dev/null || echo "")
case "$_uname" in
  Linux*)  EXT="so";   PREFIX="lib" ;;
  Darwin*) EXT="dylib"; PREFIX="lib" ;;
  MINGW*|MSYS*|CYGWIN*) EXT="dll"; PREFIX="" ;;
  *)       EXT="so";   PREFIX="lib" ;;
esac

# Copies plugin shared libraries into the plugins/ directory (OS-aware: $PREFIX*.$EXT).

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
RELEASE_DIR="${1:-$ROOT_DIR/target/release}"
PLUGIN_DIR="$ROOT_DIR/plugins"

echo "Release dir: $RELEASE_DIR"
echo "Plugin dir:  $PLUGIN_DIR"

mkdir -p "$PLUGIN_DIR"

# Remove old shared libraries for this OS
find "$PLUGIN_DIR" -maxdepth 1 -type f -name "*.${EXT}" -print -delete || true

shopt -s nullglob
COPIED=0
# First: copy by crate names discovered in plugins/*/Cargo.toml
while IFS= read -r cargo; do
  name=$(grep -m1 '^name\s*=\s*"' "$cargo" | sed -E 's/.*"([^"]+)".*/\1/')
  [ -n "$name" ] || continue
  artifact="${RELEASE_DIR}/${PREFIX}${name}.${EXT}"
  if [ -f "$artifact" ]; then
    cp -f "$artifact" "$PLUGIN_DIR/"
    echo "Copied $(basename "$artifact")"
    COPIED=$((COPIED+1))
  fi
done < <(find "$ROOT_DIR/plugins" -maxdepth 2 -type f -name Cargo.toml)

# Fallback: copy any *plugin*.$EXT artifacts
for f in "$RELEASE_DIR"/*plugin*.${EXT}; do
  [ -e "$f" ] || continue
  cp -f "$f" "$PLUGIN_DIR/"
  echo "Copied $(basename "$f")"
  COPIED=$((COPIED+1))
done

echo "Done. Copied $COPIED artifacts to plugins/."


