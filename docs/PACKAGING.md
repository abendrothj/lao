# 📦 LAO Packaging Guide

This guide explains how to create packages for LAO across different platforms for distribution.

## 🎯 Package Types

### **Linux Packages**
- **`.deb`** - Debian/Ubuntu packages
- **`.rpm`** - Red Hat/Fedora packages  
- **`.tar.gz`** - Universal Linux archive
- **`.AppImage`** - Portable Linux application

### **macOS Packages**
- **`.dmg`** - macOS disk image installer
- **`.tar.gz`** - Universal macOS archive
- **`.app`** - macOS application bundle

### **Windows Packages**
- **`.msi`** - Windows installer package
- **`.zip`** - Portable Windows archive
- **`.exe`** - Windows executable installer

## 🚀 Quick Start

### **Simple Package Creation**
```bash
# Create packages for current platform
bash scripts/create-simple-packages.sh
```

### **Full Package Creation**
```bash
# Create all package formats for current platform
bash scripts/create-packages.sh
```

## 📋 Package Contents

Each package includes:

### **Core Binaries**
- `lao-cli` - Command-line interface
- `lao-ui` - Graphical user interface

### **Plugins** (Platform-specific)
- **Linux**: `*.so` files
- **macOS**: `*.dylib` files  
- **Windows**: `*.dll` files

### **Documentation**
- `README.md` - Project documentation
- `LICENSE` - License information
- `install.sh`/`install.bat` - Installation script

## 🔧 Platform-Specific Instructions

### **Linux Packages**

#### **Debian Package (.deb)**
```bash
# Install dependencies
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf pkg-config

# Create package
bash scripts/create-packages.sh

# Install package
sudo dpkg -i dist/lao_0.1.0_amd64.deb
```

#### **RPM Package (.rpm)**
```bash
# Install dependencies
sudo dnf install gtk3-devel webkit2gtk3-devel

# Create package
bash scripts/create-packages.sh

# Install package
sudo rpm -i dist/lao-0.1.0-1.*.rpm
```

#### **AppImage**
```bash
# Install appimagetool
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage

# Create AppImage
./appimagetool-x86_64.AppImage dist/AppImage/LAO.AppDir dist/LAO-0.1.0-x86_64.AppImage
```

### **macOS Packages**

#### **DMG Package**
```bash
# Create DMG
bash scripts/create-packages.sh

# Mount and install
open dist/LAO-0.1.0.dmg
# Drag LAO.app to Applications folder
```

#### **Tar Archive**
```bash
# Create archive
bash scripts/create-simple-packages.sh

# Extract and install
tar -xzf dist/lao-0.1.0-macos-x86_64.tar.gz
cd lao-0.1.0-macos
sudo ./install.sh
```

### **Windows Packages**

#### **MSI Installer**
```bash
# Install WiX Toolset
# Download from: https://wixtoolset.org/releases/

# Create MSI
bash scripts/create-packages.sh

# Install MSI
msiexec /i dist/lao-0.1.0.msi
```

#### **ZIP Archive**
```bash
# Create archive
bash scripts/create-simple-packages.sh

# Extract and install
# Extract lao-0.1.0-windows-x86_64.zip
# Run install.bat as Administrator
```

## 🏗️ Build Process

### **1. Build Release Binaries**
```bash
cargo build --release --bin lao-cli
cargo build --release --bin lao-ui
```

### **2. Build Plugins**
```bash
bash scripts/build-plugins.sh
```

### **3. Create Packages**
```bash
bash scripts/create-packages.sh
```

## 📁 Package Structure

### **Linux Package Structure**
```
lao-0.1.0-linux/
├── lao-cli                 # CLI binary
├── lao-ui                  # UI binary
├── plugins/                # Plugin directory
│   ├── libecho_plugin.so
│   ├── libollama_plugin.so
│   └── ...
├── README.md               # Documentation
└── install.sh              # Installation script
```

### **macOS Package Structure**
```
lao-0.1.0-macos/
├── lao-cli                 # CLI binary
├── lao-ui                  # UI binary
├── plugins/                # Plugin directory
│   ├── libecho_plugin.dylib
│   ├── libollama_plugin.dylib
│   └── ...
├── README.md               # Documentation
└── install.sh              # Installation script
```

### **Windows Package Structure**
```
lao-0.1.0-windows/
├── lao-cli.exe             # CLI binary
├── lao-ui.exe              # UI binary
├── plugins/                # Plugin directory
│   ├── libecho_plugin.dll
│   ├── libollama_plugin.dll
│   └── ...
├── README.md               # Documentation
└── install.bat             # Installation script
```

## 🔄 Automated Packaging

### **GitHub Actions**
The project includes automated packaging via GitHub Actions:

```yaml
# .github/workflows/package.yml
name: Cross-Platform Packaging
on:
  push:
    tags: ['v*']
  workflow_dispatch:
```

### **Manual Trigger**
```bash
# Trigger packaging workflow
gh workflow run package.yml -f platform=all
```

## 🧪 Testing Packages

### **Test Installation**
```bash
# Linux
sudo dpkg -i dist/lao_0.1.0_amd64.deb
lao-cli --help
lao-ui

# macOS
open dist/LAO-0.1.0.dmg
# Test installation

# Windows
msiexec /i dist/lao-0.1.0.msi
# Test installation
```

### **Test Functionality**
```bash
# Test CLI
lao-cli plugin-list
lao-cli run workflows/test.yaml

# Test UI
lao-ui
# Verify GUI loads and plugins work
```

## 📊 Package Sizes

| Platform | Package Type | Size | Contents |
|----------|-------------|------|----------|
| **Linux** | `.deb` | ~15MB | Binaries + plugins |
| **Linux** | `.tar.gz` | ~8MB | Binaries + plugins |
| **macOS** | `.dmg` | ~20MB | App bundle + plugins |
| **macOS** | `.tar.gz` | ~8MB | Binaries + plugins |
| **Windows** | `.msi` | ~18MB | Binaries + plugins |
| **Windows** | `.zip` | ~8MB | Binaries + plugins |

## 🔧 Customization

### **Package Metadata**
Edit `packaging.toml` to customize:
- Package descriptions
- Dependencies
- Maintainer information
- License details

### **Installation Scripts**
Modify installation scripts in:
- `scripts/create-packages.sh`
- `scripts/create-simple-packages.sh`

### **Platform-Specific Settings**
Configure platform-specific options:
- Desktop file creation
- Registry entries
- PATH modifications
- Service installation

## 🚨 Troubleshooting

### **Common Issues**

#### **Missing Dependencies**
```bash
# Linux
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev

# macOS
xcode-select --install

# Windows
# Install Visual Studio Build Tools
```

#### **Permission Errors**
```bash
# Fix permissions
chmod +x scripts/*.sh
chmod +x dist/*/install.sh
```

#### **Plugin Loading Issues**
```bash
# Verify plugin files
ls -la plugins/*.{so,dylib,dll}

# Test plugin loading
cargo run --bin lao-cli plugin-list
```

## 📚 Additional Resources

- [Rust Packaging Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Debian Packaging](https://www.debian.org/doc/manuals/debian-faq/ch-pkg_basics.en.html)
- [RPM Packaging](https://rpm-packaging-guide.github.io/)
- [macOS App Distribution](https://developer.apple.com/distribute/)
- [Windows Installer Creation](https://docs.microsoft.com/en-us/windows/msi/)

## 🎉 Success!

You now have comprehensive packaging for LAO across all major platforms! Users can easily install and use LAO on Linux, macOS, and Windows with platform-appropriate package formats.
