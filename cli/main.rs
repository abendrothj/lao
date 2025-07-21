use clap::{Parser, Subcommand};
use lao_orchestrator_core::{run_workflow_yaml, load_workflow_yaml};
use lao_orchestrator_core::plugins::PluginRegistry;

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
    },
    /// Generate and run a workflow from a prompt
    Prompt {
        prompt: String,
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
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { path, dry_run } => {
            if dry_run {
                match load_workflow_yaml(&path) {
                    Ok(workflow) => {
                        let plugin_registry = PluginRegistry::default_registry();
                        println!("[DRY RUN] Workflow: {}", workflow.workflow);
                        for (i, step) in workflow.steps.iter().enumerate() {
                            let plugin = plugin_registry.get(&step.run);
                            println!("Step {}: {}", i + 1, step.run);
                            match plugin {
                                Some(p) => {
                                    let sig = p.io_signature();
                                    println!("  Input: {:?}", sig.input_type);
                                    println!("  Output: {:?}", sig.output_type);
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
                    let plugin_registry = PluginRegistry::default_registry();
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
            let plugin_registry = PluginRegistry::default_registry();
            println!("Available plugins:");
            for (name, plugin) in &plugin_registry.plugins {
                let sig = plugin.io_signature();
                println!("- {}: {}\n    Input: {:?}\n    Output: {:?}", name, sig.description, sig.input_type, sig.output_type);
            }
        }
        Commands::NewWorkflow { name } => {
            let path = format!("workflows/{}.yaml", name);
            let template = format!(
                "workflow: \"{}\"\nsteps:\n  - run: Whisper\n    input: audio.wav\n    retry_count: 2\n    retry_delay: 1000\n    cache_key: \"whisper_{}\"\n  - run: Ollama\n    input_from: Whisper\n    cache_key: \"summary_{}\"\n",
                name, name, name
            );
            std::fs::create_dir_all("workflows").ok();
            std::fs::write(&path, template).expect("Failed to write workflow file");
            println!("Scaffolded new workflow at {}", path);
        }
        Commands::Prompt { prompt } => {
            // Use the PromptDispatcherPlugin to generate a workflow YAML
            let mut registry = PluginRegistry::default_registry();
            let dispatcher = registry.get_mut("PromptDispatcher").expect("PromptDispatcherPlugin not found");
            let result = dispatcher.execute(lao_orchestrator_core::plugins::PluginInput::Text(prompt));
            match result {
                Ok(lao_orchestrator_core::plugins::PluginOutput::Text(yaml)) => {
                    println!("Generated workflow:\n{}", yaml);
                    // Parse the YAML into a Workflow struct
                    match serde_yaml::from_str::<lao_orchestrator_core::Workflow>(&yaml) {
                        Ok(workflow) => {
                            // Save to a temp file and run
                            let temp_path = "workflows/generated_from_prompt.yaml";
                            std::fs::create_dir_all("workflows").ok();
                            std::fs::write(temp_path, &yaml).expect("Failed to write workflow file");
                            match run_workflow_yaml(temp_path) {
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
                        Err(e) => {
                            eprintln!("Failed to parse generated workflow YAML: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Ok(_) => {
                    eprintln!("PromptDispatcher did not return YAML text");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("PromptDispatcher failed: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::ValidatePrompts { path, fail_fast, verbose } => {
            use serde::Deserialize;
            use std::fs;
            #[derive(Deserialize)]
            struct PromptPair { prompt: String, workflow: String }
            fn normalize_yaml(yaml: &str) -> serde_yaml::Value {
                serde_yaml::from_str(yaml).unwrap_or(serde_yaml::Value::Null)
            }
            let data = fs::read_to_string(&path).expect("Failed to read prompt_library.json");
            let pairs: Vec<PromptPair> = serde_json::from_str(&data).expect("Failed to parse prompt_library.json");
            let mut registry = PluginRegistry::default_registry();
            let dispatcher = registry.get_mut("PromptDispatcher").expect("PromptDispatcherPlugin not found");
            let mut passed = 0;
            let mut failed = 0;
            for (i, pair) in pairs.iter().enumerate() {
                println!("\nTest {}: {}", i + 1, pair.prompt);
                let result = dispatcher.execute(lao_orchestrator_core::plugins::PluginInput::Text(pair.prompt.clone()));
                match result {
                    Ok(lao_orchestrator_core::plugins::PluginOutput::Text(generated)) => {
                        let expected_norm = normalize_yaml(&pair.workflow);
                        let generated_norm = normalize_yaml(&generated);
                        if expected_norm == generated_norm {
                            if verbose { println!("  ✅ PASS"); }
                            passed += 1;
                        } else {
                            println!("  ❌ FAIL");
                            if verbose || !fail_fast {
                                println!("  Expected:\n{}", pair.workflow);
                                println!("  Got:\n{}", generated);
                            }
                            failed += 1;
                            if fail_fast { break; }
                        }
                    }
                    Ok(_) => {
                        println!("  ❌ FAIL: Dispatcher did not return YAML text");
                        failed += 1;
                        if fail_fast { break; }
                    }
                    Err(e) => {
                        println!("  ❌ FAIL: Dispatcher error: {:?}", e);
                        failed += 1;
                        if fail_fast { break; }
                    }
                }
            }
            println!("\nTest Summary: {} passed, {} failed, {} total", passed, failed, passed + failed);
            if failed > 0 { std::process::exit(1); }
        }
    }
} 