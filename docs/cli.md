# LAO CLI Documentation

## Usage
```
lao <COMMAND> [OPTIONS]
```

## Commands
- `run <workflow.yaml> [--dry-run]`  
  Run a workflow. Use `--dry-run` to simulate execution and show expected IO types.
- `validate <workflow.yaml>`  
  Validate workflow structure, types, and plugin availability.
- `plugin-list`  
  List all available plugins, their IO signatures, and descriptions.

## Examples
```
lao run workflows/test.yaml
lao run workflows/test.yaml --dry-run
lao validate workflows/test.yaml
lao plugin-list
``` 