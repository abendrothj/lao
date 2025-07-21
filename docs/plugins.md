# LAO Plugin System

## What is a Plugin?
A plugin is a modular unit of computation that implements the `LaoPlugin` trait. Plugins can perform AI tasks (e.g., transcription, summarization, LLM inference) and are fully local/offline.

## Plugin Lifecycle
- `init`: Setup resources/config
- `pre_execute`: Prepare for execution
- `execute`: Main logic (input → output)
- `post_execute`: Cleanup or enrich output
- `shutdown`: Release resources

## Example Trait
```rust
pub trait LaoPlugin {
    fn name(&self) -> &'static str;
    fn init(&mut self, config: PluginConfig) -> Result<(), LaoError>;
    fn pre_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError>;
    fn post_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn io_signature(&self) -> IOSignature;
    fn shutdown(&mut self) -> Result<(), LaoError> { Ok(()) }
}
```

## Registering Plugins
Add your plugin to the `PluginRegistry` in `default_registry()`.

## Using Plugins in Workflows
Reference the plugin by name in your workflow YAML:
```yaml
steps:
  - run: Echo
    input: "Hello!"
``` 