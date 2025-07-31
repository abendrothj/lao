# LAO Orchestrator Documentation

Welcome to the documentation for LAO Orchestrator! This site provides comprehensive guides and references for users, plugin developers, and contributors.

## 📚 Documentation Index

- [Architecture Overview](./architecture.md)
- [Plugin System](./plugins.md)
- [Workflows](./workflows.md)
- [CLI Usage](./cli.md)
- [Observability & Debugging](./observability.md)

## 🚀 Quick Start

1. **Install dependencies** and build the project (see main project README).
2. **Run a workflow:**
   ```sh
   lao run workflows/test.yaml
   ```
3. **Generate a workflow from a prompt:**
   ```sh
   lao prompt "Summarize this audio and tag action items"
   ```
4. **List available plugins:**
   ```sh
   lao plugin-list
   ```

## 🧩 Plugin Development
- See [Plugin System](./plugins.md) for how to create, test, and contribute plugins.

## 🛠️ Contributing
- Please read the [Architecture](./architecture.md) and [Plugin System](./plugins.md) docs before contributing.
- Add new prompt/workflow pairs to the prompt library for validation and LLM tuning.

---

For more information, see the individual documentation files above. 