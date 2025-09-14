use lao_orchestrator_core::{load_workflow_yaml, run_workflow_yaml_with_callback, run_workflow_yaml_parallel_with_callback, StepEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub run: String,
    pub input_type: Option<String>,
    pub output_type: Option<String>,
    pub status: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
}

pub struct BackendState {
    pub workflow_path: String,
    pub graph: Option<WorkflowGraph>,
    pub error: String,
    pub plugins: Vec<UiPluginInfo>,
    pub live_logs: Vec<String>,
    pub selected_node: Option<String>,
}

impl Default for BackendState {
    fn default() -> Self {
        Self {
            workflow_path: String::new(),
            graph: None,
            error: String::new(),
            plugins: Vec::new(),
            live_logs: Vec::new(),
            selected_node: None,
        }
    }
}

pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub fn get_workflow_graph(path: &str) -> Result<WorkflowGraph, String> {
    let workflow = load_workflow_yaml(path)?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    
    for (_i, step) in workflow.steps.iter().enumerate() {
        let id = step.run.clone();
        nodes.push(GraphNode {
            id: id.clone(),
            run: step.run.clone(),
            input_type: None,
            output_type: None,
            status: "pending".to_string(),
            x: 100.0 + (_i as f32 * 150.0),
            y: 100.0,
        });
        
        if let Some(ref from) = step.input_from {
            edges.push(GraphEdge { 
                from: from.clone(), 
                to: id.clone() 
            });
        }
        
        if let Some(ref deps) = step.depends_on {
            for d in deps {
                edges.push(GraphEdge { 
                    from: d.clone(), 
                    to: id.clone() 
                });
            }
        }
    }
    
    Ok(WorkflowGraph { nodes, edges })
}

pub fn list_plugins_for_ui() -> Result<Vec<UiPluginInfo>, String> {
    let plugins_dir = resolve_plugins_dir();
    let mut out: Vec<UiPluginInfo> = Vec::new();

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
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: scan shared libs for names if no manifests found
    if out.is_empty() {
        if let Ok(files) = std::fs::read_dir(&plugins_dir) {
            for f in files.flatten() {
                let path = f.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if matches!(ext, "so" | "dll" | "dylib") {
                        if let Some(fname) = path.file_stem().and_then(|s| s.to_str()) {
                            let base = fname.strip_prefix("lib").unwrap_or(fname);
                            if !out.iter().any(|i| i.name.eq_ignore_ascii_case(base)) {
                                out.push(UiPluginInfo {
                                    name: base.to_string(),
                                    version: String::new(),
                                    description: String::new(),
                                    author: String::new(),
                                    tags: Vec::new(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(out)
}

fn resolve_plugins_dir() -> String {
    if let Ok(dir) = std::env::var("LAO_PLUGINS_DIR") {
        if std::path::Path::new(&dir).exists() { 
            return dir; 
        }
    }
    
    let candidates = [
        "plugins/",
        "./plugins/", 
        "../plugins/",
        "../../plugins/",
    ];
    
    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    
    "plugins/".to_string()
}

pub fn run_workflow_stream(path: String, parallel: bool, log_callback: impl Fn(String) + Send + Sync + 'static) -> Result<(), String> {
    std::thread::spawn(move || {
        let emit = |e: StepEvent| {
            log_callback(format!("{:?}", e));
        };
        
        let result = if parallel {
            run_workflow_yaml_parallel_with_callback(&path, emit)
        } else {
            run_workflow_yaml_with_callback(&path, emit)
        };
        
        match result {
            Ok(logs) => log_callback(format!("DONE: Workflow completed with {} steps", logs.len())),
            Err(err) => log_callback(format!("ERROR: {}", err)),
        }
    });
    
    Ok(())
}