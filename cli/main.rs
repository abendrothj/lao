use clap::{Parser, Subcommand};
use lao_orchestrator_core::{run_workflow_yaml, load_workflow_yaml};
use lao_orchestrator_core::plugins::PluginRegistry;
use lao_plugin_api::PluginInput;
use serde::Deserialize;

#[derive(Deserialize)]
struct PromptPair {
    prompt: String,
    workflow: String,
}

fn normalize_yaml(yaml: &str) -> serde_yaml::Value {
    serde_yaml::from_str(yaml).unwrap_or(serde_yaml::Value::Null)
}

fn strip_code_fences(s: &str) -> String {
    s.lines()
        .filter(|line| !line.trim_start().starts_with("```") )
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[derive(Parser)]
#[command(name = "lao")]
#[command(about = "Local AI Orchestrator CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a workflow YAML file
    Run {
        path: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate a workflow YAML file (type & plugin availability)
    Validate {
        path: String,
    },
    /// List available plugins
    PluginList,
    /// Scaffold a new workflow YAML template
    NewWorkflow {
        name: String,
        #[arg(long, help = "Output file path (default: workflows/<name>.yaml)")]
        output: Option<String>,
    },
    /// Generate and run a workflow from a prompt
    Prompt {
        prompt: String,
        #[arg(long, help = "Output file path (default: workflows/generated_from_prompt.yaml)")]
        output: Option<String>,
    },
    /// Validate prompt-to-workflow generation using the prompt library
    ValidatePrompts {
        #[arg(long, default_value = "core/prompt_dispatcher/prompt/prompt_library.json")]
        path: String,
        #[arg(long)]
        fail_fast: bool,
        #[arg(long)]
        verbose: bool,
    },
    /// List all saved workflows in the workflows/ directory
    ListWorkflows,
    /// View a workflow YAML file by name (from workflows/ directory)
    ViewWorkflow {
        name: String,
    },
    /// Delete a workflow YAML file by name (from workflows/ directory)
    DeleteWorkflow {
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { path, dry_run } => {
            if dry_run {
                match load_workflow_yaml(&path) {
                    Ok(workflow) => {
                        let plugin_registry = PluginRegistry::dynamic_registry("plugins/");
                        println!("[DRY RUN] Workflow: {}", workflow.workflow);
                        for (i, step) in workflow.steps.iter().enumerate() {
                            let plugin = plugin_registry.plugins.get(&step.run);
                            println!("Step {}: {}", i + 1, step.run);
                            match plugin {
                                Some(_p) => {
                                    println!("  [OK] Plugin '{}' loaded.", step.run);
                                }
                                None => {
                                    println!("  [ERROR] Plugin '{}' not found!", step.run);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[DRY RUN] Failed to load workflow: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                match run_workflow_yaml(&path) {
                    Ok(results) => {
                        println!("Workflow executed successfully. Step outputs:");
                        for (i, output) in results.iter().enumerate() {
                            println!("Step {}: {:?}", i + 1, output);
                        }
                    }
                    Err(e) => {
                        eprintln!("Workflow execution failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Validate { path } => {
            match load_workflow_yaml(&path) {
                Ok(workflow) => {
                    let plugin_registry = PluginRegistry::dynamic_registry("plugins/");
                    let dag = lao_orchestrator_core::build_dag(&workflow.steps).unwrap();
                    let errors = lao_orchestrator_core::validate_workflow_types(&dag, &plugin_registry);
                    if errors.is_empty() {
                        println!("Validation passed: all steps and plugins available.");
                    } else {
                        for (step, msg) in errors {
                            println!("Step {}: {}", step, msg);
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load workflow: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::PluginList => {
            let plugin_registry = PluginRegistry::dynamic_registry("plugins/");
            println!("Available plugins:");
            for (name, _plugin) in &plugin_registry.plugins {
                println!("- {}", name);
            }
        }
        Commands::NewWorkflow { name, output } => {
            let path = output.unwrap_or_else(|| format!("workflows/{}.yaml", name));
            let template = format!(
                "workflow: \"{}\"\nsteps:\n  - run: Whisper\n    input: audio.wav\n    retry_count: 2\n    retry_delay: 1000\n    cache_key: \"whisper_{}\"\n  - run: Ollama\n    input_from: Whisper\n    cache_key: \"summary_{}\"\n",
                name, name, name
            );
            if let Some(parent) = std::path::Path::new(&path).parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("[ERROR] Failed to create directory {}: {}", parent.display(), e);
                    std::process::exit(1);
                }
            }
            if let Err(e) = std::fs::write(&path, template) {
                eprintln!("[ERROR] Failed to write workflow file {}: {}", path, e);
                std::process::exit(1);
            }
            println!("Scaffolded new workflow at {}", path);
        }
        Commands::Prompt { prompt, output } => {
            // Use the PromptDispatcherPlugin to generate a workflow YAML
            let mut registry = PluginRegistry::dynamic_registry("plugins/");
            let dispatcher = registry.plugins.get("PromptDispatcher").expect("PromptDispatcherPlugin not found");
            // Call the run function via the vtable
            use std::ffi::CString;
            let c_prompt = CString::new(prompt.clone()).unwrap();
            let input = PluginInput { text: c_prompt.as_ptr() };
            let output_obj = unsafe { ((*dispatcher.vtable).run)(&input) };
            let c_str = unsafe { std::ffi::CStr::from_ptr(output_obj.text) };
            let yaml = c_str.to_string_lossy().to_string();
            unsafe { ((*dispatcher.vtable).free_output)(output_obj) };
            println!("Generated workflow:\n{}", yaml);
            let clean_yaml = strip_code_fences(&yaml);
            match serde_yaml::from_str::<lao_orchestrator_core::Workflow>(&clean_yaml) {
                Ok(_workflow) => {
                    let out_path = output.unwrap_or_else(|| "workflows/generated_from_prompt.yaml".to_string());
                    if let Some(parent) = std::path::Path::new(&out_path).parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            eprintln!("[ERROR] Failed to create directory {}: {}", parent.display(), e);
                            std::process::exit(1);
                        }
                    }
                    if let Err(e) = std::fs::write(&out_path, &clean_yaml) {
                        eprintln!("[ERROR] Failed to write workflow file {}: {}", out_path, e);
                        std::process::exit(1);
                    }
                    println!("Workflow saved to {}", out_path);
                }
                Err(e) => {
                    eprintln!("Failed to parse generated workflow YAML: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::ValidatePrompts { path, fail_fast, verbose } => {
            // Load prompt pairs from the prompt library JSON
            let prompt_pairs: Vec<PromptPair> = {
                let data = std::fs::read_to_string(&path).expect("Failed to read prompt library");
                serde_json::from_str(&data).expect("Failed to parse prompt library JSON")
            };
            let mut registry = PluginRegistry::dynamic_registry("plugins/");
            let dispatcher = registry.plugins.get("PromptDispatcher").expect("PromptDispatcherPlugin not found");
            let mut failures = 0;
            for (i, pair) in prompt_pairs.iter().enumerate() {
                use std::ffi::CString;
                let c_prompt = CString::new(pair.prompt.clone()).unwrap();
                let input = PluginInput { text: c_prompt.as_ptr() };
                let output_obj = unsafe { ((*dispatcher.vtable).run)(&input) };
                let c_str = unsafe { std::ffi::CStr::from_ptr(output_obj.text) };
                let generated = c_str.to_string_lossy().to_string();
                unsafe { ((*dispatcher.vtable).free_output)(output_obj) };
                let expected = normalize_yaml(&pair.workflow);
                let actual = normalize_yaml(&generated);
                let pass = expected == actual;
                if !pass {
                    failures += 1;
                    println!("[FAIL] Prompt {}: {}\nExpected:\n{}\nActual:\n{}\n", i + 1, pair.prompt, pair.workflow, generated);
                    if fail_fast {
                        println!("Fail-fast enabled. Stopping at first failure.");
                        std::process::exit(1);
                    }
                } else if verbose {
                    println!("[PASS] Prompt {}: {}", i + 1, pair.prompt);
                }
            }
            if failures == 0 {
                println!("All prompts passed validation!");
            } else {
                println!("{} prompts failed validation.", failures);
                std::process::exit(1);
            }
        }
        Commands::ListWorkflows => {
            let dir = std::path::Path::new("workflows");
            match std::fs::read_dir(dir) {
                Ok(entries) => {
                    println!("Available workflows:");
                    let mut found = false;
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "yaml" || ext == "yml" {
                                println!("- {}", path.file_name().unwrap().to_string_lossy());
                                found = true;
                            }
                        }
                    }
                    if !found {
                        println!("[INFO] No workflow YAML files found in workflows/ directory.");
                    }
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to read workflows directory: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::ViewWorkflow { name } => {
            let path = format!("workflows/{}.yaml", name);
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    println!("Workflow {}:\n{}", name, contents);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to read workflow file {}: {}", path, e);
                    std::process::exit(1);
                }
            }
        }
        Commands::DeleteWorkflow { name } => {
            let path = format!("workflows/{}.yaml", name);
            match std::fs::remove_file(&path) {
                Ok(_) => {
                    println!("Deleted workflow file {}", path);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to delete workflow file {}: {}", path, e);
                    std::process::exit(1);
                }
            }
        }
    }
} 