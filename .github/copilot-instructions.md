# LAO: Local AI Workflow Orchestrator

LAO is a cross-platform Rust-based Local AI Workflow Orchestrator with a modular plugin system, visual workflow builder, and CLI interface. It supports offline execution with Tauri-based UI (Rust backend + Svelte frontend).

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Working Effectively

### Bootstrap and Build System
Follow these exact steps to set up the development environment:

1. **Install System Dependencies (Linux/Ubuntu)**:
   ```bash
   sudo apt update
   sudo apt install -y libgtk-3-dev libglib2.0-dev libpango1.0-dev libcairo2-dev libgdk-pixbuf-2.0-dev libatk1.0-dev libsoup2.4-dev libjavascriptcoregtk-4.1-dev libwebkit2gtk-4.1-dev pkg-config
   ```
   - These GTK and WebKit dependencies are **REQUIRED** for Tauri UI compilation
   - Without these, the workspace will fail to build with glib-sys errors

2. **Verify Rust and Node.js Versions**:
   ```bash
   rustc --version  # Should be 1.70+ (tested with 1.89.0)
   cargo --version
   node --version   # Should be 16+ (tested with v20.19.5) 
   npm --version    # Should be 8+ (tested with 10.8.2)
   ```

3. **Build the Complete Workspace**:
   ```bash
   # Quick check (80 seconds)
   cargo check --workspace
   
   # Full release build (4.5 minutes) - NEVER CANCEL
   cargo build --workspace --release
   ```
   - **NEVER CANCEL**: Build takes 4-5 minutes. NEVER CANCEL. Set timeout to 10+ minutes.
   - Workspace uses resolver = "1" (ignore deprecation warnings)
   - Some unused import warnings are expected

4. **Build and Install Plugins**:
   ```bash
   # Copy plugin shared libraries to plugins/ directory
   ./tools/dll-puller.sh
   ```
   - This script copies all built plugin .so files to the plugins/ directory
   - Cross-platform: handles .so (Linux), .dylib (macOS), .dll (Windows)
   - Takes less than 1 second to complete

### Testing and Validation

1. **Run All Tests** (30 seconds) - NEVER CANCEL:
   ```bash
   cargo test --workspace
   ```
   - Should pass 17/17 tests including comprehensive integration tests
   - Tests validate plugin loading, workflow execution, caching, and error handling
   - Some clippy warnings in old dependencies are expected

2. **Validate Plugin System**:
   ```bash
   # List all loaded plugins (should show 8 plugins)
   cargo run --bin lao-cli plugin-list
   ```
   - Expected plugins: EchoPlugin, SummarizerPlugin, GGUFPlugin, WhisperPlugin, OllamaPlugin, LMStudioPlugin, PromptDispatcherPlugin, plugin-template

3. **Test CLI Workflow Execution**:
   ```bash
   # Note: The CLI has some issues with release builds vs debug builds
   # Tests pass but CLI workflow execution may fail - this is a known issue
   cargo run --bin lao-cli run workflows/test.yaml
   ```

### UI Development

1. **Install UI Dependencies** (7 seconds):
   ```bash
   cd ui/lao-ui
   npm install
   ```
   - Some npm audit warnings are expected (non-critical)

2. **Build UI Static Assets** (3 seconds):
   ```bash
   npm run build
   ```
   - Some Svelte linting warnings are expected but non-blocking
   - Missing eslint-plugin-svelte dependency is known issue

3. **Test Tauri CLI**:
   ```bash
   npm run tauri -- --help
   ```
   - Should show Tauri CLI commands
   - **Note**: Cannot actually run `tauri dev` in CI environment due to display requirements

## Validation Scenarios

### Manual Testing Workflows
After making changes, always run these validation steps:

1. **Full Build Validation**:
   ```bash
   # From repo root
   cargo clean
   cargo build --workspace --release  # 4.5 minutes - NEVER CANCEL
   ./tools/dll-puller.sh
   cargo test --workspace             # 30 seconds
   ```

2. **Plugin System Validation**:
   ```bash
   cargo run --bin lao-cli plugin-list
   # Should see 8 plugins loaded successfully
   ```

3. **UI Build Validation**:
   ```bash
   cd ui/lao-ui
   npm install
   npm run build
   npm run tauri -- --help
   ```

## Key Projects and Structure

### Repository Structure
```
lao/
├── core/                    # Core orchestrator library
├── cli/                     # Command-line interface
├── lao_plugin_api/         # Plugin API definitions
├── plugins/                # Built-in plugins (8 total)
│   ├── EchoPlugin/         # Simple echo/test plugin
│   ├── SummarizerPlugin/   # Text summarization
│   ├── OllamaPlugin/       # Ollama LLM integration
│   └── ...                 # Other AI model plugins
├── ui/lao-ui/              # Tauri desktop application
├── tools/                  # Build utilities
│   ├── dll-puller.sh       # Plugin library copier
│   ├── plugin-generator/   # Plugin scaffold tool
│   └── plugin-registry/    # Plugin registry server
└── workflows/              # Example workflow files
```

### Critical Build Timing Expectations
- **cargo check --workspace**: ~80 seconds
- **cargo build --workspace --release**: ~4.5 minutes - **NEVER CANCEL**
- **cargo test --workspace**: ~30 seconds
- **./tools/dll-puller.sh**: <1 second
- **npm install** (UI): ~7 seconds
- **npm run build** (UI): ~3 seconds

### Known Issues and Workarounds
1. **Workspace resolver warning**: Non-critical deprecation warning, workspace still builds
2. **CLI release binary issue**: Debug builds work correctly, release binary has plugin loading issues
3. **ESLint missing dependencies**: UI linting not fully configured but builds work
4. **GTK dependencies**: Must install system libraries before first build or build will fail

## Working with Plugins
- All plugins implement the `LaoPlugin` trait with C ABI exports
- Plugin building creates shared libraries (.so/.dylib/.dll)
- Use `./tools/dll-puller.sh` to copy built plugins to runtime directory
- Plugin manifest files (plugin.yaml) define metadata and capabilities

## Development Workflow
1. Make changes to code
2. Build: `cargo build --workspace --release` (4.5 min - **NEVER CANCEL**)
3. Copy plugins: `./tools/dll-puller.sh`
4. Test: `cargo test --workspace` (30 sec)
5. Validate CLI: `cargo run --bin lao-cli plugin-list`
6. Build UI: `cd ui/lao-ui && npm run build`

## Common Commands Reference
- **Build workspace**: `cargo build --workspace --release`
- **Run tests**: `cargo test --workspace`
- **List plugins**: `cargo run --bin lao-cli plugin-list`
- **Copy plugins**: `./tools/dll-puller.sh`
- **UI development**: `cd ui/lao-ui && npm run tauri dev` (requires display)
- **UI build**: `cd ui/lao-ui && npm run build`

## Repository Validation Commands
Use these frequently-needed command outputs:

### Project root listing
```bash
ls -la
# .git .github .gitignore Cargo.toml LICENSE README.md
# assets cli core dll-puller.ps1 docs lao_plugin_api
# plugins tools ui workflows
```

### Plugin directory after build
```bash
ls plugins/lib*.so
# libecho_plugin.so libgguf_plugin.so liblmstudio_plugin.so
# libollama_plugin.so libplugin_template.so 
# libprompt_dispatcher_plugin.so libsummarizer_plugin.so
# libwhisper_plugin.so
```