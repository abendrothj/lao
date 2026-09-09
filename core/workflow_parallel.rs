use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use crate::cross_platform::PathUtils;
use crate::plugins::PluginRegistry;
use crate::state_manager::WorkflowStateManager;
use crate::step_executor::StepExecutor;
use crate::trust::TrustPolicy;
use crate::workflow_dag::*;
use crate::workflow_state::{StepResult, StepStatus, WorkflowState};
use crate::workflow_types::*;

// Preserve the historical public path for the loop kernel.
pub use crate::step_executor::execute_with_loop;

// Group nodes by execution level (nodes at same level have no inter-dependencies).
pub fn group_by_execution_levels(dag: &[DagNode]) -> Vec<Vec<String>> {
    let mut levels = Vec::new();
    let mut remaining: std::collections::HashSet<String> =
        dag.iter().map(|n| n.id.clone()).collect();
    let node_map: std::collections::HashMap<String, &DagNode> =
        dag.iter().map(|n| (n.id.clone(), n)).collect();

    while !remaining.is_empty() {
        let mut current_level = Vec::new();
        for node_id in remaining.iter() {
            if let Some(node) = node_map.get(node_id) {
                let all_parents_done = node
                    .parents
                    .iter()
                    .all(|parent_id| !remaining.contains(parent_id));
                if all_parents_done {
                    current_level.push(node_id.clone());
                }
            }
        }

        if current_level.is_empty() {
            break;
        }

        // Deterministic ordering within a level keeps serial runs reproducible.
        current_level.sort();

        for node_id in &current_level {
            remaining.remove(node_id);
        }

        levels.push(current_level);
    }

    levels
}

fn new_workflow_run_id(path: &str) -> String {
    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    if let Ok(duration) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        duration.as_nanos().hash(&mut h);
    }
    format!("{:016x}", h.finish())
}

fn record_workflow_state(
    state_dir: &str,
    path: &str,
    workflow: &Workflow,
    dag_len: usize,
    logs: &[StepLog],
) {
    let Ok(mut manager) = WorkflowStateManager::new(state_dir) else {
        tracing::warn!("Failed to open workflow state dir: {}", state_dir);
        return;
    };

    let workflow_id = new_workflow_run_id(path);
    let mut state = WorkflowState::new(workflow_id, workflow.workflow.clone(), dag_len);
    state.workflow_path = Some(path.to_string());
    state.start();

    for log in logs {
        let status = if log.error.is_some() {
            StepStatus::Failed
        } else if log.validation.as_deref().is_some_and(|v| v == "skipped") {
            StepStatus::Skipped
        } else {
            StepStatus::Success
        };
        state.add_step_result(StepResult {
            step_id: log.step_id.clone(),
            plugin_name: log.runner.clone(),
            status,
            output: log.output.clone(),
            error: log.error.clone(),
            started_at: std::time::SystemTime::now(),
            completed_at: Some(std::time::SystemTime::now()),
            duration_ms: None,
            retry_count: log.attempt.saturating_sub(1),
        });
        if let Some(out) = &log.output {
            state.outputs.insert(log.step_id.clone(), out.clone());
        }
    }

    if logs.iter().any(|l| l.error.is_some()) {
        let msg = logs
            .iter()
            .find_map(|l| l.error.as_ref())
            .cloned()
            .unwrap_or_else(|| "workflow step failed".to_string());
        state.fail(msg);
    } else {
        state.complete();
    }

    if let Err(e) = manager.save_state(&state) {
        tracing::warn!("Failed to persist workflow state: {}", e);
    }
}

/// Back-compat entry point: parallel execution with state recording to `LAO_STATE_DIR`.
pub fn run_workflow_yaml_parallel_with_callback<F>(
    path: &str,
    on_event: F,
) -> Result<Vec<StepLog>, String>
where
    F: FnMut(StepEvent) + Send,
{
    let state_dir =
        std::env::var("LAO_STATE_DIR").unwrap_or_else(|_| "workflow_states".to_string());
    run_workflow_with_options(path, true, true, &state_dir, on_event)
}

/// Unified runner shared by serial and parallel orchestration.
///
/// `parallel` toggles per-level concurrency; `record_state` controls whether a
/// `WorkflowState` is persisted to `state_dir`. Step semantics are identical across
/// both modes because they route through the same `StepExecutor`.
pub fn run_workflow_with_options<F>(
    path: &str,
    parallel: bool,
    record_state: bool,
    state_dir: &str,
    mut on_event: F,
) -> Result<Vec<StepLog>, String>
where
    F: FnMut(StepEvent) + Send,
{
    let workflow = load_workflow_yaml(path)?;
    let trust_policy = TrustPolicy::load_default();
    trust_policy.validate_workflow(&workflow)?;
    let dag = build_dag(&workflow.steps)?;
    let registry = Arc::new(Mutex::new(PluginRegistry::default_registry()));

    {
        let reg_guard = registry.lock().expect("plugin registry mutex poisoned");
        let plugin_count = reg_guard.plugin_count();
        if plugin_count == 0 {
            tracing::error!(" No plugins loaded! Cannot validate workflow.");
            tracing::info!(
                " Expected plugins directory: {}",
                PathUtils::plugin_dir().display()
            );
            tracing::info!(" Make sure plugins are built: bash scripts/build-plugins.sh");
            return Err(format!(
                "No plugins loaded. Plugin directory: {}",
                PathUtils::plugin_dir().display()
            ));
        }

        let errors = validate_workflow_types(&dag, &reg_guard);
        if !errors.is_empty() {
            tracing::error!(" Workflow validation failed with {} errors", errors.len());
            for (step_idx, error_msg) in &errors {
                tracing::error!(" Step {}: {}", step_idx, error_msg);
            }
            return Err(format!("Workflow validation failed: {:?}", errors));
        }

        // Reconcile each plugin's declared manifest capabilities with the trust policy
        // before any step runs (defense in depth alongside per-step input checks).
        for node in &dag {
            if let Some(plugin) = reg_guard.get(&node.step.run) {
                trust_policy
                    .check_manifest_capabilities(&plugin.info.name, &plugin.info.capabilities)?;
            }
        }
    }

    let execution_levels = group_by_execution_levels(&dag);
    let node_map: HashMap<String, &DagNode> = dag.iter().map(|n| (n.id.clone(), n)).collect();

    let outputs = Arc::new(Mutex::new(HashMap::new()));
    let logs_mutex = Arc::new(Mutex::new(Vec::<StepLog>::new()));
    let step_counter = Arc::new(AtomicUsize::new(0));

    let executor = Arc::new(StepExecutor::new(
        registry.clone(),
        trust_policy,
        outputs.clone(),
        logs_mutex.clone(),
        step_counter.clone(),
    ));

    let (event_tx, event_rx) = std::sync::mpsc::channel::<StepEvent>();

    for level in execution_levels {
        if parallel {
            let mut handles = Vec::new();
            for node_id in level {
                let Some(node) = node_map.get(&node_id) else {
                    tracing::error!(" Node '{}' not found in DAG during execution", node_id);
                    continue;
                };
                let step = node.step.clone();
                let exec = executor.clone();
                let tx = event_tx.clone();
                handles.push(std::thread::spawn(move || exec.execute(node_id, step, &tx)));
            }
            for handle in handles {
                if let Err(e) = handle.join() {
                    tracing::error!(" Thread panicked during parallel execution: {:?}", e);
                }
            }
        } else {
            for node_id in level {
                let Some(node) = node_map.get(&node_id) else {
                    tracing::error!(" Node '{}' not found in DAG during execution", node_id);
                    continue;
                };
                let step = node.step.clone();
                executor.execute(node_id, step, &event_tx);
            }
        }
    }

    drop(event_tx);

    let mut collected_events = Vec::new();
    while let Ok(event) = event_rx.try_recv() {
        collected_events.push(event);
    }
    while let Ok(event) = event_rx.recv_timeout(std::time::Duration::from_millis(100)) {
        collected_events.push(event);
    }

    collected_events.sort_by_key(|e| e.step);
    for event in collected_events {
        on_event(event);
    }

    let mut logs = {
        let logs_guard = logs_mutex.lock().expect("logs mutex poisoned");
        logs_guard.clone()
    };

    logs.sort_by_key(|l| l.step);
    if record_state {
        record_workflow_state(state_dir, path, &workflow, dag.len(), &logs);
    }
    Ok(logs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dag_node(id: &str, parents: Vec<&str>, plugin: &str) -> DagNode {
        DagNode {
            id: id.to_string(),
            step: WorkflowStep {
                run: plugin.to_string(),
                params: serde_yaml::Value::Null,
                depends_on: if parents.is_empty() {
                    None
                } else {
                    Some(parents.iter().map(|p| p.to_string()).collect())
                },
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: None,
                for_each: None,
                condition: None,
            },
            parents: parents.iter().map(|p| p.to_string()).collect(),
        }
    }

    #[test]
    fn test_group_single_node() {
        let dag = vec![make_dag_node("a", vec![], "EchoPlugin")];
        let levels = group_by_execution_levels(&dag);
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0], vec!["a"]);
    }

    #[test]
    fn test_group_linear_chain() {
        // a -> b -> c
        let dag = vec![
            make_dag_node("a", vec![], "EchoPlugin"),
            make_dag_node("b", vec!["a"], "EchoPlugin"),
            make_dag_node("c", vec!["b"], "EchoPlugin"),
        ];
        let levels = group_by_execution_levels(&dag);
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0], vec!["a"]);
        assert_eq!(levels[1], vec!["b"]);
        assert_eq!(levels[2], vec!["c"]);
    }

    #[test]
    fn test_group_parallel_nodes() {
        // a and b are independent, c depends on both
        let dag = vec![
            make_dag_node("a", vec![], "EchoPlugin"),
            make_dag_node("b", vec![], "EchoPlugin"),
            make_dag_node("c", vec!["a", "b"], "EchoPlugin"),
        ];
        let levels = group_by_execution_levels(&dag);
        assert_eq!(levels.len(), 2);

        // First level should contain both a and b (order may vary)
        assert_eq!(levels[0].len(), 2);
        assert!(levels[0].contains(&"a".to_string()));
        assert!(levels[0].contains(&"b".to_string()));

        // Second level is just c
        assert_eq!(levels[1], vec!["c"]);
    }

    #[test]
    fn test_group_diamond_dag() {
        //     a
        //    / \
        //   b   c
        //    \ /
        //     d
        let dag = vec![
            make_dag_node("a", vec![], "EchoPlugin"),
            make_dag_node("b", vec!["a"], "EchoPlugin"),
            make_dag_node("c", vec!["a"], "EchoPlugin"),
            make_dag_node("d", vec!["b", "c"], "EchoPlugin"),
        ];
        let levels = group_by_execution_levels(&dag);
        assert_eq!(levels.len(), 3);

        assert_eq!(levels[0], vec!["a"]);
        assert_eq!(levels[1].len(), 2);
        assert!(levels[1].contains(&"b".to_string()));
        assert!(levels[1].contains(&"c".to_string()));
        assert_eq!(levels[2], vec!["d"]);
    }

    #[test]
    fn test_group_empty_dag() {
        let dag: Vec<DagNode> = vec![];
        let levels = group_by_execution_levels(&dag);
        assert_eq!(levels.len(), 0);
    }

    #[test]
    fn test_group_wide_fan_out() {
        // a -> b, c, d, e (all parallel)
        let dag = vec![
            make_dag_node("a", vec![], "EchoPlugin"),
            make_dag_node("b", vec!["a"], "EchoPlugin"),
            make_dag_node("c", vec!["a"], "EchoPlugin"),
            make_dag_node("d", vec!["a"], "EchoPlugin"),
            make_dag_node("e", vec!["a"], "EchoPlugin"),
        ];
        let levels = group_by_execution_levels(&dag);
        assert_eq!(levels.len(), 2);
        assert_eq!(levels[0].len(), 1);
        assert_eq!(levels[1].len(), 4);
    }
}
