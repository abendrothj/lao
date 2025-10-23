# LAO Native GUI

Native Rust GUI for LAO Orchestrator built with egui.

## Features

- Visual workflow builder with drag-and-drop nodes
- Real-time workflow execution monitoring
- Multi-modal file upload support
- Live status updates and logging
- Workflow save/export functionality

## Running

```sh
# From project root
cargo run --bin lao-ui

# Or from this directory
cargo run
```

## Development

Built with:
- **egui**: Immediate mode GUI framework
- **eframe**: Application framework for egui
- **rfd**: File dialogs
- **tokio**: Async runtime

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
