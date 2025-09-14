# LAO: Local AI Workflow Orchestrator

LAO is a cross-platform desktop application that orchestrates local AI models through a modular plugin system. Built with Rust + Tauri, it supports agentic workflow creation, visual DAG editing, and prompt-driven orchestration—all running offline.

**Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.**

## Repository Architecture

LAO is a Rust workspace with the following structure:

### Core Components
- **`core/`** - Main orchestrator library (`lao-orchestrator-core`)
  - DAG engine with parallel execution, caching, retries
  - Plugin system with dynamic loading
  - Prompt dispatcher with LLM integration
- **`cli/`** - Command-line interface (`lao-cli`)
  - Workflow execution, validation, plugin management
  - Prompt-driven workflow generation
- **`lao_plugin_api/`** - Plugin trait definitions and API
- **`ui/lao-ui/`** - Tauri + Svelte desktop application
  - Visual DAG builder, real-time execution monitoring

### Plugin Ecosystem
- **`plugins/`** - Built-in plugins (8 total):
  - `EchoPlugin` - Simple text processing (best for testing)
  - `WhisperPlugin` - Speech-to-text transcription
  - `OllamaPlugin` - Local LLM integration
  - `GGUFPlugin` - GGUF model support
  - `LMStudioPlugin` - LM Studio integration
  - `SummarizerPlugin` - Text summarization
  - `PromptDispatcherPlugin` - Workflow generation from prompts
- **`tools/`** - Development utilities
  - `plugin-generator/` - Plugin scaffolding tool
  - `plugin-registry/` - Plugin discovery system

### Supporting Directories
- **`workflows/`** - Example YAML workflow files
- **`docs/`** - Comprehensive documentation
  - `architecture.md`, `plugins.md`, `workflows.md`, `cli.md`
- **`assets/`** - Application resources

## Prerequisites & Setup

### Required Dependencies
- **Rust 1.70+**: Install from https://rustup.rs/
- **Node.js 20+**: Install from https://nodejs.org/

### System Dependencies (Linux)
Install Tauri prerequisites:
```bash
sudo apt-get update
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf pkg-config
```

## Building & Running

### Build Commands
```bash
# Full workspace build (release)
cargo build --release
# ⚠️ NEVER CANCEL: Takes 4-5 minutes, set timeout to 10+ minutes

# Debug build (faster iteration)
cargo build
# Takes ~15 seconds

# Run complete test suite
cargo test
# ⚠️ NEVER CANCEL: Takes 2-3 minutes, set timeout to 5+ minutes
```

### CLI Usage
**CRITICAL**: Due to hardcoded plugin paths, always run CLI from the `core/` directory:

```bash
cd core

# Basic commands
../target/release/lao-cli --help
../target/release/lao-cli plugin-list
../target/release/lao-cli run ../workflows/test.yaml

# Prompt-driven workflows
../target/release/lao-cli prompt "Summarize this audio and tag action items"
../target/release/lao-cli validate-prompts

# Workflow validation
../target/release/lao-cli validate ../workflows/test.yaml
```

Available CLI commands:
- `run` - Execute workflow YAML files
- `validate` - Validate workflow syntax and plugin availability
- `plugin-list` - List all available plugins
- `prompt` - Generate workflows from natural language prompts
- `validate-prompts` - Test prompt-to-workflow generation

### UI Development
```bash
cd ui/lao-ui
npm install        # ~8 seconds
npm run build      # ~4 seconds
npm run tauri dev  # Start development mode with hot reload
```

## Plugin System

### Plugin Architecture
- Plugins are dynamic libraries (.so/.dll) loaded at runtime
- Each plugin implements the `LaoPlugin` trait from `lao_plugin_api`
- Plugin discovery scans the `plugins/` directory
- Plugins export a C ABI function `plugin_entry_point`

### Plugin Usage in Workflows
- **Always use full plugin names** in YAML workflows (e.g., `EchoPlugin` not `Echo`)
- Plugin input/output types are strongly typed via the API
- Each plugin directory contains a `plugin.yaml` manifest

### Plugin Development
```bash
# Use existing plugins as templates (EchoPlugin is simplest)
# Implement LaoPlugin trait and export C ABI functions
# Build as cdylib and place .so/.dll in plugins/ directory

# Plugin generator tool (has compilation issues currently)
cd tools/plugin-generator
cargo build
```

## Development Workflow

### Recommended Development Process
1. **Start with tests**: `cargo test` to ensure everything works
2. **Build in debug mode**: `cargo build` for faster iteration  
3. **Test CLI functionality**: Always run from `core/` directory
4. **Validate changes**: Use CLI validation commands
5. **Test UI changes**: Use `npm run tauri dev` for hot reload
6. **Final validation**: Full release build and test suite

### Validation Scenarios
1. **Plugin System**:
   ```bash
   cd core
   ../target/release/lao-cli plugin-list
   # Should show 8 plugins: EchoPlugin, WhisperPlugin, OllamaPlugin, etc.
   ```

2. **Workflow Execution**:
   ```bash
   cd core
   ../target/release/lao-cli run ../workflows/test.yaml
   # Should output: "Step 1: Hello from echo!"
   ```

3. **Prompt Generation**:
   ```bash
   cd core
   ../target/release/lao-cli validate-prompts
   # Tests prompt library against expected outputs
   ```

## Common Issues & Solutions

### Plugin Path Resolution
**Problem**: "Plugin 'X' not found" errors
**Solution**: 
- Always run CLI from `core/` directory: `cd core && ../target/release/lao-cli ...`
- Use full plugin names in workflows (`EchoPlugin`, not `Echo`)

### Build Dependencies
**Problem**: Tauri build fails with missing system libraries
**Solution**: Install system dependencies first (see Prerequisites section)

### Workspace Warnings
**Problem**: Warning about workspace resolver version
**Solution**: This is non-fatal, builds still work correctly

## Documentation References

For detailed information, consult:
- **Architecture**: `docs/architecture.md` - Core components and data flow
- **Plugin Development**: `docs/plugins.md` - Plugin API and development guide
- **Workflow Syntax**: `docs/workflows.md` - YAML workflow specification
- **CLI Reference**: `docs/cli.md` - Complete command documentation
- **Observability**: `docs/observability.md` - Logging and monitoring

## Build Artifacts & Output

### Key Build Outputs
- **CLI Binary**: `target/release/lao-cli`
- **Core Library**: `target/release/lao-orchestrator-core`
- **Plugin Libraries**: `target/release/deps/lib*_plugin.so`
- **UI Application**: `ui/lao-ui/src-tauri/target/release/`

### Timeout Recommendations
Always set appropriate timeouts for long-running operations:
- **Release builds**: 10+ minutes (actual: 4-5 minutes)
- **Test suites**: 5+ minutes (actual: 2-3 minutes)
- **Debug builds**: 30 seconds (actual: ~15 seconds)
- **npm operations**: 30 seconds (actual: ~8-12 seconds)