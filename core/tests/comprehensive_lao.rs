use lao_orchestrator_core::plugins::PluginRegistry;
use lao_plugin_api::{PluginInput, PluginOutput};
use lao_orchestrator_core::{Workflow, WorkflowStep, build_dag, validate_workflow_types, run_workflow_yaml};
use std::fs;
use std::path::Path;

fn test_plugin_loading() {
    // Valid plugin
    let reg = PluginRegistry::dynamic_registry("plugins/");
    println!("Loaded plugins: {:?}", reg.plugins.keys().collect::<Vec<_>>());
    assert!(reg.get("Echo").is_some(), "Echo plugin should load");
    // Missing manifest
    let plugin_dir = "plugins/EchoPlugin";
    let manifest_path = Path::new(plugin_dir).join("plugin.yaml");
    let orig = fs::read_to_string(&manifest_path).ok();
    if manifest_path.exists() {
        fs::rename(&manifest_path, manifest_path.with_extension("bak")).unwrap();
    }
    let reg2 = PluginRegistry::dynamic_registry("plugins/");
    assert!(reg2.get("Echo").is_none(), "Plugin should not load without manifest");
    if let Some(orig) = orig {
        fs::write(&manifest_path, orig).unwrap();
    } else if manifest_path.with_extension("bak").exists() {
        fs::rename(manifest_path.with_extension("bak"), &manifest_path).unwrap();
    }
    // Malformed manifest
    let orig = fs::read_to_string(&manifest_path).ok();
    fs::write(&manifest_path, "not: yaml: [").unwrap();
    let reg3 = PluginRegistry::dynamic_registry("plugins/");
    assert!(reg3.get("Echo").is_none(), "Plugin should not load with malformed manifest");
    if let Some(orig) = orig {
        fs::write(&manifest_path, orig).unwrap();
    }
}

fn test_workflow_execution_success() {
    let workflow = Workflow {
        workflow: "Echo Test".to_string(),
        steps: vec![WorkflowStep {
            run: "Echo".to_string(),
            params: serde_yaml::from_str("input: 'Hello, LAO!'").unwrap(),
            retries: Some(1),
            retry_delay: None,
            cache_key: None,
            input_from: None,
            depends_on: None,
        }],
    };
    let path = "temp_workflow.yaml";
    fs::write(path, serde_yaml::to_string(&workflow).unwrap()).unwrap();
    let logs = run_workflow_yaml(path).unwrap();
    for log in &logs {
        println!("Echo workflow log: step={} runner={} output={:?} error={:?}", log.step, log.runner, log.output, log.error);
    }
    assert!(logs.iter().any(|log| log.output.as_ref().map(|o| o.contains("Hello, LAO!")).unwrap_or(false)), "Echo output should be present");
    fs::remove_file(path).unwrap();
}

fn test_workflow_plugin_missing() {
    let workflow = Workflow {
        workflow: "Missing Plugin".to_string(),
        steps: vec![WorkflowStep {
            run: "NonExistentPlugin".to_string(),
            params: serde_yaml::Value::Null,
            retries: None,
            retry_delay: None,
            cache_key: None,
            input_from: None,
            depends_on: None,
        }],
    };
    let dag = build_dag(&workflow.steps).unwrap();
    let reg = PluginRegistry::dynamic_registry("plugins/");
    let errors = validate_workflow_types(&dag, &reg);
    assert!(!errors.is_empty(), "Should report error for missing plugin");
}

fn test_workflow_invalid_step() {
    let workflow = Workflow {
        workflow: "Invalid Step".to_string(),
        steps: vec![WorkflowStep {
            run: "Echo".to_string(),
            params: serde_yaml::Value::Null, // missing required input
            retries: None,
            retry_delay: None,
            cache_key: None,
            input_from: None,
            depends_on: None,
        }],
    };
    let dag = build_dag(&workflow.steps).unwrap();
    let reg = PluginRegistry::dynamic_registry("plugins/");
    let errors = validate_workflow_types(&dag, &reg);
    // Should not error at type level, but runtime may fail
    let path = "temp_invalid.yaml";
    fs::write(path, serde_yaml::to_string(&workflow).unwrap()).unwrap();
    let logs = run_workflow_yaml(path).unwrap();
    assert!(logs.iter().any(|log| log.error.is_some()), "Should log error for invalid step");
    fs::remove_file(path).unwrap();
}

fn test_prompt_to_workflow_success() {
    let mut reg = PluginRegistry::dynamic_registry("plugins/");
    let dispatcher = reg.plugins.get_mut("PromptDispatcherPlugin").expect("PromptDispatcherPlugin not found");
    let input = PluginInput { text: std::ffi::CString::new("Summarize this Markdown doc and extract key ideas").unwrap().into_raw() };
    let result = unsafe { ((*dispatcher.vtable).run)(&input) };
    let c_str = unsafe { std::ffi::CStr::from_ptr(result.text) };
    let output = c_str.to_string_lossy().to_string();
    unsafe { ((*dispatcher.vtable).free_output)(result) };
    assert!(!output.is_empty(), "PromptDispatcher should return YAML");
}

fn test_prompt_to_workflow_failure() {
    let mut reg = PluginRegistry::dynamic_registry("plugins/");
    let dispatcher = reg.plugins.get_mut("PromptDispatcherPlugin").expect("PromptDispatcherPlugin not found");
    let input = PluginInput { text: std::ffi::CString::new("nonsense input that should fail").unwrap().into_raw() };
    let result = unsafe { ((*dispatcher.vtable).run)(&input) };
    let c_str = unsafe { std::ffi::CStr::from_ptr(result.text) };
    let output = c_str.to_string_lossy().to_string();
    unsafe { ((*dispatcher.vtable).free_output)(result) };
    println!("PromptDispatcherPlugin nonsense input output: '{}'", output);
    assert!(output.contains("error") || output.is_empty(), "PromptDispatcher should error on nonsense input");
}

fn test_caching_and_retries() {
    let workflow = Workflow {
        workflow: "Echo Cache Test".to_string(),
        steps: vec![WorkflowStep {
            run: "Echo".to_string(),
            params: serde_yaml::from_str("input: 'Cache me!'").unwrap(),
            retries: Some(2),
            retry_delay: Some(10),
            cache_key: Some("echo_cache_test".to_string()),
            input_from: None,
            depends_on: None,
        }],
    };
    let path = "temp_cache.yaml";
    fs::write(path, serde_yaml::to_string(&workflow).unwrap()).unwrap();
    // First run: should not hit cache
    let logs1 = run_workflow_yaml(path).unwrap();
    assert!(logs1.iter().any(|log| log.validation.as_deref() == Some("saved")), "Should save to cache");
    // Second run: should hit cache
    let logs2 = run_workflow_yaml(path).unwrap();
    assert!(logs2.iter().any(|log| log.validation.as_deref() == Some("cache")), "Should hit cache");
    fs::remove_file(path).unwrap();
    let cache_path = "../cache/echo_cache_test.json";
    if Path::new(cache_path).exists() {
        fs::remove_file(cache_path).unwrap();
    }
}

fn test_log_output() {
    let workflow = Workflow {
        workflow: "Echo Log Test".to_string(),
        steps: vec![WorkflowStep {
            run: "Echo".to_string(),
            params: serde_yaml::from_str("input: 'Log this!'").unwrap(),
            retries: Some(1),
            retry_delay: None,
            cache_key: None,
            input_from: None,
            depends_on: None,
        }],
    };
    let path = "temp_log.yaml";
    fs::write(path, serde_yaml::to_string(&workflow).unwrap()).unwrap();
    let logs = run_workflow_yaml(path).unwrap();
    for log in &logs {
        println!("Step {}: runner={} output={:?} error={:?} attempt={}", log.step, log.runner, log.output, log.error, log.attempt);
    }
    assert!(logs.iter().any(|log| log.output.as_ref().map(|o| o.contains("Log this!")).unwrap_or(false)), "Log output should be present");
    fs::remove_file(path).unwrap();
}

fn main() {
    println!("[LAO Comprehensive Test Suite]");
    test_plugin_loading();
    println!("- Plugin loading tests passed");
    test_workflow_execution_success();
    println!("- Workflow execution (success) passed");
    test_workflow_plugin_missing();
    println!("- Workflow plugin missing test passed");
    test_workflow_invalid_step();
    println!("- Workflow invalid step test passed");
    test_prompt_to_workflow_success();
    println!("- Prompt-to-workflow (success) passed");
    test_prompt_to_workflow_failure();
    println!("- Prompt-to-workflow (failure) passed");
    test_caching_and_retries();
    println!("- Caching and retries test passed");
    test_log_output();
    println!("- Log output test passed");
    println!("[All comprehensive LAO tests passed]");
} 