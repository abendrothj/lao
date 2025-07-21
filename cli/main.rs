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
    }
} 