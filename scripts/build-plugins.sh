#!/usr/bin/env bash
set -euo pipefail

# Cross-platform plugin build script for LAO
# Builds all plugins in-place (artifacts stay in each plugin's target/release/)
# The plugin registry discovers them automatically from target/release/

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PLUGIN_DIR="$ROOT_DIR/plugins"

# Detect current platform
OS=$(uname -s 2>/dev/null || echo "Unknown")
case "$OS" in
    Linux*)   PLATFORM="linux"; EXT="so" ;;
    Darwin*)  PLATFORM="macos"; EXT="dylib" ;;
    MINGW*|MSYS*|CYGWIN*) PLATFORM="windows"; EXT="dll" ;;
    *)        PLATFORM="unknown"; EXT="so" ;;
esac

echo "Building plugins for platform: $PLATFORM"
echo "Plugin directory: $PLUGIN_DIR"
echo ""

# Find all plugin directories with Cargo.toml
plugin_dirs=()
for dir in "$PLUGIN_DIR"/*/; do
    if [ -f "$dir/Cargo.toml" ]; then
        plugin_dirs+=("$dir")
    fi
done

if [ ${#plugin_dirs[@]} -eq 0 ]; then
    echo "No plugin directories found in $PLUGIN_DIR"
    exit 1
fi

echo "Found ${#plugin_dirs[@]} plugins:"
for dir in "${plugin_dirs[@]}"; do
    echo "  - $(basename "$dir")"
done
echo ""

# Build each plugin
failed_plugins=()
for plugin_dir in "${plugin_dirs[@]}"; do
    plugin_name=$(basename "$plugin_dir")
    printf "  %-30s" "$plugin_name"

    # Force a per-plugin target dir so the artifact lands in
    # plugins/<name>/target/release/, where the registry discovers it.
    # (In a workspace, a bare `cargo build` would write to the root target/.)
    if (cd "$plugin_dir" && cargo build --release --locked --target-dir target 2>/dev/null); then
        echo "OK"
    else
        echo "FAILED"
        failed_plugins+=("$plugin_name")
    fi
done

echo ""

# Report results
if [ ${#failed_plugins[@]} -eq 0 ]; then
    echo "All ${#plugin_dirs[@]} plugins built successfully."
else
    echo "Failed to build ${#failed_plugins[@]} plugins:"
    for plugin in "${failed_plugins[@]}"; do
        echo "  - $plugin"
    done
    exit 1
fi

# List built artifacts
echo ""
echo "Built artifacts:"
for dir in "${plugin_dirs[@]}"; do
    plugin_name=$(basename "$dir")
    artifact=$(find "$dir/target/release" -maxdepth 1 -name "lib*.$EXT" 2>/dev/null | head -1)
    if [ -n "$artifact" ]; then
        size=$(du -h "$artifact" | cut -f1)
        printf "  %-30s %s (%s)\n" "$plugin_name" "$(basename "$artifact")" "$size"
    fi
done
