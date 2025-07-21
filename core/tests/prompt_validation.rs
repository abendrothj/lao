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
} 