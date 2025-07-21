use lao_plugin_api::{Plugin, PluginInput, PluginOutput};

pub struct SummarizerPlugin;

impl Plugin for SummarizerPlugin {
    fn run(&self, input: PluginInput) -> Result<PluginOutput, String> {
        let text = input.text.ok_or("Missing text input")?;
        let summary = ollama_summarize(&text)?;
        Ok(PluginOutput {
            text: Some(summary),
            ..Default::default()
        })
    }
}

fn ollama_summarize(text: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "mistral",
            "prompt": format!("Summarize this:\n\n{}", text),
            "stream": false
        }))
        .send()
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = res.json().map_err(|e| e.to_string())?;
    Ok(json["response"].as_str().unwrap_or("").to_string())
}

// Provide a default PluginOutput for convenience
impl Default for PluginOutput {
    fn default() -> Self {
        PluginOutput {
            text: None,
            ..Default::default()
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_entry_point() -> *mut dyn LaoPlugin {
    Box::into_raw(Box::new(SummarizerPlugin))
} 