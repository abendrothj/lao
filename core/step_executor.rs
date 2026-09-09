//! Single-step execution kernel shared by serial and parallel orchestration.
//!
//! `StepExecutor` owns the per-step lifecycle: condition checks, parameter wiring,
//! loop expansion, caching, retries, trust gating, plugin invocation, event emission,
//! and structured logging. Orchestration layers decide *when* and *how concurrently*
//! to call `execute`; the step semantics live here in one place.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::{
    env as std_env, fs, thread,
    time::{Duration, Instant},
};

use crate::execution::{legacy_adapter, StepMetadata};
use crate::plugin_result::PluginRunResult;
use crate::plugins::PluginRegistry;
use crate::trust::TrustPolicy;
use crate::workflow_helpers::*;
use crate::workflow_types::*;

pub(crate) type SharedOutputs = Arc<Mutex<HashMap<String, String>>>;
pub(crate) type SharedLogs = Arc<Mutex<Vec<StepLog>>>;
pub(crate) type SharedRegistry = Arc<Mutex<PluginRegistry>>;

/// Shared, thread-safe state threaded through every step of one workflow run.
pub struct StepExecutor {
    registry: SharedRegistry,
    trust: TrustPolicy,
    outputs: SharedOutputs,
    logs: SharedLogs,
    step_counter: Arc<AtomicUsize>,
    cache_dir: String,
}

impl StepExecutor {
    pub fn new(
        registry: SharedRegistry,
        trust: TrustPolicy,
        outputs: SharedOutputs,
        logs: SharedLogs,
        step_counter: Arc<AtomicUsize>,
    ) -> Self {
        let cache_dir = std_env::var("LAO_CACHE_DIR").unwrap_or_else(|_| "cache".to_string());
        Self {
            registry,
            trust,
            outputs,
            logs,
            step_counter,
            cache_dir,
        }
    }

    pub fn registry(&self) -> &SharedRegistry {
        &self.registry
    }

    fn push_log(&self, log: StepLog) {
        self.logs.lock().expect("logs mutex poisoned").push(log);
    }

    /// Execute a single workflow step end to end, emitting events and recording a log.
    pub fn execute(&self, node_id: String, step: WorkflowStep, event_tx: &Sender<StepEvent>) {
        let step_idx = self.step_counter.fetch_add(1, Ordering::SeqCst);

        let should_execute = {
            let logs_guard = self.logs.lock().expect("logs mutex poisoned");
            let dependent_step = step.depends_on.as_ref().and_then(|deps| deps.first());
            should_execute_step(&step, &logs_guard, dependent_step.map(|s| s.as_str()))
        };

        if !should_execute {
            let _ = event_tx.send(StepEvent {
                step: step_idx,
                step_id: node_id.clone(),
                runner: step.run.clone(),
                status: "skipped".to_string(),
                attempt: 1,
                message: Some("condition not met".to_string()),
                output: None,
                error: None,
            });
            self.push_log(StepLog {
                step: step_idx,
                step_id: node_id,
                runner: step.run.clone(),
                input: step.params.clone(),
                output: Some("skipped due to condition".to_string()),
                error: None,
                attempt: 1,
                input_type: None,
                output_type: None,
                validation: Some("skipped".to_string()),
            });
            return;
        }

        // Build params with outputs from previous steps.
        let mut params = step.params.clone();
        {
            let outputs_guard = self.outputs.lock().expect("outputs mutex poisoned");
            substitute_params(&mut params, &outputs_guard);

            if let Some(input_from) = &step.input_from {
                if let Some(step_output) = outputs_guard.get(input_from) {
                    if let Some(mapping) = params.as_mapping_mut() {
                        mapping.insert(
                            serde_yaml::Value::String("input".to_string()),
                            serde_yaml::Value::String(step_output.clone()),
                        );
                    } else {
                        let mut new_mapping = serde_yaml::Mapping::new();
                        new_mapping.insert(
                            serde_yaml::Value::String("input".to_string()),
                            serde_yaml::Value::String(step_output.clone()),
                        );
                        params = serde_yaml::Value::Mapping(new_mapping);
                    }
                }
            }
        }

        if step.for_each.is_some() {
            self.execute_loop_step(step_idx, node_id, step, params, event_tx);
            return;
        }

        self.execute_plugin_step(step_idx, node_id, step, params, event_tx);
    }

    fn execute_loop_step(
        &self,
        step_idx: usize,
        node_id: String,
        step: WorkflowStep,
        params: serde_yaml::Value,
        event_tx: &Sender<StepEvent>,
    ) {
        let loop_config = step.for_each.as_ref().expect("checked by caller");
        let outputs_map: HashMap<String, String> = {
            let outputs_guard = self.outputs.lock().expect("outputs mutex poisoned");
            outputs_guard.clone()
        };

        let _ = event_tx.send(StepEvent {
            step: step_idx,
            step_id: node_id.clone(),
            runner: step.run.clone(),
            status: "running".to_string(),
            attempt: 1,
            message: Some("Executing loop".to_string()),
            output: None,
            error: None,
        });

        match execute_with_loop(&step, loop_config, &params, &outputs_map, &self.registry) {
            Ok(loop_results) => {
                let output_str =
                    serde_json::to_string(&loop_results).unwrap_or_else(|_| "[]".to_string());

                let _ = event_tx.send(StepEvent {
                    step: step_idx,
                    step_id: node_id.clone(),
                    runner: step.run.clone(),
                    status: "success".to_string(),
                    attempt: 1,
                    message: Some(format!("Loop completed: {} iterations", loop_results.len())),
                    output: Some(output_str.clone()),
                    error: None,
                });

                self.outputs
                    .lock()
                    .expect("outputs mutex poisoned")
                    .insert(node_id.clone(), output_str.clone());

                self.push_log(StepLog {
                    step: step_idx,
                    step_id: node_id,
                    runner: step.run.clone(),
                    input: params,
                    output: Some(output_str),
                    error: None,
                    attempt: 1,
                    input_type: None,
                    output_type: None,
                    validation: Some(format!("loop:{} iterations", loop_results.len())),
                });
            }
            Err(e) => {
                let _ = event_tx.send(StepEvent {
                    step: step_idx,
                    step_id: node_id.clone(),
                    runner: step.run.clone(),
                    status: "error".to_string(),
                    attempt: 1,
                    message: None,
                    output: None,
                    error: Some(e.clone()),
                });

                self.push_log(StepLog {
                    step: step_idx,
                    step_id: node_id,
                    runner: step.run.clone(),
                    input: params,
                    output: None,
                    error: Some(e),
                    attempt: 1,
                    input_type: None,
                    output_type: None,
                    validation: None,
                });
            }
        }
    }

    fn execute_plugin_step(
        &self,
        step_idx: usize,
        node_id: String,
        step: WorkflowStep,
        params: serde_yaml::Value,
        event_tx: &Sender<StepEvent>,
    ) {
        let plugin_input = build_plugin_input(&params);
        let input_text = plugin_input_text(&params);

        let plugin_info = {
            let reg_guard = self
                .registry
                .lock()
                .expect("plugin registry mutex poisoned");
            reg_guard
                .get(&step.run)
                .map(|p| (p.info.name.clone(), p.info.version.clone()))
        };

        let Some((_, plugin_version)) = plugin_info else {
            let _ = event_tx.send(StepEvent {
                step: step_idx,
                step_id: node_id.clone(),
                runner: step.run.clone(),
                status: "error".to_string(),
                attempt: 1,
                message: Some(format!("Plugin '{}' not found", step.run)),
                output: None,
                error: Some(format!("Plugin '{}' not found", step.run)),
            });
            self.push_log(StepLog {
                step: step_idx,
                step_id: node_id,
                runner: step.run.clone(),
                input: params,
                output: None,
                error: Some("Plugin not found".to_string()),
                attempt: 1,
                input_type: None,
                output_type: None,
                validation: None,
            });
            return;
        };

        let plugin_name = step.run.clone();
        let max_attempts = step.retries.unwrap_or(1) + 1;
        let mut last_error = None;

        let mut cache_status = None;
        let cache_key_effective = step
            .cache_key
            .clone()
            .unwrap_or_else(|| compute_default_cache_key(&step, &plugin_version));
        let cache_path = format!("{}/{}.json", self.cache_dir, cache_key_effective);

        if let Ok(cached) = fs::read_to_string(&cache_path) {
            if let Ok(cached_output) = serde_json::from_str::<String>(&cached) {
                self.outputs
                    .lock()
                    .expect("outputs mutex poisoned")
                    .insert(node_id.clone(), cached_output.clone());
                let _ = event_tx.send(StepEvent {
                    step: step_idx,
                    step_id: node_id.clone(),
                    runner: plugin_name.clone(),
                    status: "cache".to_string(),
                    attempt: 1,
                    message: Some("cache hit".to_string()),
                    output: Some(cached_output.clone()),
                    error: None,
                });
                self.push_log(StepLog {
                    step: step_idx,
                    step_id: node_id,
                    runner: plugin_name,
                    input: params,
                    output: Some(cached_output),
                    error: None,
                    attempt: 1,
                    input_type: None,
                    output_type: None,
                    validation: Some("cache".to_string()),
                });
                return;
            }
        }

        for attempt in 1..=max_attempts {
            let _ = event_tx.send(StepEvent {
                step: step_idx,
                step_id: node_id.clone(),
                runner: plugin_name.clone(),
                status: "running".to_string(),
                attempt,
                message: if attempt > 1 {
                    Some("retrying".to_string())
                } else {
                    None
                },
                output: None,
                error: None,
            });

            let attempt_start = Instant::now();
            let run_result: PluginRunResult = {
                if let Err(e) = self.trust.validate_step_input(&plugin_name, &input_text) {
                    PluginRunResult::runtime_error(e)
                } else {
                    let reg_guard = self
                        .registry
                        .lock()
                        .expect("plugin registry mutex poisoned");
                    if let Some(plugin) = reg_guard.get(&plugin_name) {
                        plugin.run_plugin(&plugin_input)
                    } else {
                        PluginRunResult::runtime_error(format!(
                            "Plugin '{}' not found",
                            plugin_name
                        ))
                    }
                }
            };

            // Adapt the plugin's ABI-derived outcome into the structured StepResult
            // model. This is purely additive: `step_result`'s success/failure and
            // output/error strings are derived identically to the legacy `run_result`
            // values they replace below, so observable behavior is unchanged.
            let step_result = legacy_adapter::adapt(
                run_result,
                StepMetadata {
                    plugin_name: plugin_name.clone(),
                    plugin_version: Some(plugin_version.clone()),
                    attempt,
                    duration_ms: attempt_start.elapsed().as_millis() as u64,
                    cache_hit: false,
                },
            );

            if step_result.is_success() {
                let output_str = step_result.primary_output_text().unwrap_or_default();
                self.outputs
                    .lock()
                    .expect("outputs mutex poisoned")
                    .insert(node_id.clone(), output_str.clone());

                if step.cache_key.is_some() {
                    fs::create_dir_all(&self.cache_dir).ok();
                    if fs::write(
                        &cache_path,
                        serde_json::to_string(&output_str).unwrap_or_default(),
                    )
                    .is_ok()
                    {
                        cache_status = Some("saved".to_string());
                    }
                }

                let _ = event_tx.send(StepEvent {
                    step: step_idx,
                    step_id: node_id.clone(),
                    runner: plugin_name.clone(),
                    status: "success".to_string(),
                    attempt,
                    message: None,
                    output: Some(output_str.clone()),
                    error: None,
                });

                self.push_log(StepLog {
                    step: step_idx,
                    step_id: node_id,
                    runner: plugin_name,
                    input: params,
                    output: Some(output_str),
                    error: None,
                    attempt,
                    input_type: None,
                    output_type: None,
                    validation: cache_status,
                });
                return;
            } else {
                let output_str = step_result.display_error();
                last_error = Some(output_str.clone());
                let _ = event_tx.send(StepEvent {
                    step: step_idx,
                    step_id: node_id.clone(),
                    runner: plugin_name.clone(),
                    status: "error".to_string(),
                    attempt,
                    message: Some("attempt failed".to_string()),
                    output: None,
                    error: Some(output_str),
                });

                if attempt < max_attempts {
                    let retry_delay = step.retry_delay.unwrap_or(1000);
                    thread::sleep(Duration::from_millis(retry_delay));
                }
            }
        }

        if let Some(error) = last_error {
            self.push_log(StepLog {
                step: step_idx,
                step_id: node_id,
                runner: plugin_name,
                input: params,
                output: None,
                error: Some(error),
                attempt: max_attempts,
                input_type: None,
                output_type: None,
                validation: None,
            });
        }
    }
}

/// Execute a step once per loop item, optionally collecting per-iteration outputs.
pub fn execute_with_loop(
    step: &WorkflowStep,
    loop_config: &LoopConfig,
    base_params: &serde_yaml::Value,
    outputs: &HashMap<String, String>,
    registry: &SharedRegistry,
) -> Result<Vec<String>, String> {
    let items = match &loop_config.items {
        LoopItems::Array(arr) => arr.clone(),
        LoopItems::Reference(ref_path) => {
            if let Some(output_str) = outputs.get(ref_path) {
                serde_json::from_str::<Vec<serde_yaml::Value>>(output_str)
                    .map_err(|e| format!("Failed to parse loop items from {}: {}", ref_path, e))?
            } else {
                return Err(format!(
                    "Loop reference '{}' not found in outputs",
                    ref_path
                ));
            }
        }
    };

    let mut results = Vec::new();
    let chunk_size = loop_config.max_parallel;

    for chunk in items.chunks(chunk_size) {
        let mut handles = Vec::new();

        for item in chunk {
            let step_clone = step.clone();
            let mut params = base_params.clone();
            let registry_clone = registry.clone();
            let item_clone = item.clone();
            let var_name = loop_config.var.clone();

            let handle = std::thread::spawn(move || {
                if let Some(mapping) = params.as_mapping_mut() {
                    mapping.insert(serde_yaml::Value::String(var_name), item_clone);
                }

                let plugin_input = build_plugin_input(&params);
                let reg_guard = registry_clone
                    .lock()
                    .expect("plugin registry mutex poisoned");
                let plugin_opt = reg_guard.get(&step_clone.run);

                if let Some(plugin) = plugin_opt {
                    let result = plugin.run_plugin(&plugin_input);
                    if result.is_success() {
                        Ok(result.output.unwrap_or_default())
                    } else {
                        Err(result.display_error())
                    }
                } else {
                    Err(format!("Plugin '{}' not found", step_clone.run))
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            match handle.join() {
                Ok(Ok(output)) => {
                    if loop_config.collect_results {
                        results.push(output);
                    }
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err("Thread panicked during loop execution".to_string()),
            }
        }
    }

    if loop_config.collect_results {
        Ok(results)
    } else {
        Ok(vec![results.last().cloned().unwrap_or_default()])
    }
}

pub(crate) fn plugin_input_text(params: &serde_yaml::Value) -> String {
    if let Some(mapping) = params.as_mapping() {
        if let Some(input_val) = mapping.get("input") {
            if let Some(s) = input_val.as_str() {
                return s.to_string();
            }
        }
    }
    serde_yaml::to_string(params).unwrap_or_default()
}
