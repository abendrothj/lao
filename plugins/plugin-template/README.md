# LAO Plugin Development Guide

Welcome to the LAO (Local AI Orchestrator) plugin development guide! This guide will help you create powerful plugins that extend LAO's capabilities.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Plugin Architecture](#plugin-architecture)
3. [Creating Your First Plugin](#creating-your-first-plugin)
4. [Plugin API Reference](#plugin-api-reference)
5. [Best Practices](#best-practices)
6. [Testing Your Plugin](#testing-your-plugin)
7. [Publishing Your Plugin](#publishing-your-plugin)
8. [Examples](#examples)

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- LAO core library
- Basic understanding of Rust

### Creating a New Plugin

```bash
# Use the LAO CLI to create a new plugin
lao plugin create my-awesome-plugin --template basic

# Or manually copy this template
cp -r plugin-template my-awesome-plugin
cd my-awesome-plugin
```

### Building Your Plugin

```bash
# Build in debug mode
cargo build

# Build in release mode
   cargo build --release

# Test your plugin
cargo test
```

## Plugin Architecture

LAO plugins are dynamic libraries that implement a specific interface. Each plugin:

- Exports a `plugin_vtable` symbol
- Implements required functions (name, run, validate_input, etc.)
- Provides metadata about capabilities and dependencies
- Can be loaded and unloaded at runtime

### Plugin Lifecycle

1. **Discovery**: LAO scans plugin directories for `.dll` files
2. **Loading**: Plugin library is loaded into memory
3. **Validation**: Plugin metadata and capabilities are verified
4. **Registration**: Plugin is registered in the plugin registry
5. **Execution**: Plugin functions are called during workflow execution
6. **Cleanup**: Plugin resources are freed when no longer needed

## Creating Your First Plugin

### 1. Plugin Structure

```
my-awesome-plugin/
├── Cargo.toml          # Plugin dependencies and metadata
├── src/
│   └── lib.rs          # Main plugin implementation
├── plugin.yaml         # Plugin manifest
├── examples/           # Example usage
├── tests/              # Plugin tests
└── README.md           # Plugin documentation
```

### 2. Plugin Manifest (plugin.yaml)

```yaml
name: "MyAwesomePlugin"
version: "0.1.0"
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

### 3. Plugin Implementation

```rust
use lao_plugin_api::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use log::{info, warn, error};

// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    pub capabilities: Vec<PluginCapability>,
    pub dependencies: Vec<PluginDependency>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            name: "MyAwesomePlugin".to_string(),
            version: "0.1.0".to_string(),
            description: "A plugin that does awesome things".to_string(),
            author: "Jake Abendroth <contact@jakea.net>".to_string(),
            tags: vec!["ai".to_string(), "processing".to_string()],
            capabilities: vec![
                PluginCapability {
                    name: "process".to_string(),
                    description: "Process input data".to_string(),
                    input_type: PluginInputType::Text,
                    output_type: PluginOutputType::Text,
                }
            ],
            dependencies: vec![],
        }
    }
}

// Plugin state
static mut PLUGIN_CONFIG: Option<PluginConfig> = None;

// Initialize plugin configuration
fn init_plugin_config() -> &'static PluginConfig {
    unsafe {
        if PLUGIN_CONFIG.is_none() {
            PLUGIN_CONFIG = Some(PluginConfig::default());
        }
        PLUGIN_CONFIG.as_ref().unwrap()
    }
}

// Plugin name function
unsafe extern "C" fn name() -> *const c_char {
    let config = init_plugin_config();
    let name_cstring = CString::new(config.name.as_str()).unwrap();
    name_cstring.into_raw()
}

// Plugin run function - This is where your main logic goes!
unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    if input.is_null() {
        error!("Received null input");
        let error_msg = CString::new("error: null input").unwrap();
        return PluginOutput { text: error_msg.into_raw() };
    }

    let c_str = CStr::from_ptr((*input).text);
    let input_text = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid UTF-8 in input");
            let error_msg = CString::new("error: invalid UTF-8 input").unwrap();
            return PluginOutput { text: error_msg.into_raw() };
        }
    };

    info!("Processing input: {}", input_text);

    // Validate input
    if !validate_input_internal(input_text) {
        let error_msg = CString::new("error: invalid input format").unwrap();
        return PluginOutput { text: error_msg.into_raw() };
    }

    // Process input - CUSTOMIZE THIS FOR YOUR PLUGIN!
    let result = match process_input(input_text) {
        Ok(output) => output,
        Err(e) => {
            error!("Processing error: {}", e);
            format!("error: {}", e)
        }
    };

    info!("Returning output: {}", result);
    let output_cstring = CString::new(result).unwrap();
    PluginOutput { text: output_cstring.into_raw() }
}

// Free output function
unsafe extern "C" fn free_output(output: PluginOutput) {
    if !output.text.is_null() {
        let _ = CString::from_raw(output.text);
    }
}

// Run with buffer function
unsafe extern "C" fn run_with_buffer(
    input: *const PluginInput,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if input.is_null() || buffer.is_null() {
        return 0;
    }

    let c_str = CStr::from_ptr((*input).text);
    let input_text = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let result = match process_input(input_text) {
        Ok(output) => output,
        Err(_) => "error: processing failed".to_string(),
    };

    let result_bytes = result.as_bytes();
    let copy_len = std::cmp::min(result_bytes.len(), buffer_len - 1);
    
    std::ptr::copy_nonoverlapping(
        result_bytes.as_ptr(),
        buffer as *mut u8,
        copy_len,
    );
    
    // Null terminate
    *buffer.add(copy_len) = 0;
    
    copy_len
}

// Get metadata function
unsafe extern "C" fn get_metadata() -> PluginMetadata {
    let config = init_plugin_config();
    
    let name_cstring = CString::new(config.name.as_str()).unwrap();
    let version_cstring = CString::new(config.version.as_str()).unwrap();
    let description_cstring = CString::new(config.description.as_str()).unwrap();
    let author_cstring = CString::new(config.author.as_str()).unwrap();
    
    let tags_json = serde_json::to_string(&config.tags).unwrap_or_default();
    let tags_cstring = CString::new(tags_json).unwrap();
    
    let deps_json = serde_json::to_string(&config.dependencies).unwrap_or_default();
    let deps_cstring = CString::new(deps_json).unwrap();
    
    let caps_json = serde_json::to_string(&config.capabilities).unwrap_or_default();
    let caps_cstring = CString::new(caps_json).unwrap();
    
    PluginMetadata {
        name: name_cstring.into_raw(),
        version: version_cstring.into_raw(),
        description: description_cstring.into_raw(),
        author: author_cstring.into_raw(),
        dependencies: deps_cstring.into_raw(),
        tags: tags_cstring.into_raw(),
        input_schema: std::ptr::null(),
        output_schema: std::ptr::null(),
        get_capabilities: caps_cstring.into_raw(),
    }
}

// Validate input function
unsafe extern "C" fn validate_input(input: *const PluginInput) -> bool {
    if input.is_null() {
        return false;
    }
    
    let c_str = CStr::from_ptr((*input).text);
    let input_text = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    validate_input_internal(input_text)
}

// Get capabilities function
unsafe extern "C" fn get_capabilities() -> *const c_char {
    let config = init_plugin_config();
    let caps_json = serde_json::to_string(&config.capabilities).unwrap_or_default();
    let caps_cstring = CString::new(caps_json).unwrap();
    caps_cstring.into_raw()
}

// Internal validation function - CUSTOMIZE THIS!
fn validate_input_internal(input: &str) -> bool {
    !input.trim().is_empty()
}

// Internal processing function - CUSTOMIZE THIS FOR YOUR PLUGIN!
fn process_input(input: &str) -> Result<String> {
    // This is where you implement your plugin's main functionality
    // For example:
    // - Call an AI model
    // - Process images
    // - Transform data
    // - Make API calls
    // - etc.
    
    let processed = format!("Processed: {}", input);
    Ok(processed)
}

// Plugin vtable - REQUIRED!
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

// Test module
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
            assert_eq!(name_str, "MyAwesomePlugin");
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
    fn test_process_input() {
        let result = process_input("test input").unwrap();
        assert_eq!(result, "Processed: test input");
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

## Plugin API Reference

### Required Functions

#### `name() -> *const c_char`
Returns the plugin's name as a C string.

#### `run(input: *const PluginInput) -> PluginOutput`
Main plugin execution function. Processes input and returns output.

#### `free_output(output: PluginOutput)`
Frees memory allocated for plugin output.

#### `run_with_buffer(input: *const PluginInput, buffer: *mut c_char, buffer_len: usize) -> usize`
Alternative execution function that writes output to a provided buffer.

#### `get_metadata() -> PluginMetadata`
Returns plugin metadata including name, version, description, etc.

#### `validate_input(input: *const PluginInput) -> bool`
Validates plugin input before processing.

#### `get_capabilities() -> *const c_char`
Returns JSON string describing plugin capabilities.

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

## Best Practices

### 1. Error Handling
- Always validate input before processing
- Return meaningful error messages
- Use proper logging for debugging
- Handle null pointers safely

### 2. Memory Management
- Free allocated memory in `free_output`
- Use `CString::into_raw()` and `CString::from_raw()` properly
- Avoid memory leaks

### 3. Performance
- Keep processing functions efficient
- Use appropriate data structures
- Consider async processing for long-running operations

### 4. Security
- Validate all input data
- Sanitize output data
- Don't trust external data sources
- Use secure defaults

### 5. Documentation
- Document your plugin's purpose and usage
- Provide clear examples
- Include test cases
- Update README with installation instructions

## Testing Your Plugin

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Test your plugin's core functionality
    }

    #[test]
    fn test_error_handling() {
        // Test error conditions
    }

    #[test]
    fn test_edge_cases() {
        // Test edge cases and boundary conditions
    }
}
```

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_plugin_integration() {
        // Test plugin with LAO core
    }
}
```

### Manual Testing
```bash
# Build your plugin
cargo build --release

# Copy to LAO plugins directory
cp target/release/my_awesome_plugin.dll /path/to/lao/plugins/

# Test with LAO CLI
lao plugin test my-awesome-plugin --input "test input"
```

## Publishing Your Plugin

### 1. Prepare Your Plugin
- Ensure all tests pass
- Update documentation
- Add proper licensing
- Include examples

### 2. Create a Release
- Tag your repository with a version
- Create release notes
- Upload compiled binaries

### 3. Submit to LAO Plugin Registry
- Fork the LAO plugin registry repository
- Add your plugin metadata
- Submit a pull request

### 4. Community Guidelines
- Follow the LAO plugin guidelines
- Respond to issues and feedback
- Maintain your plugin
- Help other developers

## Examples

### Simple Echo Plugin
```rust
fn process_input(input: &str) -> Result<String> {
    Ok(format!("Echo: {}", input))
}
```

### AI Model Plugin
```rust
fn process_input(input: &str) -> Result<String> {
    // Call AI model API
    let response = call_ai_model(input)?;
    Ok(response)
}
```

### Data Transformation Plugin
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

### Image Processing Plugin
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

## Getting Help

- **Documentation**: Check the LAO documentation
- **Issues**: Report bugs on GitHub
- **Discussions**: Join the LAO community
- **Examples**: Look at existing plugins for reference

## Contributing

We welcome contributions! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

Thank you for contributing to LAO! 