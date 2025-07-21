# ⚡️ LAO: Local AI Workflow Orchestrator

> Chain. Build. Run. All offline.
> LAO is how developers bend AI to their will—no cloud, no compromise.

---

## 🧠 What is LAO?

LAO is a cross-platform desktop tool built for developers who want to **chain local AI models together** like whispering agents in a secure lab. Run audio transcription, summarization, tagging, and refactoring—all without sending a single byte to the cloud.

A DAG engine for workflows.  
A plugin system for community-powered agents.  
An orchestration layer for your offline AI stack.

If Zapier and Node-RED had a hacker baby powered by Ollama and Whisper.cpp, LAO is that child. And it’s ready to grow.

---

## ✨ Key Capabilities

- 🧩 **Modular Plugins** – Drop in local runners like Whisper, Mistral, or custom tasks. Build your own tools as composable agents.
- 📊 **Offline DAG Execution** – Chain tasks with full dependency ordering, retries, and validation. Works like a brain.
- 🎛️ **Hybrid CLI + GUI** – Tauri-powered visual flow builder + typed YAML workflows.
- 🔒 **Local-First Forever** – Your data stays yours. LAO never phones home unless you tell it to.
- 💻 **Dev-Loved Stack** – Rust backend + Svelte UI. Born fast, stays lean.

---

## 🚀 Quickstart

```sh
# Set up the UI
cd src/ui/lao-ui
npm install

# Run the desktop app
npm run tauri dev

# Run the backend with a sample workflow
cd src/core
cargo run --bin test_runner -- ../../workflows/test.yaml
```

Bring your own `.wav` files, plugin folders, and YAML workflows—and LAO will chain them together like clockwork.

---

## 🧭 Project Structure

```
lao-orchestrator/
├── core/            # Rust DAG engine
├── plugins/         # Custom AI runners
├── ui/              # Tauri + Svelte desktop builder
├── cli/             # CLI tools (coming soon)
├── workflows/       # Typed YAML execution templates
├── docs/            # Architecture + plugin API
├── .cursor/         # Task specs and dev automation
└── README.md
```

This repo breathes orchestration. Step into any folder and build.

---

## 🔌 Plugins

LAO plugins are modular units of intelligence. You can build one in Rust, declare it with a manifest, and invoke it like an agent.

```rust
pub trait LaoPlugin {
  fn name(&self) -> &'static str;
  fn init(&mut self, config: PluginConfig) -> Result<(), LaoError>;
  fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError>;
  fn io_signature(&self) -> IOSignature;
  fn shutdown(&mut self) -> Result<(), LaoError>;
}
```

Each plugin declares its input/output types, logs its execution, and runs in a sandbox of your choosing. No APIs required.

---

## 🧪 Workflow Example (Typed, Chained)

```yaml
workflow: "Summarize Meeting"
steps:
  - run: Whisper
    input: "meeting.wav"
  - run: Summarizer
    input_from: Whisper
  - run: Tagger
    input_from: Summarizer
```

This pipeline turns voice into structured insight. And it runs entirely offline. Welcome to the future.

---

## 🧙‍♂️ Why LAO Exists

Developers deserve tools that **respect their data**, **run at local speed**, and **don’t lock them into SaaS pricing**. LAO is a new category:

- 🔧 **Local Agent Orchestration**
- 🤖 **Offline AI Task Chaining**
- 💡 **Composable Intelligence Toolkit**

Built by devs, for devs—with room to customize everything. We don’t chase trends. We forge foundations.

---

## 🙌 Get Involved

- 🚀 Fork it. Hack it. Shape it.
- 🧱 Submit plugins, runners, templates.
- 💬 Join the LAO dev conversation.
- 🛠️ Use Cursor, Continue, or your favorite IDE—we’re multi-agent ready.

---

## 📄 License

MIT (open-source and unboxed)

---

## 💬 Questions?

> Is LAO cross-platform?  
Yes—macOS, Linux, Windows. Built on Tauri.

> Does it work offline?  
Absolutely. That’s the point.

> What models are supported?  
Whisper.cpp, Ollama, LM Studio, GGUF—and more to come.

> How do I contribute?  
Fork. Build. PR. The future of local AI tooling needs you.

---

## 🌌 Manifesto

Cloud is optional. Intelligence is modular. Agents are composable.  
LAO is how devs build AI workflows with total control.  
**No tokens. No latency. No lock-in.**

Let’s define the category—one plugin at a time.