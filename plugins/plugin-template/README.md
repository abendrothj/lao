# LAO Plugin Template

This directory provides a starting point for creating a new LAO plugin.

## How to Use
1. Copy the `plugin-template` directory and rename it (e.g., `MyPlugin`).
2. Update `Cargo.toml` with your plugin's name and metadata.
3. Implement your plugin logic in `src/lib.rs` by implementing the `LaoPlugin` trait.
4. Update `plugin.yaml` with your plugin's name, version, description, input, and output types.
5. Add a `README.md` describing your plugin's usage, input/output, and example workflows.
6. Build your plugin as a `cdylib`:
   ```sh
   cargo build --release
   ```
7. Place the resulting `.dll`/`.so`/`.dylib` in the `plugins/` directory of your LAO orchestrator.

## Example plugin.yaml
```yaml
name: MyPlugin
version: 0.1.0
description: "Describe what your plugin does."
input: string
output: string
```

## Example lib.rs
```rust
use lao_plugin_api::{LaoPlugin, PluginInput, PluginOutput};

pub struct MyPlugin;

impl LaoPlugin for MyPlugin {
    fn execute(&self, input: PluginInput) -> PluginOutput {
        // Your logic here
    }
}

#[no_mangle]
pub extern "C" fn plugin_entry_point() -> *mut dyn LaoPlugin {
    Box::into_raw(Box::new(MyPlugin))
}
``` 