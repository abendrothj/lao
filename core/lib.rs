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
        let mut parents = Vec::new();
        if let Some(input_from) = &step.input_from {
            parents.push(input_from.clone());
        }
        if let Some(depends_on) = &step.depends_on {
            parents.extend(depends_on.clone());
        }
        nodes.push(DagNode {
            id: step.run.clone(),
            step: step.clone(),
            parents,
        });
    }
    Ok(nodes)
}

pub fn topo_sort(nodes: &[DagNode]) -> Result<Vec<String>, String> {
    let mut visited = std::collections::HashSet::new();
    let mut visiting = std::collections::HashSet::new();
    let mut order = Vec::new();
    let node_map: HashMap<String, &DagNode> = nodes.iter().map(|n| (n.id.clone(), n)).collect();

    fn visit(
        n: &DagNode,
        map: &HashMap<String, &DagNode>,
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), String> {
        if visiting.contains(&n.id) {
            return Err(format!("Circular dependency detected involving {}", n.id));
        }
        if visited.contains(&n.id) {
            return Ok(());
        }
        visiting.insert(n.id.clone());
        for parent_id in &n.parents {
            if let Some(parent) = map.get(parent_id) {
                visit(parent, map, visited, visiting, order)?;
            }
        }
        visiting.remove(&n.id);
        visited.insert(n.id.clone());
        order.push(n.id.clone());
        Ok(())
    }

    for node in nodes {
        if !visited.contains(&node.id) {
            visit(node, &node_map, &mut visited, &mut visiting, &mut order)?;
        }
    }
    Ok(order)
}

pub fn validate_workflow_types(
    dag: &[DagNode],
    plugin_registry: &PluginRegistry,
) -> Vec<(usize, String)> {
    let mut errors = Vec::new();
    for (i, node) in dag.iter().enumerate() {
        if let Some(plugin) = plugin_registry.get(&node.step.run) {
            // Basic validation: check if plugin exists
            // In a real implementation, you'd validate input/output types
        } else {
            errors.push((i, format!("Plugin '{}' not found", node.step.run)));
        }
    }
    errors
}

pub fn run_workflow_yaml(path: &str) -> Result<Vec<StepLog>, String> {
    let workflow = load_workflow_yaml(path)?;
    let dag = build_dag(&workflow.steps)?;
    let registry = PluginRegistry::dynamic_registry("../plugins/");
    
    // Validate workflow
    let errors = validate_workflow_types(&dag, &registry);
    if !errors.is_empty() {
        return Err(format!("Workflow validation failed: {:?}", errors));
    }
    
    // Topological sort
    let execution_order = topo_sort(&dag)?;
    
    let mut logs = Vec::new();
    let mut outputs = HashMap::new();
    let start_time = Instant::now();
    
    for (step_idx, node_id) in execution_order.iter().enumerate() {
        let node = dag.iter().find(|n| &n.id == node_id).unwrap();
        let step = &node.step;
        
        // Build input parameters
        let mut params = step.params.clone();
        substitute_params(&mut params, &outputs);
        
        // Build plugin input
        let plugin_input = build_plugin_input(&params);
        
        // Get plugin
        let plugin = registry.get(&step.run)
            .ok_or_else(|| format!("Plugin '{}' not found", step.run))?;
        
        // Run with retries
        let mut last_error = None;
        let max_attempts = step.retries.unwrap_or(1) + 1;
        
        for attempt in 1..=max_attempts {
            let attempt_start = Instant::now();
            
            // Check cache first
            let mut cache_status = None;
            if let Some(cache_key) = &step.cache_key {
                let cache_dir = std_env::var("LAO_CACHE_DIR").unwrap_or_else(|_| "cache".to_string());
                let cache_path = format!("{}/{}.json", cache_dir, cache_key);
                if let Ok(cached) = fs::read_to_string(&cache_path) {
                    if let Ok(cached_output) = serde_json::from_str::<String>(&cached) {
                        cache_status = Some("cache".to_string());
                        outputs.insert(node_id.clone(), cached_output.clone());
                        logs.push(StepLog {
                            step: step_idx,
                            runner: step.run.clone(),
                            input: params.clone(),
                            output: Some(cached_output),
                            error: None,
                            attempt,
                            input_type: None,
                            output_type: None,
                            validation: cache_status,
                        });
                        break;
                    }
                }
            }
            
            // Run plugin
            let result = unsafe { ((*plugin.vtable).run)(&plugin_input) };
            let output_str = unsafe { 
                std::ffi::CStr::from_ptr(result.text).to_string_lossy().to_string() 
            };
            unsafe { ((*plugin.vtable).free_output)(result) };
            
            if !output_str.is_empty() && !output_str.contains("error") {
                // Success
                outputs.insert(node_id.clone(), output_str.clone());
                
                // Save to cache
                if let Some(cache_key) = &step.cache_key {
                    let cache_dir = std_env::var("LAO_CACHE_DIR").unwrap_or_else(|_| "cache".to_string());
                    fs::create_dir_all(&cache_dir).ok();
                    let cache_path = format!("{}/{}.json", cache_dir, cache_key);
                    if let Ok(cache_json) = serde_json::to_string(&output_str) {
                        fs::write(&cache_path, cache_json).ok();
                        cache_status = Some("saved".to_string());
                    }
                }
                
                logs.push(StepLog {
                    step: step_idx,
                    runner: step.run.clone(),
                    input: params.clone(),
                    output: Some(output_str),
                    error: None,
                    attempt,
                    input_type: None,
                    output_type: None,
                    validation: cache_status,
                });
                break;
            } else {
                // Error
                last_error = Some(output_str);
                
                if attempt < max_attempts {
                    let retry_delay = step.retry_delay.unwrap_or(1000);
                    let delay = if attempt > 1 {
                        retry_delay * 2u64.pow(attempt - 2)
                    } else {
                        retry_delay
                    };
                    thread::sleep(Duration::from_millis(delay));
                }
            }
        }
        
        if let Some(error) = last_error {
            logs.push(StepLog {
                step: step_idx,
                runner: step.run.clone(),
                input: params.clone(),
                output: None,
                error: Some(error),
                attempt: max_attempts,
                input_type: None,
                output_type: None,
                validation: None,
            });
            // Continue execution instead of failing the entire workflow
            // This allows tests to check for errors in the logs
        }
    }
    
    let duration = start_time.elapsed();
    Ok(logs)
}

fn substitute_params(params: &mut serde_yaml::Value, outputs: &HashMap<String, String>) {
    if let Some(mapping) = params.as_mapping_mut() {
        for (_, value) in mapping.iter_mut() {
            if let Some(s) = value.as_str() {
                *value = serde_yaml::Value::String(substitute_vars(s, outputs));
            }
        }
    }
}

fn substitute_vars(s: &str, outputs: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (key, value) in outputs {
        let placeholder = format!("${{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

fn build_plugin_input(params: &serde_yaml::Value) -> PluginInput {
    // Try to extract the "input" field first, fallback to full YAML
    if let Some(mapping) = params.as_mapping() {
        if let Some(input_val) = mapping.get("input") {
            if let Some(input_str) = input_val.as_str() {
                let c_string = CString::new(input_str).unwrap();
                return PluginInput { text: c_string.into_raw() };
            }
        }
    }
    
    // Fallback: serialize the entire params object
    let text = serde_yaml::to_string(params).unwrap_or_default();
    let c_string = CString::new(text).unwrap();
    PluginInput { text: c_string.into_raw() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet_function() {
        let result = greet("World");
        assert_eq!(result, "Hello, World! You've been greeted from Rust!");
    }

    #[test]
    fn test_build_dag_simple() {
        let steps = vec![
            WorkflowStep {
                run: "Echo".to_string(),
                params: serde_yaml::from_str("input: 'hello'").unwrap(),
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: None,
                depends_on: None,
            }
        ];
        
        let dag = build_dag(&steps).unwrap();
        assert_eq!(dag.len(), 1);
        assert_eq!(dag[0].id, "Echo");
        assert_eq!(dag[0].parents.len(), 0);
    }

    #[test]
    fn test_build_dag_with_dependencies() {
        let steps = vec![
            WorkflowStep {
                run: "Step1".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: None,
                depends_on: None,
            },
            WorkflowStep {
                run: "Step2".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: Some("Step1".to_string()),
                depends_on: None,
            }
        ];
        
        let dag = build_dag(&steps).unwrap();
        assert_eq!(dag.len(), 2);
        assert_eq!(dag[1].parents.len(), 1);
        assert_eq!(dag[1].parents[0], "Step1");
    }

    #[test]
    fn test_topo_sort_simple() {
        let steps = vec![
            WorkflowStep {
                run: "A".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: None,
                depends_on: None,
            },
            WorkflowStep {
                run: "B".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: Some("A".to_string()),
                depends_on: None,
            }
        ];
        
        let dag = build_dag(&steps).unwrap();
        let order = topo_sort(&dag).unwrap();
        assert_eq!(order, vec!["A", "B"]);
    }

    #[test]
    fn test_topo_sort_circular_dependency() {
        let steps = vec![
            WorkflowStep {
                run: "A".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: Some("B".to_string()),
                depends_on: None,
            },
            WorkflowStep {
                run: "B".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: Some("A".to_string()),
                depends_on: None,
            }
        ];
        
        let dag = build_dag(&steps).unwrap();
        let result = topo_sort(&dag);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }

    #[test]
    fn test_substitute_vars() {
        let mut outputs = HashMap::new();
        outputs.insert("Echo".to_string(), "hello world".to_string());
        
        let result = substitute_vars("Input: ${Echo}", &outputs);
        assert_eq!(result, "Input: hello world");
    }

    #[test]
    fn test_substitute_vars_no_match() {
        let outputs = HashMap::new();
        let result = substitute_vars("Input: ${Missing}", &outputs);
        assert_eq!(result, "Input: ${Missing}");
    }
}
