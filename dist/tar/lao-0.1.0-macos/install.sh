#!/bin/bash
set -e

echo "Installing LAO Orchestrator on macOS..."

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
