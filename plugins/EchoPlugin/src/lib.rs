use lao_plugin_api::{Plugin, PluginInput, PluginOutput};

pub struct EchoPlugin;

impl Plugin for EchoPlugin {
    fn run(&self, input: PluginInput) -> Result<PluginOutput, String> {
        Ok(PluginOutput {
            text: input.text,
            audio: input.audio,
            json: input.json,
            tagged_data: input.tagged_data,
            ..Default::default()
        })
    }
}

#[no_mangle]
pub extern "C" fn plugin_entry_point() -> *mut dyn LaoPlugin {
    Box::into_raw(Box::new(EchoPlugin))
} 