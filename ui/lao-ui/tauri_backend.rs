use lao_orchestrator_core::{load_workflow_yaml, run_model_runner, run_workflow_yaml, Workflow, WorkflowStep};
use serde::Serialize;

#[derive(Serialize)]
pub struct WorkflowGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Serialize)]
pub struct GraphNode {
    pub id: String,
    pub run: String,
    pub input_type: Option<String>,
    pub output_type: Option<String>,
    pub status: String,
}

#[derive(Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn tauri_load_workflow_yaml(path: &str) -> Result<lao_orchestrator_core::Workflow, String> {
    load_workflow_yaml(path)
}

#[tauri::command]
fn tauri_run_model_runner(runner: &str, params: serde_yaml::Value) -> Result<String, String> {
    run_model_runner(runner, params)
}

#[tauri::command]
fn tauri_run_workflow_yaml(path: &str) -> Result<Vec<lao_orchestrator_core::StepLog>, String> {
    run_workflow_yaml(path)
}

#[tauri::command]
fn get_workflow_graph(path: &str) -> Result<WorkflowGraph, String> {
    let workflow = load_workflow_yaml(path)?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for (i, step) in workflow.steps.iter().enumerate() {
        let id = format!("step{}", i + 1);
        nodes.push(GraphNode {
            id: id.clone(),
            run: step.run.clone(),
            input_type: None, // Could be filled with plugin_registry lookup
            output_type: None,
            status: "pending".to_string(),
        });
        if let Some(ref from) = step.input_from {
            edges.push(GraphEdge { from: from.clone(), to: id.clone() });
        }
        if let Some(ref deps) = step.depends_on {
            for d in deps {
                edges.push(GraphEdge { from: d.clone(), to: id.clone() });
            }
        }
    }
    Ok(WorkflowGraph { nodes, edges })
}

#[tauri::command]
fn dispatch_prompt(prompt: String) -> Result<String, String> {
    let mut registry = lao_orchestrator_core::plugins::PluginRegistry::default_registry();
    let dispatcher = registry.get_mut("PromptDispatcher").ok_or("PromptDispatcherPlugin not found")?;
    match dispatcher.execute(lao_orchestrator_core::plugins::PluginInput::Text(prompt)) {
        Ok(lao_orchestrator_core::plugins::PluginOutput::Text(yaml)) => Ok(yaml),
        Ok(_) => Err("PromptDispatcher did not return YAML text".to_string()),
        Err(e) => Err(format!("PromptDispatcher failed: {:?}", e)),
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, tauri_load_workflow_yaml, tauri_run_model_runner, tauri_run_workflow_yaml, get_workflow_graph, dispatch_prompt])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 