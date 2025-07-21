use lao_plugin_api::{Plugin, PluginInput, PluginOutput};
use std::process::Command;

pub struct WhisperPlugin;

impl Plugin for WhisperPlugin {
    fn run(&self, input: PluginInput) -> Result<PluginOutput, String> {
        let audio_path = input.audio.ok_or("Missing audio input")?;
        let output = Command::new("./whisper.cpp")
            .arg(&audio_path)
            .output()
            .map_err(|e| format!("Failed to run whisper.cpp: {}", e))?;
        if !output.status.success() {
            return Err(format!("whisper.cpp failed: {}", String::from_utf8_lossy(&output.stderr)));
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
    Box::into_raw(Box::new(WhisperPlugin))
} 