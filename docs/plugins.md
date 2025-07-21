# LAO Plugin System

## What is a Plugin?
A plugin is a modular unit of computation that implements the `LaoPlugin` trait. Plugins can perform AI tasks (e.g., transcription, summarization, LLM inference, code refactoring) and are fully local/offline.

## Plugin Lifecycle
- `init`: Setup resources/config
- `pre_execute`: Prepare for execution
- `execute`: Main logic (input → output)
- `post_execute`: Cleanup or enrich output
- `shutdown`: Release resources

## Explainability & Discovery
- Use `lao plugin list` to see available plugins, IO signatures, and descriptions
- (Planned) `lao explain plugin <name>` for detailed info and examples
- (Planned) Dynamic plugin loading and community sharing

## Prompt-Driven Plugin Selection
- The PromptDispatcherPlugin maps user intent to plugins using the system prompt
- Add new plugins to the registry and update the system prompt to make them available for agentic workflows

## Contributing Plugins
- Implement the `LaoPlugin` trait in your plugin crate
- Build your plugin as a `cdylib` and place the resulting dynamic library (.dll/.so/.dylib) in the `plugins/` directory
- Expose a C ABI function named `plugin_entry_point` that returns a `Box<dyn LaoPlugin>`
- Add prompt/workflow pairs to the prompt library for validation

## Example Plugin Entry Point
```rust
#[no_mangle]
pub extern "C" fn plugin_entry_point() -> *mut dyn LaoPlugin {
    Box::into_raw(Box::new(EchoPlugin))
}
```

## Using Plugins in Workflows
Reference the plugin by name in your workflow YAML:
```