# LAO CLI

This is the dedicated command-line interface for the LAO Orchestrator.

## Usage

```
cargo run --bin lao-cli run workflows/test.yaml
cargo run --bin lao-cli validate workflows/test.yaml
cargo run --bin lao-cli plugin list
```

## Commands
- `run <workflow.yaml> [--dry-run]`  
  Run a workflow. Use `--dry-run` to simulate execution.
- `validate <workflow.yaml>`  
  Validate workflow structure and plugin availability.
- `plugin list`  
  List all available plugins and their IO signatures. 