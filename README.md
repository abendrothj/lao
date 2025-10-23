# ⚡️ LAO: Local AI Workflow Orchestrator

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust)
![Native GUI](https://img.shields.io/badge/GUI-Native%20Rust-green?logo=rust)
![Runs Offline](https://img.shields.io/badge/Runs-Offline-success?logo=powerbi&logoColor=white)

> Chain. Build. Run. All offline.
> LAO is how developers bend AI to their will—no cloud, no compromise.

---

## 🧠 What is LAO?

LAO is a cross-platform desktop tool for chaining local AI models and plugins into powerful, agentic workflows. It supports prompt-driven orchestration, visual DAG editing, and full offline execution.

---

## ✨ Features

- [x] Modular plugin system (Rust, local-first)
- [x] Offline DAG engine (retries, caching, lifecycle hooks)
- [x] Prompt-driven agentic workflows (LLM-powered, system prompt file)
- [x] Visual workflow builder (UI, YAML export, node/edge display)
- [x] CLI (run, validate, prompt, validate-prompts, plugin list)
- [x] Prompt library (Markdown + JSON, for validation/fine-tuning)
- [x] Test harness for prompt validation
- [ ] End-to-end “Run” from UI (execute and show logs/results)
- [x] UI streaming run with real-time step events and parallel execution option
- [ ] Node/edge editing in UI (drag, connect, edit)
- [ ] Plugin explainability (`lao explain plugin <name>`)
- [ ] Conditional/branching steps
- [ ] Plugin marketplace/discovery
- [ ] Live workflow status/logs in UI
- [ ] Multi-modal input (files, voice, etc.)
- [ ] Installer/distribution polish

---

## 🚀 Quickstart

```sh
# Run the native GUI
cargo run --bin lao-ui

# Run the CLI
cargo run --bin lao-cli run workflows/test.yaml
cargo run --bin lao-cli prompt "Summarize this audio and tag action items"
cargo run --bin lao-cli validate-prompts
```

---

## 🧩 Prompt-Driven Workflows

LAO can generate and execute workflows from natural language prompts using a local LLM (Ollama). The system prompt is editable at `core/prompt_dispatcher/prompt/system_prompt.txt`.

Example:
```bash
lao prompt "Refactor this Python file and add comments"
```

---

## 📚 Prompt Library & Validation

- Prompts and expected workflows: `core/prompt_dispatcher/prompt/prompt_library.md` and `.json`
- Validate with: `cargo run --bin lao-cli validate-prompts`
- Add new prompts to improve LLM output and test new plugins

---

## 🛠️ Contributing Plugins & Prompts
- Add new plugins by implementing the `LaoPlugin` trait, building as a `cdylib`, and placing the resulting library in the `plugins/` directory
- Expose a C ABI function named `plugin_entry_point` that returns a `Box<dyn LaoPlugin>`
- Add prompt/workflow pairs to the prompt library for validation and LLM tuning
- See `docs/plugins.md` and `docs/workflows.md` for details

---

## 📄 Documentation
- Architecture: `docs/architecture.md`
- Plugins: `docs/plugins.md`
- Workflows: `docs/workflows.md`
- CLI: `docs/cli.md`
- Observability: `docs/observability.md`

---

## 🌌 Manifesto
Cloud is optional. Intelligence is modular. Agents are composable.  
LAO is how devs build AI workflows with total control.  
**No tokens. No latency. No lock-in.**

Let’s define the category—one plugin at a time.