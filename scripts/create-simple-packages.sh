#!/usr/bin/env bash
set -euo pipefail

# Simple package creation script for LAO
# Creates basic packages for testing

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT_DIR/target"
DIST_DIR="$ROOT_DIR/dist"
VERSION="0.1.0"

echo "📦 Creating simple packages for LAO v$VERSION"

# Create distribution directory
mkdir -p "$DIST_DIR"

# Detect platform
OS=$(uname -s 2>/dev/null || echo "Unknown")
case "$OS" in
    Linux*)   PLATFORM="linux" ;;
    Darwin*)  PLATFORM="macos" ;;
    MINGW*|MSYS*|CYGWIN*) PLATFORM="windows" ;;
    *)        PLATFORM="unknown" ;;
esac

echo "🔧 Platform: $PLATFORM"

# Build release binaries
echo "🔨 Building release binaries..."
cargo build --release --bin lao-cli
cargo build --release --bin lao-ui

# Build plugins
echo "🔌 Building plugins..."
bash scripts/build-plugins.sh

# Create platform-specific packages
case "$PLATFORM" in
    "macos")
        echo "🍎 Creating macOS package..."
        
        # Create tar.gz archive
        archive_dir="$DIST_DIR/tar"
        package_dir="$archive_dir/lao-$VERSION-macos"
        mkdir -p "$package_dir"
        
        # Copy binaries
        cp "$TARGET_DIR/release/lao-cli" "$package_dir/"
        cp "$TARGET_DIR/release/lao-ui" "$package_dir/"
        
        # Copy plugins
        mkdir -p "$package_dir/plugins"
        cp plugins/*.dylib "$package_dir/plugins/" 2>/dev/null || true
        
        # Copy documentation
        cp README.md "$package_dir/" 2>/dev/null || true
        
        # Create install script
        cat > "$package_dir/install.sh" << 'EOF'
#!/bin/bash
set -e

echo "Installing LAO Orchestrator by Jake Abendroth on macOS..."

# Create directories
sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/local/lib/lao/plugins

# Install binaries
sudo cp lao-cli /usr/local/bin/
sudo cp lao-ui /usr/local/bin/
sudo chmod +x /usr/local/bin/lao-*

# Install plugins
sudo cp plugins/*.dylib /usr/local/lib/lao/plugins/ 2>/dev/null || true

echo "✅ LAO installed successfully!"
echo "Run 'lao-ui' to start the GUI or 'lao-cli --help' for CLI options"
echo "For support, contact Jake Abendroth at contact@jakea.net"
EOF
        chmod +x "$package_dir/install.sh"
        
        # Create archive
        tar -czf "$DIST_DIR/lao-$VERSION-macos-x86_64.tar.gz" -C "$archive_dir" "lao-$VERSION-macos"
        
        echo "✅ macOS package created: $DIST_DIR/lao-$VERSION-macos-x86_64.tar.gz"
        ;;
        
    "linux")
        echo "🐧 Creating Linux package..."
        
        # Create tar.gz archive
        archive_dir="$DIST_DIR/tar"
        package_dir="$archive_dir/lao-$VERSION-linux"
        mkdir -p "$package_dir"
        
        # Copy binaries
        cp "$TARGET_DIR/release/lao-cli" "$package_dir/"
        cp "$TARGET_DIR/release/lao-ui" "$package_dir/"
        
        # Copy plugins
        mkdir -p "$package_dir/plugins"
        cp plugins/*.so "$package_dir/plugins/" 2>/dev/null || true
        
        # Copy documentation
        cp README.md "$package_dir/" 2>/dev/null || true
        
        # Create install script
        cat > "$package_dir/install.sh" << 'EOF'
#!/bin/bash
set -e

echo "Installing LAO Orchestrator by Jake Abendroth on Linux..."

# Create directories
sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/local/lib/lao/plugins

# Install binaries
sudo cp lao-cli /usr/local/bin/
sudo cp lao-ui /usr/local/bin/
sudo chmod +x /usr/local/bin/lao-*

# Install plugins
sudo cp plugins/*.so /usr/local/lib/lao/plugins/ 2>/dev/null || true

echo "✅ LAO installed successfully!"
echo "Run 'lao-ui' to start the GUI or 'lao-cli --help' for CLI options"
echo "For support, contact Jake Abendroth at contact@jakea.net"
EOF
        chmod +x "$package_dir/install.sh"
        
        # Create archive
        tar -czf "$DIST_DIR/lao-$VERSION-linux-x86_64.tar.gz" -C "$archive_dir" "lao-$VERSION-linux"
        
        echo "✅ Linux package created: $DIST_DIR/lao-$VERSION-linux-x86_64.tar.gz"
        ;;
        
    "windows")
        echo "🪟 Creating Windows package..."
        
        # Create ZIP archive
        archive_dir="$DIST_DIR/zip"
        package_dir="$archive_dir/lao-$VERSION-windows"
        mkdir -p "$package_dir"
        
        # Copy binaries
        cp "$TARGET_DIR/release/lao-cli.exe" "$package_dir/" 2>/dev/null || true
        cp "$TARGET_DIR/release/lao-ui.exe" "$package_dir/" 2>/dev/null || true
        
        # Copy plugins
        mkdir -p "$package_dir/plugins"
        cp plugins/*.dll "$package_dir/plugins/" 2>/dev/null || true
        
        # Copy documentation
        cp README.md "$package_dir/" 2>/dev/null || true
        
        # Create install script
        cat > "$package_dir/install.bat" << 'EOF'
@echo off
echo Installing LAO Orchestrator by Jake Abendroth on Windows...

REM Create directories
mkdir "%ProgramFiles%\LAO" 2>nul
mkdir "%ProgramFiles%\LAO\plugins" 2>nul

REM Install binaries
copy "lao-cli.exe" "%ProgramFiles%\LAO\" >nul
copy "lao-ui.exe" "%ProgramFiles%\LAO\" >nul

REM Install plugins
copy "plugins\*.dll" "%ProgramFiles%\LAO\plugins\" >nul

REM Add to PATH
setx PATH "%PATH%;%ProgramFiles%\LAO" /M >nul

echo LAO installed successfully!
echo Run 'lao-ui.exe' to start the GUI or 'lao-cli.exe --help' for CLI options
echo For support, contact Jake Abendroth at contact@jakea.net
pause
EOF
        
        # Create archive
        cd "$archive_dir"
        zip -r "$DIST_DIR/lao-$VERSION-windows-x86_64.zip" "lao-$VERSION-windows"
        cd "$ROOT_DIR"
        
        echo "✅ Windows package created: $DIST_DIR/lao-$VERSION-windows-x86_64.zip"
        ;;
        
    *)
        echo "❌ Unsupported platform: $PLATFORM"
        exit 1
        ;;
esac

echo "🎉 Package creation complete!"
echo "📦 Packages created in: $DIST_DIR"
echo "📋 Available packages:"
ls -la "$DIST_DIR"/*.{tar.gz,zip} 2>/dev/null || echo "  No packages found"