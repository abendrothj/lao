# LAO Architecture Overview

## Core Components

- **DAG Engine**: Executes workflows as directed acyclic graphs, handling step dependencies, retries, caching, and lifecycle hooks.
- **Plugin System**: Modular Rust trait-based plugins for local AI tasks (Whisper, LLMs, custom agents). Plugins declare IO types and lifecycle hooks.
- **CLI**: Command-line interface for running, validating, and inspecting workflows and plugins. Supports dry-run, retries, and caching.
- **UI (Tauri + Svelte)**: Visual flow builder and execution monitor, communicating with the backend via Tauri bridge.

## Data Flow

1. User defines a workflow YAML (steps, dependencies, config).
2. CLI/UI loads and validates the workflow.
3. DAG engine builds the execution graph.
4. Each step:
   - Checks cache (if enabled)
   - Runs plugin (with retries/lifecycle hooks)
   - Logs output, errors, and status
5. UI/CLI displays results and logs.

## Extensibility
- Add new plugins by implementing the `LaoPlugin` trait.
- Extend CLI with new commands via Clap.
- UI can visualize any DAG structure and step status. 