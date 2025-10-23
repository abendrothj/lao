# LAO Plugin Development Guide

Welcome to the comprehensive guide for developing plugins for LAO (Local AI Orchestrator). This guide covers everything you need to know to create, test, and publish plugins for the LAO ecosystem.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Plugin Architecture](#plugin-architecture)
3. [Development Tools](#development-tools)
4. [Plugin API Reference](#plugin-api-reference)
5. [Best Practices](#best-practices)
6. [Testing & Validation](#testing--validation)
7. [Publishing & Distribution](#publishing--distribution)
8. [Advanced Topics](#advanced-topics)
9. [Examples](#examples)
10. [Troubleshooting](#troubleshooting)

## Quick Start

### Prerequisites

- **Rust 1.70+**: Latest stable Rust toolchain
- **LAO Core**: Latest version of LAO orchestrator
- **Basic Rust Knowledge**: Understanding of Rust syntax and concepts
- **Git**: For version control and publishing

### Creating Your First Plugin

#### Option 1: Using the Plugin Generator (Recommended)

```bash
# Install the plugin generator
cargo install --path tools/plugin-generator

# Create a new plugin
lao-plugin-generator create my-awesome-plugin

# Follow the interactive prompts to configure your plugin
```

#### Option 2: Manual Setup

```bash
# Copy the template
cp -r plugins/plugin-template my-awesome-plugin
cd my-awesome-plugin

# Update the configuration files
# Edit Cargo.toml, plugin.yaml, and src/lib.rs
```

### Building and Testing

```bash
# Build your plugin
cargo build --release

# Run tests
cargo test

# Test with LAO CLI
lao plugin test my-awesome-plugin --input "test input"
```

## Plugin Architecture

### Overview

LAO plugins are dynamic libraries that implement a standardized interface. Each plugin:

- **Exports a `plugin_vtable`**: Contains function pointers to required operations
- **Implements required functions**: name, run, validate_input, etc.
- **Provides metadata**: Capabilities, dependencies, version information
- **Supports lifecycle management**: Loading, validation, execution, cleanup

### Plugin Lifecycle

```
1. Discovery    → LAO scans plugin directories
2. Loading      → Dynamic library loaded into memory
3. Validation   → Metadata and capabilities verified
4. Registration → Plugin added to registry
5. Execution    → Functions called during workflows
6. Cleanup      → Resources freed when no longer needed
```

### File Structure

```
my-awesome-plugin/
├── Cargo.toml          # Dependencies and metadata
├── src/
│   └── lib.rs          # Main plugin implementation
├── plugin.yaml         # Plugin manifest
├── examples/           # Example usage
├── tests/              # Plugin tests
└── README.md           # Documentation
```

## Development Tools

### Plugin Generator

The LAO plugin generator provides an interactive way to create new plugins:

```bash
# List available templates
lao-plugin-generator templates

# Create plugin with specific template
lao-plugin-generator create my-plugin --template ai-model

# Initialize in current directory
lao-plugin-generator init --non-interactive
```

#### Available Templates

- **basic**: Minimal plugin with core functionality
- **ai-model**: AI model integration template
- **data-processor**: Data transformation template
- **api-client**: API integration template
- **image-processor**: Image processing template
- **web-scraper**: Web scraping template

### CLI Tools

LAO provides comprehensive CLI tools for plugin management:

```bash
# List installed plugins
lao plugin list --detailed

# Install plugin from registry
lao plugin install ai-summarizer --version 2.1.0

# Validate plugin compatibility
lao plugin validate my-plugin

# Test plugin functionality
lao plugin test my-plugin --input "test data"

# Build plugin project
lao plugin build ./my-plugin --release
```

### Plugin Registry

The LAO plugin registry provides a centralized repository for plugin discovery and distribution:

```bash
# Start local registry server
lao-plugin-registry

# Browse plugins at http://localhost:8080/plugins
# API documentation at http://localhost:8080/docs
```

## Plugin API Reference

### Required Functions

#### `name() -> *const c_char`
Returns the plugin's name as a C string.

```rust
unsafe extern "C" fn name() -> *const c_char {
    let name_cstring = CString::new("MyPlugin").unwrap();
    name_cstring.into_raw()
}
```

#### `run(input: *const PluginInput) -> PluginOutput`
Main plugin execution function.

```rust
unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    // Validate input
    if input.is_null() {
        return error_output("null input");
    }
    
    // Process input
    let result = process_input(input);
    
    // Return output
    let output_cstring = CString::new(result).unwrap();
    PluginOutput { text: output_cstring.into_raw() }
}
```

#### `free_output(output: PluginOutput)`
Frees memory allocated for plugin output.

```rust
unsafe extern "C" fn free_output(output: PluginOutput) {
    if !output.text.is_null() {
        let _ = CString::from_raw(output.text);
    }
}
```

#### `get_metadata() -> PluginMetadata`
Returns comprehensive plugin metadata.

```rust
unsafe extern "C" fn get_metadata() -> PluginMetadata {
    PluginMetadata {
        name: name_cstring.into_raw(),
        version: version_cstring.into_raw(),
        description: desc_cstring.into_raw(),
        author: author_cstring.into_raw(),
        dependencies: deps_cstring.into_raw(),
        tags: tags_cstring.into_raw(),
        input_schema: std::ptr::null(),
        output_schema: std::ptr::null(),
        get_capabilities: caps_cstring.into_raw(),
    }
}
```

#### `validate_input(input: *const PluginInput) -> bool`
Validates plugin input before processing.

```rust
unsafe extern "C" fn validate_input(input: *const PluginInput) -> bool {
    if input.is_null() {
        return false;
    }
    
    let c_str = CStr::from_ptr((*input).text);
    let input_text = c_str.to_str().unwrap_or("");
    
    validate_input_internal(input_text)
}
```

#### `get_capabilities() -> *const c_char`
Returns JSON string describing plugin capabilities.

```rust
unsafe extern "C" fn get_capabilities() -> *const c_char {
    let capabilities = vec![
        PluginCapability {
            name: "process".to_string(),
            description: "Process input data".to_string(),
            input_type: PluginInputType::Text,
            output_type: PluginOutputType::Text,
        }
    ];
    
    let caps_json = serde_json::to_string(&capabilities).unwrap();
    let caps_cstring = CString::new(caps_json).unwrap();
    caps_cstring.into_raw()
}
```

### Data Structures

#### `PluginInput`
```rust
pub struct PluginInput {
    pub text: *mut c_char,
}
```

#### `PluginOutput`
```rust
pub struct PluginOutput {
    pub text: *mut c_char,
}
```

#### `PluginMetadata`
```rust
pub struct PluginMetadata {
    pub name: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
    pub author: *const c_char,
    pub dependencies: *const c_char,
    pub tags: *const c_char,
    pub input_schema: *const c_char,
    pub output_schema: *const c_char,
    pub get_capabilities: *const c_char,
}
```

#### `PluginCapability`
```rust
pub struct PluginCapability {
    pub name: String,
    pub description: String,
    pub input_type: PluginInputType,
    pub output_type: PluginOutputType,
}
```

#### `PluginDependency`
```rust
pub struct PluginDependency {
    pub name: String,
    pub version: String,
    pub optional: bool,
}
```

### Plugin VTable

The plugin vtable is the main interface that LAO uses to interact with plugins:

```rust
#[no_mangle]
pub static plugin_vtable: PluginVTable = PluginVTable {
    version: 1,
    name,
    run,
    free_output,
    run_with_buffer,
    get_metadata,
    validate_input,
    get_capabilities,
};
```

## Best Practices

### 1. Error Handling

- **Always validate input**: Check for null pointers and invalid data
- **Return meaningful errors**: Provide clear error messages
- **Use proper logging**: Include debug information for troubleshooting
- **Handle edge cases**: Consider boundary conditions and unexpected inputs

```rust
unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    if input.is_null() {
        error!("Received null input");
        return error_output("null input");
    }
    
    let c_str = CStr::from_ptr((*input).text);
    let input_text = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid UTF-8 in input");
            return error_output("invalid UTF-8");
        }
    };
    
    // Process input...
}
```

### 2. Memory Management

- **Free allocated memory**: Always call `free_output` after using plugin output
- **Use proper C string handling**: Use `CString::into_raw()` and `CString::from_raw()`
- **Avoid memory leaks**: Be careful with string allocations

```rust
unsafe extern "C" fn free_output(output: PluginOutput) {
    if !output.text.is_null() {
        let _ = CString::from_raw(output.text);
    }
}
```

### 3. Performance

- **Keep processing efficient**: Optimize your main processing function
- **Use appropriate data structures**: Choose efficient algorithms and data types
- **Consider async processing**: For long-running operations
- **Profile your code**: Identify and fix performance bottlenecks

### 4. Security

- **Validate all input**: Never trust external data
- **Sanitize output**: Ensure output is safe and properly formatted
- **Use secure defaults**: Implement secure-by-default configurations
- **Handle sensitive data carefully**: Don't log or expose sensitive information

### 5. Documentation

- **Document your plugin**: Clear README with usage examples
- **Include examples**: Provide sample inputs and expected outputs
- **Add inline comments**: Explain complex logic and important decisions
- **Update documentation**: Keep docs in sync with code changes

## Testing & Validation

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use lao_plugin_api::*;
    use std::ffi::CString;

    #[test]
    fn test_plugin_name() {
        unsafe {
            let name_ptr = name();
            let name_cstr = CStr::from_ptr(name_ptr);
            let name_str = name_cstr.to_str().unwrap();
            assert_eq!(name_str, "MyPlugin");
        }
    }

    #[test]
    fn test_validate_input() {
        unsafe {
            let valid_input = CString::new("valid input").unwrap();
            let input = PluginInput { text: valid_input.into_raw() };
            assert!(validate_input(&input));
            
            let invalid_input = CString::new("").unwrap();
            let input = PluginInput { text: invalid_input.into_raw() };
            assert!(!validate_input(&input));
        }
    }

    #[test]
    fn test_plugin_run() {
        unsafe {
            let input_text = CString::new("test input").unwrap();
            let input = PluginInput { text: input_text.into_raw() };
            
            let output = run(&input);
            let output_cstr = CStr::from_ptr(output.text);
            let output_str = output_cstr.to_str().unwrap();
            
            assert_eq!(output_str, "Processed: test input");
            
            free_output(output);
        }
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_plugin_integration() {
        // Test plugin with LAO core
        let registry = PluginRegistry::new();
        // Add integration tests...
    }
}
```

### Manual Testing

```bash
# Build and test your plugin
cargo build --release
cargo test

# Test with LAO CLI
lao plugin test my-plugin --input "test input"

# Test in workflow
lao run workflow.yaml
```

### Validation

```bash
# Validate plugin compatibility
lao plugin validate my-plugin

# Check plugin metadata
lao plugin list --detailed

# Test plugin capabilities
lao plugin test my-plugin --capability process
```

## Publishing & Distribution

### Preparing Your Plugin

1. **Ensure quality**: All tests pass, code is well-documented
2. **Update metadata**: Version, description, dependencies
3. **Create release**: Tag your repository with version
4. **Build binaries**: Create release builds for target platforms
5. **Write release notes**: Document changes and new features

### Publishing to Registry

#### Option 1: Using CLI

```bash
# Publish to local registry
lao plugin publish my-plugin --registry http://localhost:8080

# Publish to remote registry
lao plugin publish my-plugin --registry https://registry.lao.dev
```

#### Option 2: Manual Upload

```bash
# Upload plugin metadata
curl -X POST http://localhost:8080/plugins \
  -H "Content-Type: application/json" \
  -d @plugin-metadata.json

# Upload plugin binary
curl -X POST http://localhost:8080/plugins/{id}/upload \
  -F "file=@my-plugin.dll"
```

### Plugin Manifest (plugin.yaml)

```yaml
name: "MyAwesomePlugin"
version: "1.0.0"
description: "A plugin that does awesome things"
author: "Jake Abendroth <contact@jakea.net>"
license: "MIT"
repository: "https://github.com/yourusername/my-awesome-plugin"
tags: ["ai", "processing", "awesome"]
capabilities:
  - name: "process"
    description: "Process input data"
    input_type: "text"
    output_type: "text"
dependencies: []
compatible_core: "0.1.0"
```

## Advanced Topics

### Plugin Dependencies

Plugins can declare dependencies on other plugins:

```rust
let dependencies = vec![
    PluginDependency {
        name: "ollama".to_string(),
        version: "0.1.0".to_string(),
        optional: false,
    },
    PluginDependency {
        name: "cache".to_string(),
        version: "0.2.0".to_string(),
        optional: true,
    }
];
```

### Custom Capabilities

Define custom capabilities for your plugin:

```rust
let capabilities = vec![
    PluginCapability {
        name: "summarize".to_string(),
        description: "Summarize text using AI".to_string(),
        input_type: PluginInputType::Text,
        output_type: PluginOutputType::Text,
    },
    PluginCapability {
        name: "translate".to_string(),
        description: "Translate text between languages".to_string(),
        input_type: PluginInputType::Text,
        output_type: PluginInputType::Text,
    }
];
```

### Schema Validation

Define input and output schemas for validation:

```rust
let input_schema = r#"{
    "type": "object",
    "properties": {
        "text": {"type": "string"},
        "language": {"type": "string"}
    },
    "required": ["text"]
}"#;

let output_schema = r#"{
    "type": "object",
    "properties": {
        "result": {"type": "string"},
        "confidence": {"type": "number"}
    }
}"#;
```

### Async Processing

For long-running operations, consider async processing:

```rust
use tokio::runtime::Runtime;

fn process_input_async(input: &str) -> Result<String> {
    let rt = Runtime::new()?;
    rt.block_on(async {
        // Async processing here
        let result = async_process(input).await?;
        Ok(result)
    })
}
```

## Examples

### Simple Echo Plugin

```rust
fn process_input(input: &str) -> Result<String> {
    Ok(format!("Echo: {}", input))
}
```

### AI Model Integration

```rust
fn process_input(input: &str) -> Result<String> {
    // Call AI model API
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "llama2",
            "prompt": input,
            "stream": false
        }))
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    Ok(result["response"].as_str().unwrap_or("").to_string())
}
```

### Data Transformation

```rust
fn process_input(input: &str) -> Result<String> {
    // Parse JSON input
    let data: serde_json::Value = serde_json::from_str(input)?;
    
    // Transform data
    let transformed = transform_data(data)?;
    
    // Return JSON output
    Ok(serde_json::to_string(&transformed)?)
}
```

### Image Processing

```rust
fn process_input(input: &str) -> Result<String> {
    // Decode base64 image
    let image_data = base64::decode(input)?;
    
    // Process image
    let processed = process_image(&image_data)?;
    
    // Return base64 encoded result
    Ok(base64::encode(processed))
}
```

## Troubleshooting

### Common Issues

#### Plugin Not Loading

```bash
# Check plugin file exists
ls -la plugins/my-plugin.dll

# Check plugin metadata
lao plugin list --detailed

# Check for errors in LAO logs
lao run --verbose workflow.yaml
```

#### Build Errors

```bash
# Check Rust version
rustc --version

# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build
```

#### Runtime Errors

```bash
# Enable debug logging
RUST_LOG=debug lao run workflow.yaml

# Test plugin directly
lao plugin test my-plugin --input "test"

# Check plugin validation
lao plugin validate my-plugin
```

### Debugging Tips

1. **Use logging**: Add `log::info!()` statements to track execution
2. **Test incrementally**: Test each function separately
3. **Check memory**: Ensure proper memory management
4. **Validate input**: Add input validation early
5. **Use debug builds**: Build with `cargo build` for better error messages

### Getting Help

- **Documentation**: Check this guide and LAO documentation
- **Examples**: Look at existing plugins for reference
- **Community**: Join LAO community discussions
- **Issues**: Report bugs on GitHub with detailed information

## Conclusion

This guide covers the essential aspects of LAO plugin development. By following these practices and using the provided tools, you can create high-quality plugins that integrate seamlessly with the LAO ecosystem.

Remember to:
- Write clear, well-documented code
- Test thoroughly before publishing
- Follow security best practices
- Contribute to the community
- Keep your plugins updated

Happy plugin development! 🚀 