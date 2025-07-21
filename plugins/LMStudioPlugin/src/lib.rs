use lao_plugin_api::{Plugin, PluginInput, PluginOutput};

pub struct LMStudioPlugin;

impl Plugin for LMStudioPlugin {
    fn run(&self, input: PluginInput) -> Result<PluginOutput, String> {
        let prompt = input.text.ok_or("Missing text input")?;
        Ok(PluginOutput {
            text: Some(format!("[LM Studio output for prompt: {}]", prompt)),
            ..Default::default()
        })
    }
}

#[no_mangle]
pub extern "C" fn plugin_entry_point() -> *mut dyn LaoPlugin {
    Box::into_raw(Box::new(LMStudioPlugin))
} 