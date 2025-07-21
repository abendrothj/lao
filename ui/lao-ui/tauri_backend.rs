use lao_orchestrator_core::{load_workflow_yaml, run_model_runner, run_workflow_yaml};

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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, tauri_load_workflow_yaml, tauri_run_model_runner, tauri_run_workflow_yaml])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 