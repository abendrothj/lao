# LAO Architecture Overview

## Core Components

- **DAG Engine**: Executes workflows as directed acyclic graphs, handling step dependencies, retries, caching, and lifecycle hooks.
  - Now supports optional parallel execution per DAG level and emits structured step events for UI/CLI streaming.
- **Plugin System**: Modular Rust trait-based plugins for local AI tasks (Whisper, LLMs, custom agents). Plugins are loaded dynamically at runtime from the `plugins/` directory as shared libraries, and declare IO types and lifecycle hooks.
- **PromptDispatcherPlugin**: Uses a local LLM (Ollama) and a system prompt file to generate workflows from natural language prompts. Hot-swappable prompt at `core/prompt_dispatcher/prompt/system_prompt.txt`.
- **Prompt Library & Validation**: Prompts and expected workflows in Markdown/JSON, validated by a test harness and CLI command.
- **CLI**: Command-line interface for running, validating, and inspecting workflows and plugins. Supports prompt-driven execution and validation.
- **UI (Tauri + Svelte)**: Visual flow builder, prompt input, and execution monitor, communicating with the backend via Tauri bridge.
  - Real-time event stream: `workflow:status` and `workflow:done` messages with step updates.

## Agentic Workflow Generation
- User enters a prompt (CLI or UI)
- PromptDispatcherPlugin uses LLM + system prompt to generate YAML workflow
- Workflow is parsed, visualized, and executed as a DAG

## Prompt Validation/Test Harness
- Loads prompt library
- Runs each prompt through the dispatcher
- Compares generated workflow to expected output (structure-aware)
- CLI and test harness for validation

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
- Add new plugins by implementing the `LaoPlugin` trait, building as a `cdylib`, and placing the library in the `plugins/` directory.
- Extend CLI with new commands via Clap.
- UI can visualize any DAG structure and step status. 