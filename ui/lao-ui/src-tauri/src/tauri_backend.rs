use lao_orchestrator_core::{load_workflow_yaml, run_model_runner, run_workflow_yaml, run_workflow_yaml_with_callback, run_workflow_yaml_parallel_with_callback, StepEvent};
use serde::Serialize;
use tauri_plugin_fs;
use lao_plugin_api::PluginInput;
use tauri::{AppHandle, Emitter};

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

#[derive(Serialize)]
pub struct UiPluginCapability {
    pub name: String,
    pub description: String,
    pub input_type: String,
    pub output_type: String,
}

#[derive(Serialize)]
pub struct UiPluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    pub capabilities: Vec<UiPluginCapability>,
}

fn resolve_plugins_dir() -> String {
    if let Ok(dir) = std::env::var("LAO_PLUGINS_DIR") {
        if std::path::Path::new(&dir).exists() { return dir; }
    }
    let candidates = [
        "plugins",
        "../plugins",
        "../../plugins",
        "../../../plugins",
    ];
    for c in candidates { if std::path::Path::new(c).exists() { return c.to_string(); } }
    // Fallback to repo root assumption
    "plugins".to_string()
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
    for (_i, step) in workflow.steps.iter().enumerate() {
        let id = step.run.clone();
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
    let plugins_dir = resolve_plugins_dir();
    let mut registry = lao_orchestrator_core::plugins::PluginRegistry::dynamic_registry(&plugins_dir);
    let dispatcher = registry.plugins.get_mut("PromptDispatcherPlugin").ok_or("PromptDispatcherPlugin not found")?;
    let c_prompt = std::ffi::CString::new(prompt).map_err(|e| format!("CString error: {}", e))?;
    let input = PluginInput { text: c_prompt.into_raw() };
    let output_obj = unsafe { ((*dispatcher.vtable).run)(&input) };
    let c_str = unsafe { std::ffi::CStr::from_ptr(output_obj.text) };
    let yaml = c_str.to_string_lossy().to_string();
    unsafe { ((*dispatcher.vtable).free_output)(output_obj) };
    Ok(yaml)
}

#[tauri::command]
fn run_workflow_stream(app: AppHandle, path: String, parallel: bool) -> Result<(), String> {
    // Spawn a thread so we do not block the Tauri runtime
    std::thread::spawn(move || {
        let emit = |e: StepEvent| {
            let _ = app.emit("workflow:status", &e);
        };
        let result = if parallel {
            run_workflow_yaml_parallel_with_callback(&path, emit)
        } else {
            run_workflow_yaml_with_callback(&path, emit)
        };
        let done_payload = match result {
            Ok(logs) => serde_json::json!({"ok": true, "logs": logs}),
            Err(err) => serde_json::json!({"ok": false, "error": err}),
        };
        let _ = app.emit("workflow:done", done_payload);
    });
    Ok(())
}

#[tauri::command]
fn list_plugins_for_ui() -> Result<Vec<UiPluginInfo>, String> {
    let plugins_dir = resolve_plugins_dir();
    let mut out: Vec<UiPluginInfo> = Vec::new();

    // Primary: scan manifests for a simple, robust list
    if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let manifest = p.join("plugin.yaml");
                if manifest.exists() {
                    if let Ok(txt) = std::fs::read_to_string(&manifest) {
                        if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(&txt) {
                            let name = val.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            if !name.is_empty() && !out.iter().any(|i| i.name == name) {
                                let version = val.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let description = val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let author = val.get("maintainer").or_else(|| val.get("author")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let tags = val.get("tags").and_then(|v| v.as_sequence()).map(|seq| {
                                    seq.iter().filter_map(|e| e.as_str().map(|s| s.to_string())).collect::<Vec<_>>()
                                }).unwrap_or_default();
                                out.push(UiPluginInfo {
                                    name,
                                    version,
                                    description,
                                    author,
                                    tags,
                                    capabilities: Vec::new(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: scan shared libs for names if no manifests or additional libs present
    if let Ok(files) = std::fs::read_dir(&plugins_dir) {
        for f in files.flatten() {
            let path = f.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if matches!(ext, "so" | "dll" | "dylib") {
                    if let Some(fname) = path.file_stem().and_then(|s| s.to_str()) {
                        // strip common prefixes like lib
                        let base = fname.strip_prefix("lib").unwrap_or(fname);
                        // keep as-is; UI will display
                        if !out.iter().any(|i| i.name.eq_ignore_ascii_case(base)) {
                            out.push(UiPluginInfo {
                                name: base.to_string(),
                                version: String::new(),
                                description: String::new(),
                                author: String::new(),
                                tags: Vec::new(),
                                capabilities: Vec::new(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by name for consistent UI
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![greet, tauri_load_workflow_yaml, tauri_run_model_runner, tauri_run_workflow_yaml, get_workflow_graph, dispatch_prompt, run_workflow_stream, list_plugins_for_ui])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 