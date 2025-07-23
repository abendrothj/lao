use lao_orchestrator_core::plugins::{PluginRegistry, PluginInput, PluginOutput};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct PromptPair {
    prompt: String,
    workflow: String,
}

fn normalize_yaml(yaml: &str) -> serde_yaml::Value {
    serde_yaml::from_str(yaml).unwrap_or(serde_yaml::Value::Null)
}

fn test_missing_plugin_manifest() {
    let plugin_dir = "../../plugins/EchoPlugin";
    let manifest_path = std::path::Path::new(plugin_dir).join("plugin.yaml");
    let orig = std::fs::read_to_string(&manifest_path).ok();
    // Temporarily remove manifest
    if manifest_path.exists() {
        std::fs::rename(&manifest_path, manifest_path.with_extension("bak")).unwrap();
    }
    let mut registry = lao_orchestrator_core::plugins::PluginRegistry::dynamic_registry("../../plugins/");
    assert!(registry.get("Echo").is_none(), "Plugin should not load without manifest");
    // Restore manifest
    if let Some(orig) = orig {
        std::fs::write(&manifest_path, orig).unwrap();
    } else if manifest_path.with_extension("bak").exists() {
        std::fs::rename(manifest_path.with_extension("bak"), &manifest_path).unwrap();
    }
}

fn test_malformed_plugin_manifest() {
    let plugin_dir = "../../plugins/EchoPlugin";
    let manifest_path = std::path::Path::new(plugin_dir).join("plugin.yaml");
    let orig = std::fs::read_to_string(&manifest_path).ok();
    std::fs::write(&manifest_path, "not: yaml: [").unwrap();
    let mut registry = lao_orchestrator_core::plugins::PluginRegistry::dynamic_registry("../../plugins/");
    assert!(registry.get("Echo").is_none(), "Plugin should not load with malformed manifest");
    // Restore manifest
    if let Some(orig) = orig {
        std::fs::write(&manifest_path, orig).unwrap();
    }
}

fn test_invalid_workflow_step() {
    let workflow = lao_orchestrator_core::Workflow {
        workflow: "Invalid Step".to_string(),
        steps: vec![lao_orchestrator_core::WorkflowStep {
            run: "NonExistentPlugin".to_string(),
            params: serde_yaml::Value::Null,
            retries: None,
            retry_delay: None,
            cache_key: None,
            input_from: None,
            depends_on: None,
        }],
    };
    let dag = lao_orchestrator_core::build_dag(&workflow.steps).unwrap();
    let registry = lao_orchestrator_core::plugins::PluginRegistry::dynamic_registry("../../plugins/");
    let errors = lao_orchestrator_core::validate_workflow_types(&dag, &registry);
    assert!(!errors.is_empty(), "Should report error for missing plugin");
}

fn test_prompt_to_workflow_failure() {
    let mut registry = lao_orchestrator_core::plugins::PluginRegistry::dynamic_registry("../../plugins/");
    let dispatcher = registry.get_mut("PromptDispatcher").expect("PromptDispatcherPlugin not found");
    let input = lao_orchestrator_core::plugins::PluginInput::Text("nonsense input that should fail".to_string());
    let result = dispatcher.execute(input);
    assert!(result.is_err(), "PromptDispatcher should error on nonsense input");
}

fn main() {
    let path = "../prompt_dispatcher/prompt/prompt_library.json";
    let data = fs::read_to_string(path).expect("Failed to read prompt_library.json");
    let pairs: Vec<PromptPair> = serde_json::from_str(&data).expect("Failed to parse prompt_library.json");
    let mut registry = PluginRegistry::default_registry();
    let dispatcher = registry.get_mut("PromptDispatcher").expect("PromptDispatcherPlugin not found");
    let mut passed = 0;
    let mut failed = 0;
    for (i, pair) in pairs.iter().enumerate() {
        println!("\nTest {}: {}", i + 1, pair.prompt);
        let result = dispatcher.execute(PluginInput::Text(pair.prompt.clone()));
        match result {
            Ok(PluginOutput::Text(generated)) => {
                let expected_norm = normalize_yaml(&pair.workflow);
                let generated_norm = normalize_yaml(&generated);
                if expected_norm == generated_norm {
                    println!("  ✅ PASS");
                    passed += 1;
                } else {
                    println!("  ❌ FAIL");
                    println!("  Expected:\n{}", pair.workflow);
                    println!("  Got:\n{}", generated);
                    failed += 1;
                }
            }
            Ok(_) => {
                println!("  ❌ FAIL: Dispatcher did not return YAML text");
                failed += 1;
            }
            Err(e) => {
                println!("  ❌ FAIL: Dispatcher error: {:?}", e);
                failed += 1;
            }
        }
    }
    println!("\nTest Summary: {} passed, {} failed, {} total", passed, failed, passed + failed);
    if failed > 0 {
        std::process::exit(1);
    }
    test_missing_plugin_manifest();
    test_malformed_plugin_manifest();
    test_invalid_workflow_step();
    test_prompt_to_workflow_failure();
} 