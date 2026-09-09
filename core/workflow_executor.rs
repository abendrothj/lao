//! Unified workflow execution entry point.
//!
//! Serial vs parallel orchestration share the same step lifecycle; parallel level
//! scheduling lives in `workflow_parallel` until a dedicated serial backend is needed.

use crate::workflow_parallel::run_workflow_with_options;
use crate::workflow_types::{StepEvent, StepLog};

/// Controls how workflows are executed and persisted.
#[derive(Debug, Clone)]
pub struct ExecutionOptions {
    /// When true, independent DAG levels run concurrently (default).
    pub parallel: bool,
    /// Persist run state to `LAO_STATE_DIR` / `state_dir`.
    pub record_state: bool,
    pub state_dir: String,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            parallel: true,
            record_state: true,
            state_dir: std::env::var("LAO_STATE_DIR")
                .unwrap_or_else(|_| "workflow_states".to_string()),
        }
    }
}

/// Shared workflow runner used by CLI, library, and scheduler paths.
pub struct WorkflowExecutor {
    options: ExecutionOptions,
}

impl WorkflowExecutor {
    pub fn new(options: ExecutionOptions) -> Self {
        Self { options }
    }

    pub fn with_defaults() -> Self {
        Self::new(ExecutionOptions::default())
    }

    pub fn options(&self) -> &ExecutionOptions {
        &self.options
    }

    pub fn run<F>(&self, path: &str, on_event: F) -> Result<Vec<StepLog>, String>
    where
        F: FnMut(StepEvent) + Send,
    {
        run_workflow_with_options(
            path,
            self.options.parallel,
            self.options.record_state,
            &self.options.state_dir,
            on_event,
        )
    }
}

pub fn run_workflow<F>(path: &str, on_event: F) -> Result<Vec<StepLog>, String>
where
    F: FnMut(StepEvent) + Send,
{
    WorkflowExecutor::with_defaults().run(path, on_event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_are_parallel_with_recording() {
        let opts = ExecutionOptions::default();
        assert!(opts.parallel);
        assert!(opts.record_state);
    }

    #[test]
    fn executor_exposes_its_options() {
        let opts = ExecutionOptions {
            parallel: false,
            record_state: false,
            state_dir: "custom_dir".to_string(),
        };
        let executor = WorkflowExecutor::new(opts);
        assert!(!executor.options().parallel);
        assert!(!executor.options().record_state);
        assert_eq!(executor.options().state_dir, "custom_dir");
    }

    #[test]
    fn state_dir_honors_env_override() {
        // SAFETY: single-threaded test, value restored immediately.
        let previous = std::env::var("LAO_STATE_DIR").ok();
        std::env::set_var("LAO_STATE_DIR", "env_state_dir");
        let opts = ExecutionOptions::default();
        assert_eq!(opts.state_dir, "env_state_dir");
        match previous {
            Some(v) => std::env::set_var("LAO_STATE_DIR", v),
            None => std::env::remove_var("LAO_STATE_DIR"),
        }
    }
}
