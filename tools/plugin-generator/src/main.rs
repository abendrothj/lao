use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use dialoguer::{Input, Select, Confirm};
use console::style;

#[derive(Parser)]
#[command(name = "lao-plugin-generator")]
#[command(about = "Generate LAO plugin templates")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new plugin from template
    Create {
        /// Plugin name
        #[arg(value_name = "NAME")]
        name: Option<String>,
        
        /// Template to use
        #[arg(short, long, default_value = "basic")]
        template: String,
        
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Skip interactive prompts
        #[arg(short, long)]
        non_interactive: bool,
    },
    
    /// List available templates
    Templates,
    
    /// Initialize plugin in current directory
    Init {
        /// Skip interactive prompts
        #[arg(short, long)]
        non_interactive: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginConfig {
    name: String,
    version: String,
    description: String,
    author: String,
    email: String,
    license: String,
    repository: String,
    tags: Vec<String>,
    capabilities: Vec<Capability>,
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Capability {
    name: String,
    description: String,
    input_type: String,
    output_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Dependency {
    name: String,
    version: String,
    optional: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "0.1.0".to_string(),
            description: String::new(),
            author: String::new(),
            email: String::new(),
            license: "MIT".to_string(),
            repository: String::new(),
            tags: vec![],
            capabilities: vec![],
            dependencies: vec![],
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Create { name, template, output, non_interactive } => {
            let plugin_name = if let Some(n) = name {
                n.clone()
            } else if *non_interactive {
                eprintln!("Error: Plugin name is required in non-interactive mode");
                process::exit(1);
            } else {
                Input::new()
                    .with_prompt("Plugin name")
                    .interact_text()?
            };

            let output_dir = if let Some(o) = output {
                o.clone()
            } else {
                PathBuf::from(&plugin_name)
            };

            create_plugin(&plugin_name, template, &output_dir, *non_interactive)?;
        }
        
        Commands::Templates => {
            list_templates()?;
        }
        
        Commands::Init { non_interactive } => {
            init_plugin_in_current_dir(*non_interactive)?;
        }
    }

    Ok(())
}

fn create_plugin(name: &str, template: &str, output_dir: &Path, non_interactive: bool) -> Result<()> {
    println!("{}", style("🚀 Creating LAO Plugin").bold().green());
    println!("Name: {}", style(name).bold());
    println!("Template: {}", style(template).bold());
    println!("Output: {}", style(output_dir.display()).bold());
    println!();

    // Validate plugin name
    if !is_valid_plugin_name(name) {
        eprintln!("{}", style("Error: Invalid plugin name").red());
        eprintln!("Plugin names must be lowercase, use hyphens for separators");
        process::exit(1);
    }

    // Create output directory
    if output_dir.exists() {
        if !non_interactive {
            let overwrite = Confirm::new()
                .with_prompt("Directory already exists. Overwrite?")
                .interact()?;
            
            if !overwrite {
                println!("{}", style("Plugin creation cancelled").yellow());
                return Ok(());
            }
        }
        
        fs::remove_dir_all(output_dir)?;
    }
    
    fs::create_dir_all(output_dir)?;

    // Get plugin configuration
    let config = if non_interactive {
        get_default_config(name)
    } else {
        get_interactive_config(name)?
    };

    // Copy template files
    copy_template_files(template, output_dir, &config)?;

    // Generate files
    generate_cargo_toml(output_dir, &config)?;
    generate_lib_rs(output_dir, &config)?;
    generate_plugin_yaml(output_dir, &config)?;
    generate_readme(output_dir, &config)?;
    generate_examples(output_dir, &config)?;
    generate_tests(output_dir, &config)?;

    println!("{}", style("✅ Plugin created successfully!").bold().green());
    println!();
    println!("Next steps:");
    println!("  cd {}", output_dir.display());
    println!("  cargo build");
    println!("  cargo test");
    println!();
    println!("For more information, see the README.md file.");

    Ok(())
}

fn list_templates() -> Result<()> {
    println!("{}", style("Available Templates").bold().blue());
    println!();
    
    let templates = vec![
        ("basic", "Basic plugin template with minimal functionality"),
        ("ai-model", "AI model integration template"),
        ("data-processor", "Data processing and transformation template"),
        ("api-client", "API client integration template"),
        ("image-processor", "Image processing template"),
        ("web-scraper", "Web scraping template"),
    ];

    for (name, description) in templates {
        println!("  {} - {}", style(name).bold(), description);
    }

    Ok(())
}

fn init_plugin_in_current_dir(non_interactive: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let plugin_name = current_dir.file_name()
        .and_then(|n| n.to_str())
        .context("Could not determine plugin name from current directory")?;

    println!("{}", style("🔧 Initializing plugin in current directory").bold().green());
    println!("Plugin name: {}", style(plugin_name).bold());
    println!();

    let config = if non_interactive {
        get_default_config(plugin_name)
    } else {
        get_interactive_config(plugin_name)?
    };

    // Generate files in current directory
    generate_cargo_toml(&current_dir, &config)?;
    generate_lib_rs(&current_dir, &config)?;
    generate_plugin_yaml(&current_dir, &config)?;
    generate_readme(&current_dir, &config)?;
    generate_examples(&current_dir, &config)?;
    generate_tests(&current_dir, &config)?;

    println!("{}", style("✅ Plugin initialized successfully!").bold().green());

    Ok(())
}

fn is_valid_plugin_name(name: &str) -> bool {
    name.chars().all(|c| c.is_alphanumeric() || c == '-') && 
    !name.starts_with('-') && 
    !name.ends_with('-') &&
    !name.is_empty()
}

fn get_default_config(name: &str) -> PluginConfig {
    PluginConfig {
        name: name.to_string(),
        version: "0.1.0".to_string(),
        description: format!("A LAO plugin for {}", name.replace('-', " ")),
        author: "Your Name".to_string(),
        email: "your.email@example.com".to_string(),
        license: "MIT".to_string(),
        repository: format!("https://github.com/yourusername/{}", name),
        tags: vec!["plugin".to_string(), "lao".to_string()],
        capabilities: vec![
            Capability {
                name: "process".to_string(),
                description: "Process input data".to_string(),
                input_type: "text".to_string(),
                output_type: "text".to_string(),
            }
        ],
        dependencies: vec![],
    }
}

fn get_interactive_config(name: &str) -> Result<PluginConfig> {
    let mut config = get_default_config(name);

    config.description = Input::new()
        .with_prompt("Plugin description")
        .with_initial_text(&config.description)
        .interact_text()?;

    config.author = Input::new()
        .with_prompt("Author name")
        .with_initial_text(&config.author)
        .interact_text()?;

    config.email = Input::new()
        .with_prompt("Author email")
        .with_initial_text(&config.email)
        .interact_text()?;

    config.repository = Input::new()
        .with_prompt("Repository URL")
        .with_initial_text(&config.repository)
        .interact_text()?;

    let license_options = vec!["MIT", "Apache-2.0", "GPL-3.0", "BSD-3-Clause"];
    let license_selection = Select::new()
        .with_prompt("Choose license")
        .items(&license_options)
        .default(0)
        .interact()?;
    config.license = license_options[license_selection].to_string();

    let add_tags = Confirm::new()
        .with_prompt("Add custom tags?")
        .interact()?;

    if add_tags {
        let tags_input: String = Input::new()
            .with_prompt("Tags (comma-separated)")
            .interact_text()?;
        config.tags = tags_input.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    let add_capabilities = Confirm::new()
        .with_prompt("Add custom capabilities?")
        .interact()?;

    if add_capabilities {
        config.capabilities = vec![];
        loop {
            let cap_name: String = Input::new()
                .with_prompt("Capability name")
                .interact_text()?;
            
            let cap_desc: String = Input::new()
                .with_prompt("Capability description")
                .interact_text()?;

            let input_types = vec!["text", "json", "binary", "any"];
            let input_selection = Select::new()
                .with_prompt("Input type")
                .items(&input_types)
                .default(0)
                .interact()?;

            let output_types = vec!["text", "json", "binary", "any"];
            let output_selection = Select::new()
                .with_prompt("Output type")
                .items(&output_types)
                .default(0)
                .interact()?;

            config.capabilities.push(Capability {
                name: cap_name,
                description: cap_desc,
                input_type: input_types[input_selection].to_string(),
                output_type: output_types[output_selection].to_string(),
            });

            let add_more = Confirm::new()
                .with_prompt("Add another capability?")
                .interact()?;

            if !add_more {
                break;
            }
        }
    }

    Ok(config)
}

fn copy_template_files(template: &str, output_dir: &Path, config: &PluginConfig) -> Result<()> {
    // For now, we'll generate all files from scratch
    // In the future, this could copy from template directories
    Ok(())
}

fn generate_cargo_toml(output_dir: &Path, config: &PluginConfig) -> Result<()> {
    let cargo_content = format!(
        r#"[package]
name = "{}"
version = "{}"
edition = "2021"
description = "{}"
authors = ["{} <{}>"]
license = "{}"
repository = "{}"
keywords = ["lao", "plugin", "{}"]
categories = ["api-bindings", "development-tools"]

[dependencies]
lao_plugin_api = {{ path = "../../lao_plugin_api" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
serde_yaml = "0.9"
anyhow = "1.0"
thiserror = "1.0"
log = "0.4"
env_logger = "0.10"

[lib]
crate-type = ["cdylib"]

[features]
default = []
test-utils = ["tokio", "tempfile"]

[dev-dependencies]
tokio = {{ version = "1.0", features = ["full"] }}
tempfile = "3.0"
"#,
        config.name,
        config.version,
        config.description,
        config.author,
        config.email,
        config.license,
        config.repository,
        config.name
    );

    fs::write(output_dir.join("Cargo.toml"), cargo_content)?;
    Ok(())
}

fn generate_lib_rs(output_dir: &Path, config: &PluginConfig) -> Result<()> {
    let src_dir = output_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    let lib_content = format!(
        r#"use lao_plugin_api::*;
use std::ffi::{{CStr, CString}};
use std::os::raw::c_char;
use serde::{{Deserialize, Serialize}};
use anyhow::Result;
use log::{{info, warn, error}};

// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {{
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    pub capabilities: Vec<PluginCapability>,
    pub dependencies: Vec<PluginDependency>,
}}

impl Default for PluginConfig {{
    fn default() -> Self {{
        Self {{
            name: "{}".to_string(),
            version: "{}".to_string(),
            description: "{}".to_string(),
            author: "{} <{}>".to_string(),
            tags: vec![{}],
            capabilities: vec![
                {}
            ],
            dependencies: vec![],
        }}
    }}
}}

// Plugin state
static mut PLUGIN_CONFIG: Option<PluginConfig> = None;

// Initialize plugin configuration
fn init_plugin_config() -> &'static PluginConfig {{
    unsafe {{
        if PLUGIN_CONFIG.is_none() {{
            PLUGIN_CONFIG = Some(PluginConfig::default());
        }}
        PLUGIN_CONFIG.as_ref().unwrap()
    }}
}}

// Plugin name function
unsafe extern "C" fn name() -> *const c_char {{
    let config = init_plugin_config();
    let name_cstring = CString::new(config.name.as_str()).unwrap();
    name_cstring.into_raw()
}}

// Plugin run function
unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {{
    if input.is_null() {{
        error!("Received null input");
        let error_msg = CString::new("error: null input").unwrap();
        return PluginOutput {{ text: error_msg.into_raw() }};
    }}

    let c_str = CStr::from_ptr((*input).text);
    let input_text = match c_str.to_str() {{
        Ok(s) => s,
        Err(_) => {{
            error!("Invalid UTF-8 in input");
            let error_msg = CString::new("error: invalid UTF-8 input").unwrap();
            return PluginOutput {{ text: error_msg.into_raw() }};
        }}
    }};

    info!("Processing input: {{}}", input_text);

    // Validate input
    if !validate_input_internal(input_text) {{
        let error_msg = CString::new("error: invalid input format").unwrap();
        return PluginOutput {{ text: error_msg.into_raw() }};
    }}

    // Process input - CUSTOMIZE THIS FOR YOUR PLUGIN!
    let result = match process_input(input_text) {{
        Ok(output) => output,
        Err(e) => {{
            error!("Processing error: {{}}", e);
            format!("error: {{}}", e)
        }}
    }};

    info!("Returning output: {{}}", result);
    let output_cstring = CString::new(result).unwrap();
    PluginOutput {{ text: output_cstring.into_raw() }}
}}

// Free output function
unsafe extern "C" fn free_output(output: PluginOutput) {{
    if !output.text.is_null() {{
        let _ = CString::from_raw(output.text);
    }}
}}

// Run with buffer function
unsafe extern "C" fn run_with_buffer(
    input: *const PluginInput,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {{
    if input.is_null() || buffer.is_null() {{
        return 0;
    }}

    let c_str = CStr::from_ptr((*input).text);
    let input_text = match c_str.to_str() {{
        Ok(s) => s,
        Err(_) => return 0,
    }};

    let result = match process_input(input_text) {{
        Ok(output) => output,
        Err(_) => "error: processing failed".to_string(),
    }};

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
}}

// Get metadata function
unsafe extern "C" fn get_metadata() -> PluginMetadata {{
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
    
    PluginMetadata {{
        name: name_cstring.into_raw(),
        version: version_cstring.into_raw(),
        description: description_cstring.into_raw(),
        author: author_cstring.into_raw(),
        dependencies: deps_cstring.into_raw(),
        tags: tags_cstring.into_raw(),
        input_schema: std::ptr::null(),
        output_schema: std::ptr::null(),
        get_capabilities: caps_cstring.into_raw(),
    }}
}}

// Validate input function
unsafe extern "C" fn validate_input(input: *const PluginInput) -> bool {{
    if input.is_null() {{
        return false;
    }}
    
    let c_str = CStr::from_ptr((*input).text);
    let input_text = match c_str.to_str() {{
        Ok(s) => s,
        Err(_) => return false,
    }};
    
    validate_input_internal(input_text)
}}

// Get capabilities function
unsafe extern "C" fn get_capabilities() -> *const c_char {{
    let config = init_plugin_config();
    let caps_json = serde_json::to_string(&config.capabilities).unwrap_or_default();
    let caps_cstring = CString::new(caps_json).unwrap();
    caps_cstring.into_raw()
}}

// Internal validation function - CUSTOMIZE THIS!
fn validate_input_internal(input: &str) -> bool {{
    !input.trim().is_empty()
}}

// Internal processing function - CUSTOMIZE THIS FOR YOUR PLUGIN!
fn process_input(input: &str) -> Result<String> {{
    // This is where you implement your plugin's main functionality
    // For example:
    // - Call an AI model
    // - Process images
    // - Transform data
    // - Make API calls
    // - etc.
    
    let processed = format!("Processed: {{}}", input);
    Ok(processed)
}}

// Plugin vtable - REQUIRED!
#[no_mangle]
pub static plugin_vtable: PluginVTable = PluginVTable {{
    version: 1,
    name,
    run,
    free_output,
    run_with_buffer,
    get_metadata,
    validate_input,
    get_capabilities,
}};

// Test module
#[cfg(test)]
mod tests {{
    use super::*;
    use lao_plugin_api::*;
    use std::ffi::CString;

    #[test]
    fn test_plugin_name() {{
        unsafe {{
            let name_ptr = name();
            let name_cstr = CStr::from_ptr(name_ptr);
            let name_str = name_cstr.to_str().unwrap();
            assert_eq!(name_str, "{}");
        }}
    }}

    #[test]
    fn test_validate_input() {{
        unsafe {{
            let valid_input = CString::new("valid input").unwrap();
            let input = PluginInput {{ text: valid_input.into_raw() }};
            assert!(validate_input(&input));
            
            let invalid_input = CString::new("").unwrap();
            let input = PluginInput {{ text: invalid_input.into_raw() }};
            assert!(!validate_input(&input));
        }}
    }}

    #[test]
    fn test_process_input() {{
        let result = process_input("test input").unwrap();
        assert_eq!(result, "Processed: test input");
    }}

    #[test]
    fn test_plugin_run() {{
        unsafe {{
            let input_text = CString::new("test input").unwrap();
            let input = PluginInput {{ text: input_text.into_raw() }};
            
            let output = run(&input);
            let output_cstr = CStr::from_ptr(output.text);
            let output_str = output_cstr.to_str().unwrap();
            
            assert_eq!(output_str, "Processed: test input");
            
            free_output(output);
        }}
    }}
}}
"#,
        config.name,
        config.version,
        config.description,
        config.author,
        config.email,
        config.tags.iter().map(|t| format!("\"{}\".to_string()", t)).collect::<Vec<_>>().join(", "),
        config.capabilities.iter().map(|c| format!(
            "PluginCapability {{\n                name: \"{}\".to_string(),\n                description: \"{}\".to_string(),\n                input_type: PluginInputType::{},\n                output_type: PluginOutputType::{},\n            }}",
            c.name,
            c.description,
            c.input_type.to_uppercase(),
            c.output_type.to_uppercase()
        )).collect::<Vec<_>>().join(",\n                "),
        config.name
    );

    fs::write(src_dir.join("lib.rs"), lib_content)?;
    Ok(())
}

fn generate_plugin_yaml(output_dir: &Path, config: &PluginConfig) -> Result<()> {
    let yaml_content = format!(
        r#"name: "{}"
version: "{}"
description: "{}"
author: "{} <{}>"
license: "{}"
repository: "{}"
tags: [{}]
capabilities:
{}
dependencies: []
compatible_core: "0.1.0"
"#,
        config.name,
        config.version,
        config.description,
        config.author,
        config.email,
        config.license,
        config.repository,
        config.tags.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", "),
        config.capabilities.iter().map(|c| format!(
            "  - name: \"{}\"\n    description: \"{}\"\n    input_type: \"{}\"\n    output_type: \"{}\"",
            c.name, c.description, c.input_type, c.output_type
        )).collect::<Vec<_>>().join("\n")
    );

    fs::write(output_dir.join("plugin.yaml"), yaml_content)?;
    Ok(())
}

fn generate_readme(output_dir: &Path, config: &PluginConfig) -> Result<()> {
    let readme_content = format!(
        r#"# {}

{}

## Description

{}

## Installation

1. Build the plugin:
   ```bash
   cargo build --release
   ```

2. Copy the DLL to your LAO plugins directory:
   ```bash
   cp target/release/{}.dll /path/to/lao/plugins/
   ```

## Usage

### In Workflows

```yaml
steps:
  - run: {}
    input: "Your input data here"
```

### Direct Testing

```bash
lao plugin test {} --input "test input"
```

## Configuration

This plugin supports the following configuration options:

- None currently

## Capabilities

{}

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running Tests

```bash
cargo test
```

## License

{}

## Author

{} <{}>

## Repository

{}
"#,
        config.name,
        config.description,
        config.description,
        config.name,
        config.name,
        config.capabilities.iter().map(|c| format!(
            "- **{}**: {} ({} → {})",
            c.name, c.description, c.input_type, c.output_type
        )).collect::<Vec<_>>().join("\n"),
        config.license,
        config.author,
        config.email,
        config.repository
    );

    fs::write(output_dir.join("README.md"), readme_content)?;
    Ok(())
}

fn generate_examples(output_dir: &Path, config: &PluginConfig) -> Result<()> {
    let examples_dir = output_dir.join("examples");
    fs::create_dir_all(&examples_dir)?;

    let example_content = format!(
        r#"# Example Input for {}

This file contains example input data for testing the {} plugin.

## Basic Example

```
Your input data here
```

## JSON Example

```json
{{
  "input": "Your input data here",
  "options": {{
    "option1": "value1",
    "option2": "value2"
  }}
}}
```

## Usage in Workflow

```yaml
workflow: "Example Workflow"
steps:
  - run: {}
    input: "Your input data here"
    retry:
      max_attempts: 3
      delay: 1000
```
"#,
        config.name,
        config.name,
        config.name
    );

    fs::write(examples_dir.join("sample_input.txt"), example_content)?;
    Ok(())
}

fn generate_tests(output_dir: &Path, config: &PluginConfig) -> Result<()> {
    let tests_dir = output_dir.join("tests");
    fs::create_dir_all(&tests_dir)?;

    let test_content = format!(
        r#"use lao_plugin_api::*;
use std::ffi::CString;

#[test]
fn test_basic_functionality() {{
    // Test basic plugin functionality
    let input = "test input";
    let expected = "Processed: test input";
    
    // This is a placeholder test - implement actual testing logic
    assert!(input.len() > 0);
    assert!(expected.contains(input));
}}

#[test]
fn test_error_handling() {{
    // Test error conditions
    let empty_input = "";
    
    // This is a placeholder test - implement actual error testing
    assert!(empty_input.is_empty());
}}

#[test]
fn test_edge_cases() {{
    // Test edge cases and boundary conditions
    let long_input = "a".repeat(1000);
    
    // This is a placeholder test - implement actual edge case testing
    assert_eq!(long_input.len(), 1000);
}}
"#,
        config.name
    );

    fs::write(tests_dir.join("integration_tests.rs"), test_content)?;
    Ok(())
} 