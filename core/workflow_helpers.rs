use lao_plugin_api::PluginInput;
use std::collections::HashMap;
use std::ffi::CString;
use std::ops::Deref;

use crate::workflow_types::*;

pub fn substitute_params(params: &mut serde_yaml::Value, outputs: &HashMap<String, String>) {
    if let Some(mapping) = params.as_mapping_mut() {
        for (_, value) in mapping.iter_mut() {
            if let Some(s) = value.as_str() {
                *value = serde_yaml::Value::String(substitute_vars(s, outputs));
            }
        }
    }
}

pub fn substitute_vars(s: &str, outputs: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (key, value) in outputs {
        let placeholder = format!("${{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

pub struct OwnedPluginInput {
    input: PluginInput,
}

impl OwnedPluginInput {
    pub fn new(text: String) -> Self {
        let c_string = CString::new(text).unwrap_or_else(|_| {
            CString::new("error: invalid input string").expect("static string")
        });
        Self {
            input: PluginInput {
                text: c_string.into_raw(),
            },
        }
    }
}

impl Deref for OwnedPluginInput {
    type Target = PluginInput;

    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl Drop for OwnedPluginInput {
    fn drop(&mut self) {
        if !self.input.text.is_null() {
            unsafe {
                let _ = CString::from_raw(self.input.text);
            }
            self.input.text = std::ptr::null_mut();
        }
    }
}

pub fn build_plugin_input(params: &serde_yaml::Value) -> OwnedPluginInput {
    // Try to extract the "input" field first, fallback to full YAML
    if let Some(mapping) = params.as_mapping() {
        if let Some(input_val) = mapping.get("input") {
            if let Some(input_str) = input_val.as_str() {
                return OwnedPluginInput::new(input_str.to_string());
            }
        }
    }

    // Fallback: serialize the entire params object
    let text = serde_yaml::to_string(params).unwrap_or_default();
    OwnedPluginInput::new(text)
}

// Compute default cache key when user does not provide one.
//
// Keys are derived from the *substituted* runtime params (post `${...}` and
// `input_from` wiring), so two runs with different upstream outputs never share
// a cache entry.
pub fn compute_default_cache_key(
    step: &WorkflowStep,
    params: &serde_yaml::Value,
    plugin_version: &str,
) -> String {
    let params_str = serde_yaml::to_string(params).unwrap_or_default();
    let mut hash: u64 = 1469598103934665603; // FNV-1a 64-bit offset basis
    for b in params_str.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{}-{}-{:x}", step.run, plugin_version, hash)
}

// Evaluate a step condition against execution context
pub fn evaluate_condition(condition: &StepCondition, step_logs: &[StepLog], step_id: &str) -> bool {
    match &condition.condition_type {
        ConditionType::OutputContains => {
            if let Some(log) = step_logs.iter().find(|l| l.step_id == step_id) {
                if let Some(output) = &log.output {
                    match condition.operator {
                        ConditionOperator::Contains => output.contains(&condition.value),
                        ConditionOperator::NotContains => !output.contains(&condition.value),
                        _ => false,
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
        ConditionType::OutputEquals => {
            if let Some(log) = step_logs.iter().find(|l| l.step_id == step_id) {
                if let Some(output) = &log.output {
                    match condition.operator {
                        ConditionOperator::Equals => output == &condition.value,
                        ConditionOperator::NotEquals => output != &condition.value,
                        _ => false,
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
        ConditionType::StatusEquals => {
            if let Some(log) = step_logs.iter().find(|l| l.step_id == step_id) {
                let status = if log.error.is_some() {
                    "error"
                } else {
                    "success"
                };
                match condition.operator {
                    ConditionOperator::Equals => status == condition.value,
                    ConditionOperator::NotEquals => status != condition.value,
                    _ => false,
                }
            } else {
                false
            }
        }
        ConditionType::ErrorContains => {
            if let Some(log) = step_logs.iter().find(|l| l.step_id == step_id) {
                if let Some(error) = &log.error {
                    match condition.operator {
                        ConditionOperator::Contains => error.contains(&condition.value),
                        ConditionOperator::NotContains => !error.contains(&condition.value),
                        _ => false,
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
        ConditionType::PreviousStepStatus => {
            if let Some(prev_log) = step_logs.last() {
                let status = if prev_log.error.is_some() {
                    "error"
                } else {
                    "success"
                };
                match condition.operator {
                    ConditionOperator::Equals => status == condition.value,
                    ConditionOperator::NotEquals => status != condition.value,
                    _ => false,
                }
            } else {
                false
            }
        }
    }
}

// Check if a step should be executed based on its condition
pub fn should_execute_step(
    step: &WorkflowStep,
    step_logs: &[StepLog],
    dependent_step_id: Option<&str>,
) -> bool {
    if let Some(condition) = &step.condition {
        if let Some(dep_id) = dependent_step_id {
            evaluate_condition(condition, step_logs, dep_id)
        } else {
            evaluate_condition(condition, step_logs, &condition.field)
        }
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_vars() {
        let mut outputs = HashMap::new();
        outputs.insert("step1".to_string(), "hello world".to_string());

        let result = substitute_vars("Input: ${step1}", &outputs);
        assert_eq!(result, "Input: hello world");
    }

    #[test]
    fn test_substitute_vars_no_match() {
        let outputs = HashMap::new();
        let result = substitute_vars("Input: ${Missing}", &outputs);
        assert_eq!(result, "Input: ${Missing}");
    }
}
