use clap::{Parser, Subcommand};
use lao_orchestrator_core::{
    run_workflow_yaml, load_workflow_yaml, plugins::PluginRegistry,
    scheduler::WorkflowScheduler, workflow_state::WorkflowSchedule,
};
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
    /// Explain a plugin's capabilities, schemas, and usage examples
    ExplainPlugin {
        name: String,
    },
    /// Schedule a workflow to run at specified intervals
    Schedule {
        workflow_path: String,
        #[arg(long, help = "Cron-like expression (e.g., 'interval:60' for every 60 minutes)")]
        cron: String,
        #[arg(long, help = "Maximum number of times to run (optional)")]
        max_runs: Option<u32>,
    },
    /// Unschedule a workflow
    Unschedule {
        workflow_id: String,
    },
    /// List all scheduled workflows
    ListScheduled,
    /// Show workflow execution history and state
    Status {
        #[arg(help = "Workflow ID (optional - shows all if not specified)")]
        workflow_id: Option<String>,
    },
    /// Clean up old workflow states
    Cleanup {
        #[arg(long, default_value = "168", help = "Remove states older than this many hours")]
        max_age_hours: u64,
    },
    /// Run the workflow scheduler daemon
    Daemon {
        #[arg(long, default_value = "60", help = "Check interval in seconds")]
        interval: u64,
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
                    let dag = match lao_orchestrator_core::build_dag(&workflow.steps) {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("[ERROR] Failed to build DAG: {}", e);
                            std::process::exit(1);
                        }
                    };
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
            let dispatcher = match registry.plugins.get("PromptDispatcher") {
                Some(d) => d,
                None => {
                    eprintln!("PromptDispatcherPlugin not found");
                    std::process::exit(1);
                }
            };
            // SAFETY: FFI call to plugin, must ensure input is valid and plugin is trusted.
            use std::ffi::CString;
            let c_prompt = match CString::new(prompt.clone()) {
                Ok(c) => c,
                Err(_) => {
                    eprintln!("Failed to create CString from prompt");
                    std::process::exit(1);
                }
            };
            let input = PluginInput { text: c_prompt.into_raw() };
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
                let data = match std::fs::read_to_string(&path) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Failed to read prompt library: {}", e);
                        std::process::exit(1);
                    }
                };
                match serde_json::from_str(&data) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Failed to parse prompt library JSON: {}", e);
                        std::process::exit(1);
                    }
                }
            };
            let mut registry = PluginRegistry::dynamic_registry("plugins/");
            let dispatcher = match registry.plugins.get("PromptDispatcher") {
                Some(d) => d,
                None => {
                    eprintln!("PromptDispatcherPlugin not found");
                    std::process::exit(1);
                }
            };
            let mut failures = 0;
            for (i, pair) in prompt_pairs.iter().enumerate() {
                use std::ffi::CString;
                let c_prompt = match CString::new(pair.prompt.clone()) {
                    Ok(c) => c,
                    Err(_) => {
                        eprintln!("Failed to create CString from prompt");
                        failures += 1;
                        continue;
                    }
                };
                let input = PluginInput { text: c_prompt.into_raw() };
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
                                match path.file_name() {
                                    Some(name) => println!("- {}", name.to_string_lossy()),
                                    None => println!("- [unknown file name]"),
                                }
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
        Commands::ExplainPlugin { name } => {
            use std::fs;
            use std::path::Path;
            let plugin_dir = format!("plugins/{}Plugin", name);
            let yaml_path = Path::new(&plugin_dir).join("plugin.yaml");
            let yaml_str = match fs::read_to_string(&yaml_path) {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("[ERROR] plugin.yaml not found for plugin '{}'. Looked in {}", name, yaml_path.display());
                    std::process::exit(1);
                }
            };
            let manifest: serde_yaml::Value = match serde_yaml::from_str(&yaml_str) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("[ERROR] Failed to parse plugin.yaml: {}", e);
                    std::process::exit(1);
                }
            };
            println!("\n--- Plugin: {} ---", name);
            if let Some(desc) = manifest.get("description").and_then(|v| v.as_str()) {
                println!("Description: {}", desc);
            }
            if let Some(tags) = manifest.get("tags") {
                println!("Tags: {:?}", tags);
            }
            if let Some(input) = manifest.get("input") {
                println!("Input Schema: {}", serde_yaml::to_string(input).unwrap_or_default().trim());
            }
            if let Some(output) = manifest.get("output") {
                println!("Output Schema: {}", serde_yaml::to_string(output).unwrap_or_default().trim());
            }
            if let Some(examples) = manifest.get("example_prompts") {
                println!("Example Prompts:");
                if let Some(arr) = examples.as_sequence() {
                    for ex in arr {
                        println!("  - {:?}", ex);
                    }
                } else {
                    println!("  - {:?}", examples);
                }
            }
        }
        Commands::Schedule { workflow_path, cron, max_runs } => {
            let workflow_id = format!("scheduled_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..8].to_string());
            
            // Validate workflow exists
            if !std::path::Path::new(&workflow_path).exists() {
                eprintln!("[ERROR] Workflow file not found: {}", workflow_path);
                std::process::exit(1);
            }
            
            let schedule = WorkflowSchedule {
                cron_expression: Some(cron.clone()),
                next_run: None,
                enabled: true,
                max_runs,
                run_count: 0,
            };
            
            let mut scheduler = match WorkflowScheduler::new("workflow_states") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[ERROR] Failed to initialize scheduler: {}", e);
                    std::process::exit(1);
                }
            };
            
            match scheduler.schedule_workflow(workflow_id.clone(), workflow_path.clone(), schedule) {
                Ok(_) => {
                    println!("✓ Scheduled workflow '{}' with ID: {}", workflow_path, workflow_id);
                    println!("  Cron expression: {}", cron);
                    if let Some(max) = max_runs {
                        println!("  Max runs: {}", max);
                    }
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to schedule workflow: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Unschedule { workflow_id } => {
            let mut scheduler = match WorkflowScheduler::new("workflow_states") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[ERROR] Failed to initialize scheduler: {}", e);
                    std::process::exit(1);
                }
            };
            
            match scheduler.unschedule_workflow(&workflow_id) {
                Ok(_) => println!("✓ Unscheduled workflow: {}", workflow_id),
                Err(e) => {
                    eprintln!("[ERROR] Failed to unschedule workflow: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::ListScheduled => {
            let scheduler = match WorkflowScheduler::new("workflow_states") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[ERROR] Failed to initialize scheduler: {}", e);
                    std::process::exit(1);
                }
            };
            
            let scheduled = scheduler.list_scheduled_workflows();
            if scheduled.is_empty() {
                println!("No scheduled workflows found.");
            } else {
                println!("Scheduled workflows:");
                for (id, workflow) in scheduled {
                    println!("  {} - {}", id, workflow.workflow_path);
                    println!("    Schedule: {:?}", workflow.schedule.cron_expression);
                    println!("    Next run: {:?}", workflow.next_run);
                    println!("    Enabled: {}", workflow.schedule.enabled);
                    if let Some(last) = workflow.last_run {
                        println!("    Last run: {:?}", last);
                    }
                    println!("    Run count: {}", workflow.schedule.run_count);
                    println!();
                }
            }
        }
        Commands::Status { workflow_id } => {
            let scheduler = match WorkflowScheduler::new("workflow_states") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[ERROR] Failed to initialize scheduler: {}", e);
                    std::process::exit(1);
                }
            };
            
            if let Some(id) = workflow_id {
                match scheduler.get_workflow_history(&id) {
                    Ok(Some(state)) => {
                        println!("Workflow: {} ({})", state.workflow_name, state.workflow_id);
                        println!("Status: {:?}", state.status);
                        println!("Created: {:?}", state.created_at);
                        if let Some(started) = state.started_at {
                            println!("Started: {:?}", started);
                        }
                        if let Some(completed) = state.completed_at {
                            println!("Completed: {:?}", completed);
                        }
                        println!("Progress: {}/{} steps", state.current_step, state.total_steps);
                        if let Some(error) = &state.error_message {
                            println!("Error: {}", error);
                        }
                    }
                    Ok(None) => println!("Workflow not found: {}", id),
                    Err(e) => eprintln!("[ERROR] Failed to load workflow state: {}", e),
                }
            } else {
                let states = scheduler.list_workflow_states();
                if states.is_empty() {
                    println!("No workflow states found.");
                } else {
                    println!("All workflow states:");
                    for state in states {
                        println!("  {} - {} ({:?})", state.workflow_id, state.workflow_name, state.status);
                    }
                }
            }
        }
        Commands::Cleanup { max_age_hours } => {
            let mut scheduler = match WorkflowScheduler::new("workflow_states") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[ERROR] Failed to initialize scheduler: {}", e);
                    std::process::exit(1);
                }
            };
            
            match scheduler.cleanup_old_states(max_age_hours) {
                Ok(count) => println!("✓ Cleaned up {} old workflow states", count),
                Err(e) => eprintln!("[ERROR] Failed to cleanup states: {}", e),
            }
        }
        Commands::Daemon { interval } => {
            println!("Starting LAO workflow scheduler daemon...");
            println!("Check interval: {} seconds", interval);
            
            let mut scheduler = match WorkflowScheduler::new("workflow_states") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[ERROR] Failed to initialize scheduler: {}", e);
                    std::process::exit(1);
                }
            };
            
            loop {
                let due_workflows = scheduler.get_due_workflows();
                if !due_workflows.is_empty() {
                    println!("Found {} due workflows", due_workflows.len());
                    for workflow_id in due_workflows {
                        // In a real implementation, you'd execute the workflow here
                        println!("Would execute workflow: {}", workflow_id);
                        let _ = scheduler.update_workflow_run(&workflow_id);
                    }
                }
                
                std::thread::sleep(std::time::Duration::from_secs(interval));
            }
        }
    }
} 