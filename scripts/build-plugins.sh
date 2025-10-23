#!/usr/bin/env bash
set -euo pipefail

# Cross-platform plugin build script for LAO
# Builds all plugins for the current platform

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PLUGIN_DIR="$ROOT_DIR/plugins"
TARGET_DIR="$ROOT_DIR/target"

# Detect current platform
OS=$(uname -s 2>/dev/null || echo "Unknown")
case "$OS" in
    Linux*)   PLATFORM="linux" ;;
    Darwin*)  PLATFORM="macos" ;;
    MINGW*|MSYS*|CYGWIN*) PLATFORM="windows" ;;
    *)        PLATFORM="unknown" ;;
esac

echo "🔧 Building plugins for platform: $PLATFORM"
echo "📁 Root directory: $ROOT_DIR"
echo "📁 Plugin directory: $PLUGIN_DIR"

# Function to build a single plugin
build_plugin() {
    local plugin_dir="$1"
    local plugin_name=$(basename "$plugin_dir")
    
    if [ ! -f "$plugin_dir/Cargo.toml" ]; then
        echo "⚠️  Skipping $plugin_name (no Cargo.toml)"
        return 0
    fi
    
    echo "🔨 Building $plugin_name..."
    cd "$plugin_dir"
    
    # Build the plugin
    if cargo build --release; then
        echo "✅ $plugin_name built successfully"
    else
        echo "❌ Failed to build $plugin_name"
        return 1
    fi
    
    cd "$ROOT_DIR"
}

# Function to copy built plugins to plugins directory
copy_plugins() {
    echo "📋 Copying built plugins to plugins/ directory..."
    
    # Use the existing dll-puller script
    if [ -f "$ROOT_DIR/tools/dll-puller.sh" ]; then
        bash "$ROOT_DIR/tools/dll-puller.sh"
    else
        echo "⚠️  dll-puller.sh not found, manually copying plugins..."
        
        # Manual copy based on platform
        case "$PLATFORM" in
            "linux")
                find "$TARGET_DIR/release" -name "lib*plugin*.so" -exec cp {} "$PLUGIN_DIR/" \;
                ;;
            "macos")
                find "$TARGET_DIR/release" -name "lib*plugin*.dylib" -exec cp {} "$PLUGIN_DIR/" \;
                ;;
            "windows")
                find "$TARGET_DIR/release" -name "*plugin*.dll" -exec cp {} "$PLUGIN_DIR/" \;
                ;;
        esac
    fi
}

# Main build process
main() {
    echo "🚀 Starting cross-platform plugin build..."
    
    # Create plugins directory if it doesn't exist
    mkdir -p "$PLUGIN_DIR"
    
    # Find all plugin directories
    local plugin_dirs=()
    while IFS= read -r -d '' dir; do
        plugin_dirs+=("$dir")
    done < <(find "$PLUGIN_DIR" -maxdepth 1 -type d -name "*Plugin" -print0)
    
    if [ ${#plugin_dirs[@]} -eq 0 ]; then
        echo "⚠️  No plugin directories found in $PLUGIN_DIR"
        return 1
    fi
    
    echo "📦 Found ${#plugin_dirs[@]} plugin directories:"
    for dir in "${plugin_dirs[@]}"; do
        echo "  - $(basename "$dir")"
    done
    
    # Build each plugin
    local failed_plugins=()
    for plugin_dir in "${plugin_dirs[@]}"; do
        if ! build_plugin "$plugin_dir"; then
            failed_plugins+=("$(basename "$plugin_dir")")
        fi
    done
    
    # Report build results
    if [ ${#failed_plugins[@]} -eq 0 ]; then
        echo "✅ All plugins built successfully!"
    else
        echo "❌ Failed to build ${#failed_plugins[@]} plugins:"
        for plugin in "${failed_plugins[@]}"; do
            echo "  - $plugin"
        done
        return 1
    fi
    
    # Copy built plugins
    copy_plugins
    
    echo "🎉 Cross-platform plugin build complete!"
    echo "📊 Platform: $PLATFORM"
    echo "📁 Plugins directory: $PLUGIN_DIR"
    
    # List final plugins
    echo "📋 Built plugins:"
    case "$PLATFORM" in
        "linux")
            ls -la "$PLUGIN_DIR"/*.so 2>/dev/null || echo "  No .so files found"
            ;;
        "macos")
            ls -la "$PLUGIN_DIR"/*.dylib 2>/dev/null || echo "  No .dylib files found"
            ;;
        "windows")
            ls -la "$PLUGIN_DIR"/*.dll 2>/dev/null || echo "  No .dll files found"
            ;;
    esac
}

# Run main function
main "$@"
