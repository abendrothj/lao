use lao_plugin_api::{LaoPlugin, PluginInput, PluginOutput};

pub struct PluginTemplate;

impl LaoPlugin for PluginTemplate {
    fn execute(&self, input: PluginInput) -> PluginOutput {
        // Example: echo input as output
        PluginOutput::Text(format!("Echo: {:?}", input))
    }
}

#[no_mangle]
pub extern "C" fn plugin_entry_point() -> *mut dyn LaoPlugin {
    Box::into_raw(Box::new(PluginTemplate))
} 