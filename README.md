# 🧠 Local AI Workflow Orchestrator (LAO)

## 💡 Mission
Build a cross-platform desktop app that lets developers **create, chain, and run local AI workflows**—entirely offline. LAO is a dev-friendly orchestration layer for tools like Ollama, Whisper.cpp, and local LLMs.

---

## 🧩 Core Features
- **Model Runners:** Supports Whisper.cpp, Ollama, LM Studio, etc. Pluggable runtime interfaces, CLI + GUI for each model.
- **Workflow Builder:** Visual graph editor with draggable nodes. Each node = one model or task. Input/output pipes to connect flows (text, audio, JSON).
- **Local DAG Engine:** Lightweight directed acyclic graph processor. Allows caching, retries, and parallel execution. Fully offline, with logging and metrics.
- **Plugin Framework:** Devs can create custom actions (summarize, tag, translate, refactor). Node SDK exposed as a module. Easy to share community plugins.
- **CLI Interface:** `lao run workflow.yaml`, `lao list models`, `lao chain whisper → summarize → tag`.

---

## 🏗️ Architecture

```
LAO/
├── lao-ui/         # Tauri + Svelte desktop app (UI)
├── plugins/        # Community/user plugins (planned)
├── workflows/      # User workflow YAMLs (planned)
├── engine/         # DAG engine, core logic (planned)
├── docs/           # Documentation (planned)
└── ...
```

- **UI Layer:** Tauri (Rust) + Svelte for a fast, native desktop experience.
- **Backend/Orchestration:** Rust for performance, process control, and plugin support.
- **Model Runners:** Adapters for each tool (Whisper.cpp, Ollama, etc.), CLI invocation, output parsing.
- **Workflow Engine:** Parses YAML, builds execution graph, runs steps in order/parallel, logs outputs.
- **Plugin Framework:** Node SDK for custom actions, hot-reload or plugin discovery (planned).

---

## 🔬 Tech Stack
| Layer           | Options                           |
|----------------|------------------------------------|
| UI Framework   | Tauri + Svelte                     |
| Backend        | Rust                               |
| AI Models      | Ollama, Whisper.cpp, GGUF Models   |
| Workflow Engine| Custom DAG or Node-RED integration |
| Storage        | Local DB (SQLite, LowDB)           |

---

## 🖥️ UI/UX Goals
- Clean, minimal interface (Svelte)
- Tauri-based for small build size
- Dark mode, keyboard-first, with drag & drop
- “Zapier meets VS Code” energy

---

## 🔒 Privacy & Localism
- No network calls unless user enables plugins with APIs
- On-device storage of workflows and outputs
- Ideal for secure environments: finance, healthcare, defense

---

## ✨ Example Workflow YAML
```yaml
workflow: "Transcribe + Summarize"
steps:
  - run: whisper
    input: audio_file.wav
  - run: ollama
    model: mistral
    prompt: "Summarize this meeting transcript:"
  - run: tagger
    type: topic
```

---

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (with Cargo)
- [Node.js & npm](https://nodejs.org/)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites/)
- **Windows:** Visual Studio Build Tools (with C++ workload)

### Setup
1. **Clone the repo:**
   ```sh
   git clone <your-repo-url>
   cd LAO
   ```
2. **Install dependencies for the UI:**
   ```sh
   cd lao-ui
   npm install
   ```
3. **Run the app in development mode:**
   ```sh
   npm run tauri dev
   ```

---

## 🧑‍💻 Development
- Main UI code is in `lao-ui/` (Tauri + Svelte)
- Planned: `engine/` for DAG logic, `plugins/` for extensions, `workflows/` for YAML samples
- Contributions welcome! See below.

---

## 🤝 Contributing
1. Fork the repo and create your branch from `main`.
2. If you’ve added code that should be tested, add tests.
3. Ensure the code lints and builds.
4. Submit a pull request with a clear description.

---

## 📄 License
MIT (or specify your license here)

---

## 🙏 Acknowledgements
- [Tauri](https://tauri.app/)
- [Svelte](https://svelte.dev/)
- [Ollama](https://ollama.ai/)
- [Whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- [Node-RED](https://nodered.org/) (inspiration) 

---

## 🗺️ Roadmap
- **MVP**
  - [x] Scaffold Tauri + Svelte desktop app
  - [ ] Basic workflow YAML parsing and execution
  - [ ] Model runner adapters for Whisper.cpp and Ollama
  - [ ] Visual workflow builder UI
  - [ ] Local DAG engine with caching and retries
- **v1.0**
  - [ ] Plugin framework and SDK
  - [ ] Community plugin sharing
  - [ ] Workflow import/export
  - [ ] CLI improvements (chain, list, run)
  - [ ] Local DB integration (SQLite/LowDB)
- **Future**
  - [ ] Node-RED integration
  - [ ] Advanced metrics and logging
  - [ ] More model runners (LM Studio, GGUF, etc.)
  - [ ] Secure plugin sandboxing
  - [ ] Cross-platform packaging and auto-update

---

## 🧩 Plugin Documentation

### What is a Plugin?
A plugin is a custom action or node that can be added to LAO workflows. Plugins can:
- Run custom code (e.g., summarize, tag, translate, refactor)
- Integrate with local tools or scripts
- Extend the workflow builder with new node types

### How to Create a Plugin
1. **Create a new folder in `plugins/`**
2. **Implement your plugin logic:**
   - For Rust plugins: expose a function with a standard interface (see examples in `plugins/`)
   - For Node.js plugins (planned): export a function/module
3. **Add a manifest (e.g., `plugin.json`):**
   ```json
   {
     "name": "summarizer",
     "description": "Summarizes text using a local LLM",
     "entry": "summarizer.rs"
   }
   ```
4. **Test your plugin:**
   - Use the LAO CLI or UI to add your plugin node to a workflow
   - Check logs/output for errors

### Sharing Plugins
- Submit your plugin to the community plugins directory (planned)
- Share your plugin folder or manifest with others

### Example Plugin (Rust)
```rust
// plugins/summarizer.rs
use lao_plugin_api::{PluginContext, PluginResult};

pub fn run(ctx: PluginContext) -> PluginResult {
    // Your summarization logic here
}
```

---

## ❓ FAQ

### What platforms does LAO support?
- Windows, macOS, and Linux (cross-platform via Tauri)

### Do I need an internet connection?
- No. LAO is fully offline by default. Only plugins you enable may use network APIs.

### What models are supported?
- Whisper.cpp, Ollama, and (planned) LM Studio, GGUF, and more.

### How do I add a new model runner?
- Implement a new adapter in the `runners/` directory and register it in the workflow engine.

### Where are workflows and outputs stored?
- On-device, in the `workflows/` and output directories (configurable in future releases).

### How do I contribute?
- See the [Contributing](#-contributing) section above. Fork, branch, PR!

### Who maintains LAO?
- The LAO community. You can join by contributing code, plugins, or feedback! 