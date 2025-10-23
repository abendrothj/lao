# Cross-Platform Support for LAO

LAO now has comprehensive cross-platform support for Linux, macOS, and Windows. This document explains how to build, deploy, and use LAO across different operating systems.

## 🌍 Supported Platforms

| Platform | OS | Architecture | Shared Library | Status |
|----------|----|--------------|----------------|---------|
| **Linux** | Ubuntu 20.04+ | x86_64 | `.so` | ✅ Supported |
| **macOS** | macOS 10.15+ | x86_64, ARM64 | `.dylib` | ✅ Supported |
| **Windows** | Windows 10+ | x86_64 | `.dll` | ✅ Supported |

## 🔧 Cross-Platform Architecture

### Plugin System
- **Dynamic Loading**: Uses `libloading` crate for cross-platform shared library loading
- **Platform Detection**: Automatic detection of OS and architecture
- **File Extensions**: Automatically handles `.so`, `.dylib`, and `.dll` files
- **ABI Compatibility**: Uses C ABI for plugin interface

### Build System
- **Rust Workspace**: Single workspace builds all components
- **Cross-Compilation**: Support for cross-compiling to different targets
- **Automated Scripts**: Platform-specific build and deployment scripts
- **CI/CD**: GitHub Actions for automated cross-platform builds

## 🚀 Building for Different Platforms

### Prerequisites

#### Linux
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies
sudo apt-get update
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf pkg-config
```

#### macOS
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Xcode Command Line Tools
xcode-select --install
```

#### Windows
```powershell
# Install Rust
Invoke-WebRequest -Uri "https://win.rustup.rs" -OutFile "rustup-init.exe"
.\rustup-init.exe

# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/
```

### Building Plugins

#### Automated Build (Recommended)
```bash
# Build all plugins for current platform
bash scripts/build-plugins.sh
```

#### Manual Build
```bash
# Build individual plugins
cd plugins/EchoPlugin
cargo build --release

# Copy to plugins directory
cp target/release/libecho_plugin.* ../../
```

### Cross-Compilation

#### Linux to Windows
```bash
# Add Windows target
rustup target add x86_64-pc-windows-msvc

# Build for Windows
cargo build --release --target x86_64-pc-windows-msvc
```

#### Linux to macOS
```bash
# Add macOS target (requires macOS SDK)
rustup target add x86_64-apple-darwin

# Build for macOS
cargo build --release --target x86_64-apple-darwin
```

## 📦 Platform-Specific Deployment

### Linux Packages
```bash
# Create Debian package
cargo deb

# Create RPM package
cargo rpm

# Create AppImage
cargo appimage
```

### macOS Packages
```bash
# Create DMG
cargo bundle --target x86_64-apple-darwin

# Create macOS app bundle
cargo bundle --target x86_64-apple-darwin --format app
```

### Windows Packages
```powershell
# Create MSI installer
cargo wix

# Create Windows app bundle
cargo bundle --target x86_64-pc-windows-msvc --format msi
```

## 🔌 Plugin Development

### Cross-Platform Plugin Template
```rust
use lao_plugin_api::{PluginInput, PluginOutput, PluginVTablePtr, PluginMetadata};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Platform-agnostic plugin implementation
unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    // Your plugin logic here
    let output = "Hello from cross-platform plugin!";
    let cstr = CString::new(output).unwrap();
    PluginOutput { text: cstr.into_raw() }
}

// Export the plugin vtable
#[no_mangle]
pub static PLUGIN_VTABLE: lao_plugin_api::PluginVTable = lao_plugin_api::PluginVTable {
    version: 1,
    name,
    run,
    free_output,
    run_with_buffer,
    get_metadata,
    validate_input,
    get_capabilities,
};

#[no_mangle]
pub extern "C" fn plugin_vtable() -> PluginVTablePtr {
    &PLUGIN_VTABLE
}
```

### Plugin Configuration
```toml
# Cargo.toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Required for dynamic loading

[dependencies]
lao_plugin_api = { path = "../../lao_plugin_api" }
```

## 🧪 Testing Cross-Platform Compatibility

### Local Testing
```bash
# Test plugin loading
cargo run --bin lao-cli plugin-list

# Test workflow execution
cargo run --bin lao-cli run workflows/test.yaml

# Test UI
cargo run --bin lao-ui
```

### CI/CD Testing
The project includes GitHub Actions workflows that automatically:
- Build on Linux, macOS, and Windows
- Test plugin loading and workflow execution
- Create platform-specific packages
- Verify cross-platform compatibility

## 🐛 Troubleshooting

### Common Issues

#### Plugin Loading Failures
```bash
# Check file permissions
ls -la plugins/

# Verify shared library format
file plugins/*.so    # Linux
file plugins/*.dylib # macOS
file plugins/*.dll   # Windows
```

#### Build Failures
```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Check Rust version
rustc --version
```

#### Cross-Compilation Issues
```bash
# Verify target is installed
rustup target list --installed

# Install missing targets
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-apple-darwin
```

## 📚 Platform-Specific Notes

### Linux
- Uses `libloading` with `RTLD_LAZY` flag
- Requires `libc` for system calls
- Supports both glibc and musl targets

### macOS
- Uses `libloading` with `RTLD_LAZY` flag
- Requires macOS SDK for cross-compilation
- Supports both Intel and Apple Silicon

### Windows
- Uses `libloading` with `LOAD_LIBRARY_SEARCH_DEFAULT_DIRS`
- Requires Visual Studio Build Tools
- Supports both MSVC and GNU toolchains

## 🔮 Future Enhancements

- **ARM64 Support**: Full ARM64 support for Linux and Windows
- **Mobile Platforms**: iOS and Android support
- **WebAssembly**: Browser-based plugin execution
- **Container Support**: Docker images for different platforms
- **Package Managers**: Integration with platform package managers

## 📖 Additional Resources

- [Rust Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [libloading Documentation](https://docs.rs/libloading/)
- [Cargo Cross-Compilation](https://doc.rust-lang.org/cargo/reference/config.html#buildtarget)
- [LAO Plugin Development Guide](docs/PLUGIN_DEVELOPMENT.md)
