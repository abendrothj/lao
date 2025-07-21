use lao_plugin_api::{Plugin, PluginInput, PluginOutput};
use std::process::Command;

pub struct OllamaPlugin;

impl Plugin for OllamaPlugin {
    fn run(&self, input: PluginInput) -> Result<PluginOutput, String> {
        let prompt = input.text.ok_or("Missing text input")?;
        let output = Command::new("ollama")
            .arg("run")
            .arg("mistral")
            .arg(&prompt)
            .output()
            .map_err(|e| format!("Failed to run ollama: {}", e))?;
        if !output.status.success() {
            return Err(format!("ollama failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        let out_str = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(PluginOutput {
            text: Some(out_str),
            ..Default::default()
        })
    }
}

#[no_mangle]
pub extern "C" fn plugin_entry_point() -> *mut dyn LaoPlugin {
    Box::into_raw(Box::new(OllamaPlugin))
} 