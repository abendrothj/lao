// --- LAO Workflow Engine ---

// Platform & infrastructure modules
pub mod code_intelligence;
pub mod cross_platform;
pub mod execution;
pub mod path_policy;
pub mod plugin_result;
pub mod plugins;
pub mod scheduler;
pub mod state_manager;
pub mod step_executor;
pub mod trust;
pub mod workflow_executor;
pub mod workflow_state;

// Workflow engine modules
pub mod workflow_dag;
pub mod workflow_exec;
pub mod workflow_helpers;
pub mod workflow_parallel;
pub mod workflow_types;
pub mod workflow_v2;

// Re-export public API to preserve backward compatibility
pub use workflow_dag::*;
pub use workflow_exec::*;
pub use workflow_executor::{ExecutionOptions, WorkflowExecutor};
pub use workflow_helpers::{evaluate_condition, should_execute_step};
pub use workflow_parallel::{run_workflow_with_options, run_workflow_yaml_parallel_with_callback};
pub use workflow_types::*;
