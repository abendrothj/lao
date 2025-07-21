use lao_orchestrator_core::{run_workflow_yaml};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: test_runner <workflow_yaml_path>");
        std::process::exit(1);
    }
    let path = &args[1];
    match run_workflow_yaml(path) {
        Ok(results) => {
            println!("Workflow executed successfully. Step outputs:");
            for (i, output) in results.iter().enumerate() {
                println!("Step {}: {}", i + 1, output);
            }
        }
        Err(e) => {
            eprintln!("Workflow execution failed: {}", e);
            std::process::exit(1);
        }
    }
} 