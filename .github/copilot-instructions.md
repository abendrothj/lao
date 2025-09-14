# LAO: Local AI Workflow Orchestrator

LAO is a Rust-based desktop application with a Tauri frontend that orchestrates local AI models through a plugin system. It supports workflow creation, visual DAG editing, and plugin development.

**Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.**

## Working Effectively

### Prerequisites
- **Rust 1.70+**: Install from https://rustup.rs/
- **Node.js 20+**: Install from https://nodejs.org/
- **System Dependencies (Linux)**: Install Tauri prerequisites
  ```bash
  sudo apt-get update
  sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf pkg-config
  ```

### Building the Project
- **Full Release Build**: 
  ```bash
  cargo build --release
  ```
  **NEVER CANCEL**: Takes 4-5 minutes. Set timeout to 10+ minutes.

- **Debug Build**: 
  ```bash
  cargo build
  ```
  Takes ~15 seconds.

- **Run Tests**: 
  ```bash
  cargo test
  ```
  **NEVER CANCEL**: Takes 2-3 minutes. Set timeout to 5+ minutes.

### Running the CLI
**CRITICAL**: Due to hardcoded plugin paths, the CLI must be run from the `core/` directory:
```bash
cd core
../target/release/lao-cli --help
../target/release/lao-cli plugin-list
../target/release/lao-cli run ../workflows/test.yaml
```

Available commands:
- `run` - Execute workflow YAML files
- `validate` - Validate workflow syntax and plugin availability
- `plugin-list` - List all available plugins
- `prompt` - Generate workflows from natural language prompts
- `validate-prompts` - Test prompt-to-workflow generation

### Running the UI
```bash
cd ui/lao-ui
npm install    # Takes ~8 seconds
npm run build  # Takes ~4 seconds
npm run tauri dev  # Start development mode
```

## Validation

### Manual Testing Scenarios
1. **Plugin System Validation**:
   ```bash
   cd core
   ../target/release/lao-cli plugin-list
   # Should show: EchoPlugin, WhisperPlugin, OllamaPlugin, etc.
   ```

2. **Workflow Execution**:
   ```bash
   cd core
   ../target/release/lao-cli run ../workflows/test.yaml
   # Should output: "Step 1: Hello from echo!"
   ```

3. **UI Build and Start**:
   ```bash
   cd ui/lao-ui
   npm run build
   # Should complete without errors
   ```

### Validation Requirements
- **ALWAYS** run from correct directories as specified above
- **NEVER CANCEL** long-running builds or tests
- Test workflow execution with the corrected workflow files
- Verify plugin discovery shows all 8 built-in plugins

## Repository Structure

### Core Components
- **`core/`**: Main orchestrator library and plugin system
- **`cli/`**: Command-line interface binary
- **`lao_plugin_api/`**: Plugin API definitions and traits
- **`plugins/`**: Built-in plugins (EchoPlugin, WhisperPlugin, etc.)
- **`ui/lao-ui/`**: Tauri + Svelte desktop application
- **`workflows/`**: Example workflow YAML files

### Plugin System
- Plugins are built as dynamic libraries (.so/.dll files)
- Plugin discovery loads from `plugins/` directory 
- Use full plugin names in workflows (e.g., `EchoPlugin` not `Echo`)
- Each plugin has a `plugin.yaml` manifest file

### Build Artifacts
- Release binaries: `target/release/lao-cli`
- Plugin libraries: `target/release/deps/lib*_plugin.so`
- UI build output: `ui/lao-ui/build/`

## Common Issues and Solutions

### Plugin Path Issues
**Problem**: "Plugin 'X' not found" errors
**Solution**: 
- Run CLI from `core/` directory: `cd core && ../target/release/lao-cli ...`
- Use full plugin names in workflows (EchoPlugin, not Echo)

### Build Failures
**Problem**: Tauri build fails with missing system libraries
**Solution**: Install system dependencies first:
```bash
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf pkg-config
```

### Workspace Resolver Warning
**Problem**: Warning about workspace resolver version
**Solution**: This is non-fatal, builds still work correctly

## Development Workflow
1. **Start with tests**: `cargo test` to ensure everything works
2. **Build in debug mode**: `cargo build` for faster iteration
3. **Test CLI functionality**: Run from `core/` directory
4. **Test UI changes**: Use `npm run tauri dev` for hot reload
5. **Final validation**: Full release build and test suite

## Plugin Development
- Plugin generator tool exists but has compilation errors
- Copy existing plugins as templates (EchoPlugin is simplest)
- Implement the `LaoPlugin` trait and export C ABI functions
- Place compiled .so/.dll files in `plugins/` directory

## Time Expectations
- **Release Build**: 4-5 minutes (NEVER CANCEL)
- **Debug Build**: ~15 seconds
- **Test Suite**: 2-3 minutes (NEVER CANCEL)
- **npm install**: ~8 seconds  
- **UI Build**: ~4 seconds
- **Plugin Discovery**: Instant

Always set timeouts of 10+ minutes for release builds and 5+ minutes for test runs.