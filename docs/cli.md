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
- `prompt <prompt>`  
  Generate and run a workflow from a natural language prompt using the local LLM.
- `validate-prompts [--path <json>] [--fail-fast] [--verbose]`  
  Validate prompt-to-workflow generation using the prompt library.
- (Planned) `explain plugin <name>`  
  Show detailed info and examples for a plugin.

## Examples
```
lao run workflows/test.yaml
lao run workflows/test.yaml --dry-run
lao validate workflows/test.yaml
lao plugin-list
lao prompt "Summarize this audio and tag action items"
lao validate-prompts --path core/prompt_dispatcher/prompt/prompt_library.json --verbose
``` 