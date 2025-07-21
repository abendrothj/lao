// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// --- Workflow Engine (Step 2) ---
use serde::{Deserialize};
use std::fs;
use std::process::Command;
use std::collections::HashMap;
use std::time::Instant;
use std::{thread, time::Duration};
use std::env as std_env;
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
            cmd.arg(format!("--{:?}", k)).arg(format!("{:?}", v));
        }
    }
    // Run the command and capture output
    let output = cmd.output().map_err(|e| format!("Failed to run {}: {}", runner, e))?;
    if !output.status.success() {
        return Err(format!("{} failed: {}", runner, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn build_dag(steps: &[WorkflowStep]) -> Result<Vec<DagNode>, String> {
    let mut nodes = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        let id = format!("step{}", i + 1);
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

fn topo_sort(nodes: &[DagNode]) -> Result<Vec<String>, String> {
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
        let sig = plugin_registry.get(&step.run).map(|p| p.io_signature());
        let input_type = sig.as_ref().map(|s| s.input_type.clone()).unwrap_or(plugins::PluginInputType::Any);
        let output_type = sig.as_ref().map(|s| s.output_type.clone()).unwrap_or(plugins::PluginInputType::Any);
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
    let mut plugin_registry = PluginRegistry::default_registry();
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
        let node = node_map.get(&node_id).unwrap();
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
        let mut input_type = None;
        let mut output_type = None;
        let mut validation: Option<String> = Some("ok".into());
        // Caching logic
        let cache_dir = std_env::var("LAO_CACHE_DIR").unwrap_or_else(|_| "cache".to_string());
        std::fs::create_dir_all(&cache_dir).ok();
        let cache_key = step.cache_key.as_ref().map(|k| k.clone());
        let cache_path = cache_key.as_ref().map(|k| format!("{}/{}.json", cache_dir, k));
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
                let delay = retry_delay * 2u64.pow((try_num - 2) as u32);
                println!("STEP {}: retry {} after {}ms", step.run, try_num, delay);
                thread::sleep(Duration::from_millis(delay));
            }
            if let Some(plugin) = plugin_registry.get_mut(&step.run) {
                let sig = plugin.io_signature();
                input_type = Some(sig.input_type.clone());
                output_type = Some(sig.output_type.clone());
                let input = build_plugin_input(&params);
                let now = chrono::Utc::now();
                println!("STEP {}: init at {}", step.run, now);
                if let Err(e) = plugin.init(PluginConfig { parameters: HashMap::new(), verbose: false }) {
                    println!("STEP {}: init error: {:?}", step.run, e);
                }
                let now = chrono::Utc::now();
                println!("STEP {}: pre_execute at {}", step.run, now);
                if let Err(e) = plugin.pre_execute() {
                    println!("STEP {}: pre_execute error: {:?}", step.run, e);
                }
                match plugin.execute(input) {
                    Ok(plugin_output) => {
                        let now = chrono::Utc::now();
                        println!("STEP {}: post_execute at {}", step.run, now);
                        if let Err(e) = plugin.post_execute() {
                            println!("STEP {}: post_execute error: {:?}", step.run, e);
                        }
                        let out_str = format!("{:?}", plugin_output);
                        output = Some(out_str.clone());
                        outputs.insert(node_id.clone(), out_str.clone());
                        last_err = None;
                        let now = chrono::Utc::now();
                        println!("STEP {}: shutdown at {}", step.run, now);
                        if let Err(e) = plugin.shutdown() {
                            println!("STEP {}: shutdown error: {:?}", step.run, e);
                        }
                        // Save to cache if cache_key is set
                        if let Some(ref path) = cache_path {
                            std::fs::write(path, serde_json::to_string(&out_str).unwrap_or_default()).ok();
                            println!("STEP {}: cache saved to {}", step.run, path);
                            cache_status = Some("saved".to_string());
                        }
                        break;
                    }
                    Err(e) => {
                        last_err = Some(format!("Plugin error: {:?}", e));
                        println!("STEP {}: error on attempt {}: {:?}", step.run, try_num, e);
                        let now = chrono::Utc::now();
                        println!("STEP {}: post_execute at {}", step.run, now);
                        if let Err(e) = plugin.post_execute() {
                            println!("STEP {}: post_execute error: {:?}", step.run, e);
                        }
                        let now = chrono::Utc::now();
                        println!("STEP {}: shutdown at {}", step.run, now);
                        if let Err(e) = plugin.shutdown() {
                            println!("STEP {}: shutdown error: {:?}", step.run, e);
                        }
                        if try_num == retries {
                            break;
                        }
                    }
                }
            } else {
                match run_model_runner(&step.run, params.clone()) {
                    Ok(out) => {
                        output = Some(out.clone());
                        outputs.insert(node_id.clone(), out.clone());
                        last_err = None;
                        // Save to cache if cache_key is set
                        if let Some(ref path) = cache_path {
                            std::fs::write(path, serde_json::to_string(&out).unwrap_or_default()).ok();
                            println!("STEP {}: cache saved to {}", step.run, path);
                            cache_status = Some("saved".to_string());
                        }
                        break;
                    }
                    Err(e) => {
                        last_err = Some(e.clone());
                        println!("STEP {}: error on attempt {}: {}", step.run, try_num, e);
                        if try_num == retries {
                            break;
                        }
                    }
                }
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
        let var = format!("${{{}.output}}", k);
        if result.contains(&var) {
            result = result.replace(&var, v);
        }
    }
    result
}

fn build_plugin_input(params: &serde_yaml::Value) -> PluginInput {
    // For now, if there's an 'input' key, use it as Text. Extend as needed.
    if let Some(map) = params.as_mapping() {
        if let Some(val) = map.get(&serde_yaml::Value::from("input")) {
            if let Some(s) = val.as_str() {
                return PluginInput::Text(s.to_string());
            }
        }
    }
    // Fallback: pass the whole params as JSON
    PluginInput::Json(serde_json::to_value(params).unwrap_or_default())
}
