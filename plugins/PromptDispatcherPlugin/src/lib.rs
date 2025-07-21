use lao_plugin_api::{PluginInput, PluginOutput, PluginVTable, PluginVTablePtr};
use std::ffi::{CStr, CString};
use std::process::Command;
use std::os::raw::c_char;

unsafe extern "C" fn name() -> *const c_char {
    b"PromptDispatcherPlugin\0".as_ptr() as *const c_char
}

unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    if input.is_null() {
        return PluginOutput { text: std::ptr::null_mut() };
    }
    let c_str = CStr::from_ptr((*input).text);
    let prompt = c_str.to_string_lossy();
    let system_prompt = include_str!("../../../core/prompt_dispatcher/prompt/system_prompt.txt");
    let full_prompt = format!("{}\n{}", system_prompt, prompt);
    let output = Command::new("ollama")
        .arg("run")
        .arg("mistral")
        .arg(&full_prompt)
        .output();
    let text = match output {
        Ok(out) if out.status.success() => {
            CString::new(String::from_utf8_lossy(&out.stdout).to_string()).unwrap().into_raw()
        },
        Ok(out) => {
            CString::new(format!("ollama failed: {}", String::from_utf8_lossy(&out.stderr))).unwrap().into_raw()
        },
        Err(e) => {
            CString::new(format!("Failed to run ollama: {}", e)).unwrap().into_raw()
        }
    };
    PluginOutput { text }
}

unsafe extern "C" fn free_output(output: PluginOutput) {
    if !output.text.is_null() {
        let _ = CString::from_raw(output.text);
    }
}

#[no_mangle]
pub static PLUGIN_VTABLE: PluginVTable = PluginVTable {
    name,
    run,
    free_output,
};

#[no_mangle]
pub extern "C" fn plugin_vtable() -> PluginVTablePtr {
    &PLUGIN_VTABLE
} 