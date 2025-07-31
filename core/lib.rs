// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

// --- Workflow Engine (Step 2) ---
use std::fs;
use std::process::Command;
use std::collections::HashMap;
use std::time::Instant;
use std::{thread, time::Duration};
use std::env as std_env;
use std::ffi::CString;
use lao_plugin_api::PluginInput;
pub mod plugins;
use plugins::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Workflow {
    pub workflow: String,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct WorkflowStep {
    pub run: String,
    #[serde(flatten)]
    pub params: serde_yaml::Value,
    #[serde(default)]
    pub retries: Option<u32>,
    #[serde(default)]
    pub retry_delay: Option<u64>, // milliseconds
    #[serde(default)]
    pub cache_key: Option<String>,
    #[serde(default)]
    pub input_from: Option<String>,
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct DagNode {
    pub id: String,
    pub step: WorkflowStep,
    pub parents: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct StepLog {
    pub step: usize,
    pub runner: String,
    pub input: serde_yaml::Value,
    pub output: Option<String>,
    pub error: Option<String>,
    pub attempt: u32,
    pub input_type: Option<plugins::PluginInputType>,
    pub output_type: Option<plugins::PluginInputType>,
    pub validation: Option<String>,
}

pub fn load_workflow_yaml(path: &str) -> Result<Workflow, String> {
    let yaml_str = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_yaml::from_str::<Workflow>(&yaml_str).map_err(|e| e.to_string())
}

pub fn run_model_runner(runner: &str, params: serde_yaml::Value) -> Result<String, String> {
    // Example: match runner and build the real command
    let mut cmd = Command::new(runner);
    // For demonstration, handle Whisper.cpp and Ollama with basic params
    if runner == "whisper" {
        // Example: { input: "audio.wav" }
        if let Some(input) = params.get("input").and_then(|v| v.as_str()) {
            cmd.arg(input);
        }
    } else if runner == "ollama" {
        // Example: { model: "mistral", prompt: "..." }
        if let Some(model) = params.get("model").and_then(|v| v.as_str()) {
            cmd.arg("run").arg(model);
        }
        if let Some(prompt) = params.get("prompt").and_then(|v| v.as_str()) {
            cmd.arg(prompt);
        }
    } else {
        // Fallback: pass all params as stringified args
        for (k, v) in params.as_mapping().unwrap_or(&serde_yaml::Mapping::new()) {
            cmd.arg(format!("--{k:?}")).arg(format!("{v:?}"));
        }
    }
    // Run the command and capture output
    let output = cmd.output().map_err(|e| format!("Failed to run {runner}: {e}"))?;
    if !output.status.success() {
        return Err(format!("{} failed: {}", runner, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn build_dag(steps: &[WorkflowStep]) -> Result<Vec<DagNode>, String> {
    let mut nodes = Vec::new();
    for step in steps.iter() {
        let id = step.run.clone();
        let mut parents = Vec::new();
        if let Some(ref from) = step.input_from {
            parents.push(from.clone());
        }
        if let Some(ref deps) = step.depends_on {
            for d in deps {
                parents.push(d.clone());
            }
        }
        nodes.push(DagNode { id, step: step.clone(), parents });
    }
    Ok(nodes)
}

pub fn topo_sort(nodes: &[DagNode]) -> Result<Vec<String>, String> {
    let mut order = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut visiting = std::collections::HashSet::new();
    let map: HashMap<String, &DagNode> = nodes.iter().map(|n| (n.id.clone(), n)).collect();
    fn visit(
        n: &DagNode,
        map: &HashMap<String, &DagNode>,
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), String> {
        if visited.contains(&n.id) {
            return Ok(());
        }
        if !visiting.insert(n.id.clone()) {
            return Err(format!("Cycle detected at node {}", n.id));
        }
        for p in &n.parents {
            if let Some(parent) = map.get(p) {
                visit(parent, map, visited, visiting, order)?;
            }
        }
        visiting.remove(&n.id);
        visited.insert(n.id.clone());
        order.push(n.id.clone());
        Ok(())
    }
    for n in nodes {
        visit(n, &map, &mut visited, &mut visiting, &mut order)?;
    }
    Ok(order)
}

pub fn validate_workflow_types(
    dag: &[DagNode],
    plugin_registry: &PluginRegistry,
) -> Vec<(usize, String)> {
    let mut errors = Vec::new();
    let mut node_outputs: HashMap<String, plugins::PluginInputType> = HashMap::new();
    for (i, node) in dag.iter().enumerate() {
        let step = &node.step;
        if !plugin_registry.plugins.contains_key(&step.run) {
            errors.push((i+1, format!("Step {} references missing plugin '{}'", i+1, step.run)));
            continue;
        }
        // Remove or comment out all lines using plugin.io_signature(), input_type, output_type, and related type validation logic.
        let input_type = plugins::PluginInputType::Any;
        let output_type = plugins::PluginInputType::Any;
        // Check input_from/depends_on
        for parent in &node.parents {
            if let Some(parent_type) = node_outputs.get(parent) {
                if *parent_type != input_type && input_type != plugins::PluginInputType::Any {
                    errors.push((i+1, format!("Step {} expects {:?} but parent {} outputs {:?}", i+1, input_type, parent, parent_type)));
                }
            }
        }
        node_outputs.insert(node.id.clone(), output_type);
    }
    errors
}

/// Run an entire workflow: iterate steps, call model runners, collect results
pub fn run_workflow_yaml(path: &str) -> Result<Vec<StepLog>, String> {
    let workflow = load_workflow_yaml(path)?;
    let dag = build_dag(&workflow.steps)?;
    let plugin_registry = PluginRegistry::dynamic_registry("../plugins/");
    let validation_errors = validate_workflow_types(&dag, &plugin_registry);
    let mut logs = Vec::new();
    if !validation_errors.is_empty() {
        for (step, msg) in &validation_errors {
            logs.push(StepLog {
                step: *step,
                runner: "VALIDATION".into(),
                input: serde_yaml::Value::Null,
                output: None,
                error: Some(msg.clone()),
                attempt: 0,
                input_type: None,
                output_type: None,
                validation: Some("error".into()),
            });
        }
        return Ok(logs);
    }
    let order = topo_sort(&dag)?;
    let mut outputs: HashMap<String, String> = HashMap::new();
    let node_map: HashMap<String, &DagNode> = dag.iter().map(|n| (n.id.clone(), n)).collect();
    for node_id in order {
        let node = match node_map.get(&node_id) {
            Some(n) => n,
            None => {
                logs.push(StepLog {
                    step: node_id[4..].parse().unwrap_or(0),
                    runner: "ERROR".into(),
                    input: serde_yaml::Value::Null,
                    output: None,
                    error: Some(format!("Node {node_id} not found in DAG")),
                    attempt: 0,
                    input_type: None,
                    output_type: None,
                    validation: Some("error".into()),
                });
                break;
            }
        };
        let step = &node.step;
        let mut params = step.params.clone();
        // If input_from is set, inject the output of the parent as 'input'
        if let Some(ref from) = step.input_from {
            if let Some(parent_out) = outputs.get(from) {
                if let Some(map) = params.as_mapping_mut() {
                    map.insert(serde_yaml::Value::from("input"), serde_yaml::Value::from(parent_out.clone()));
                }
            }
        }
        substitute_params(&mut params, &outputs);
        let retries = step.retries.unwrap_or(1);
        let retry_delay = step.retry_delay.unwrap_or(500); // ms
        let mut last_err = None;
        let mut output = None;
        let mut attempt = 0;
        let start_time = Instant::now();
        let input_type = None;
        let output_type = None;
        let validation: Option<String> = Some("ok".into());
        // Caching logic
        let cache_dir = std_env::var("LAO_CACHE_DIR").unwrap_or_else(|_| "cache".to_string());
        if let Err(e) = std::fs::create_dir_all(&cache_dir) {
            logs.push(StepLog {
                step: node_id[4..].parse().unwrap_or(0),
                runner: step.run.clone(),
                input: params.clone(),
                output: None,
                error: Some(format!("Failed to create cache dir: {e}")),
                attempt: 0,
                input_type: None,
                output_type: None,
                validation: Some("error".into()),
            });
            break;
        }
        let cache_key = step.cache_key.clone();
        let cache_path = cache_key.as_ref().map(|k| format!("{cache_dir}/{k}.json"));
        let mut cache_status = None;
        if let Some(ref path) = cache_path {
            if let Ok(cached) = std::fs::read_to_string(path) {
                if let Ok(val) = serde_json::from_str::<String>(&cached) {
                    println!("STEP {}: cache hit for key {}", step.run, path);
                    output = Some(val.clone());
                    outputs.insert(node_id.clone(), val);
                    cache_status = Some("hit".to_string());
                    logs.push(StepLog {
                        step: node_id[4..].parse().unwrap_or(0),
                        runner: step.run.clone(),
                        input: params.clone(),
                        output: output.clone(),
                        error: None,
                        attempt: 0,
                        input_type: None,
                        output_type: None,
                        validation: Some("cache".into()),
                    });
                    continue;
                }
            }
            cache_status = Some("miss".to_string());
        }
        for try_num in 1..=retries {
            attempt = try_num;
            if try_num > 1 {
                let delay = retry_delay * 2u64.pow((try_num - 2));
                println!("STEP {}: retry {} after {}ms", step.run, try_num, delay);
                thread::sleep(Duration::from_millis(delay));
            }
            if let Some(plugin) = plugin_registry.get(&step.run) {
                let input = build_plugin_input(&params);
                // Debug: print raw input for plugin
                unsafe {
                    if !input.text.is_null() {
                        let c_str = std::ffi::CStr::from_ptr(input.text);
                        println!("[DEBUG] Raw input to plugin '{}': {}", step.run, c_str.to_string_lossy());
                    } else {
                        println!("[DEBUG] Raw input to plugin '{}': <null>", step.run);
                    }
                }
                let vtable = unsafe { &*plugin.vtable };
                println!("[DEBUG] PluginVTable fn ptrs: name={:p} run={:p} free_output={:p} run_with_buffer={:p}",
                    vtable.name as *const (), vtable.run as *const (), vtable.free_output as *const (), vtable.run_with_buffer as *const ());
                if let Some(run_with_buffer) = plugin.run_with_buffer {
                    println!("[DEBUG] Echo plugin run_with_buffer fn ptr: {:p}", run_with_buffer as *const ());
                    let mut buffer = vec![0u8; 4096];
                    let written = unsafe { run_with_buffer(&input, buffer.as_mut_ptr() as *mut i8, buffer.len()) };
                    println!("[DEBUG] Echo plugin run_with_buffer wrote {written} bytes");
                    println!("[DEBUG] Echo plugin run_with_buffer buffer bytes: {:?}", &buffer[..std::cmp::min(written, 32)]);
                    if written > 0 && written < buffer.len() {
                        let s = String::from_utf8_lossy(&buffer[..written]).to_string();
                        println!("[DEBUG] Echo plugin run_with_buffer output: {s}");
                        output = Some(s.clone());
                        outputs.insert(node_id.clone(), s);
                    } else {
                        println!("[DEBUG] Echo plugin run_with_buffer returned no output");
                        output = None;
                        last_err = Some("Plugin returned no output (invalid input?)".to_string());
                    }
                } else {
                    // SAFETY: FFI call to plugin, must ensure input is valid and plugin is trusted.
                    let plugin_output = unsafe { (vtable.run)(&input) };
                    println!("[DEBUG] Echo plugin_output.text ptr: {:?}", plugin_output.text);
                    if !plugin_output.text.is_null() {
                        let c_str = unsafe { std::ffi::CStr::from_ptr(plugin_output.text) };
                        println!("[DEBUG] Echo plugin_output.text string: {}", c_str.to_string_lossy());
                        let out_str = c_str.to_string_lossy().to_string();
                        output = Some(out_str.clone());
                        outputs.insert(node_id.clone(), out_str);
                        unsafe { (vtable.free_output)(plugin_output) };
                    } else {
                        println!("[DEBUG] Echo plugin_output.text is null");
                        output = None;
                        last_err = Some("Plugin returned no output (invalid input?)".to_string());
                    }
                }
                // Save to cache if cache_key is set
                if let Some(ref path) = cache_path {
                    if let Some(ref out_val) = output {
                        if let Err(e) = std::fs::write(path, serde_json::to_string(out_val).unwrap_or_default()) {
                            logs.push(StepLog {
                                step: node_id[4..].parse().unwrap_or(0),
                                runner: step.run.clone(),
                                input: params.clone(),
                                output: None,
                                error: Some(format!("Failed to write cache: {e}")),
                                attempt,
                                input_type: None,
                                output_type: None,
                                validation: Some("error".into()),
                            });
                        } else {
                            println!("STEP {}: cache saved to {}", step.run, path);
                            cache_status = Some("saved".to_string());
                        }
                    }
                }
                break;
            }
        }
        let duration = start_time.elapsed();
        logs.push(StepLog {
            step: node_id[4..].parse().unwrap_or(0),
            runner: step.run.clone(),
            input: params.clone(),
            output: output.clone(),
            error: last_err.clone(),
            attempt,
            input_type,
            output_type,
            validation: cache_status.clone(),
        });
        if last_err.is_some() {
            break;
        }
    }
    Ok(logs)
}

fn substitute_params(params: &mut serde_yaml::Value, outputs: &HashMap<String, String>) {
    match params {
        serde_yaml::Value::String(s) => {
            *s = substitute_vars(s, outputs);
        }
        serde_yaml::Value::Mapping(map) => {
            for v in map.values_mut() {
                substitute_params(v, outputs);
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for v in seq.iter_mut() {
                substitute_params(v, outputs);
            }
        }
        _ => {}
    }
}

fn substitute_vars(s: &str, outputs: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (k, v) in outputs {
        let var = format!("${{{k}.output}}");
        if result.contains(&var) {
            result = result.replace(&var, v);
        }
    }
    result
}

fn build_plugin_input(params: &serde_yaml::Value) -> PluginInput {
    // For now, if there's an 'input' key, use it as text. Extend as needed.
    if let Some(map) = params.as_mapping() {
        if let Some(val) = map.get(serde_yaml::Value::from("input")) {
            if let Some(s) = val.as_str() {
                match CString::new(s) {
                    Ok(cstr) => return PluginInput { text: cstr.into_raw() },
                    Err(_) => return PluginInput { text: std::ptr::null_mut() },
                }
            }
        }
    }
    // Fallback: pass empty string
    match CString::new("") {
        Ok(cstr) => PluginInput { text: cstr.into_raw() },
        Err(_) => PluginInput { text: std::ptr::null_mut() },
    }
}
