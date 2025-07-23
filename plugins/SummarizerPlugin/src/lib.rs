use lao_plugin_api::{PluginInput, PluginOutput, PluginVTable, PluginVTablePtr};
use std::ffi::{CStr, CString};
use reqwest;
use serde_json;
use std::os::raw::c_char;

unsafe extern "C" fn name() -> *const c_char {
    b"SummarizerPlugin\0".as_ptr() as *const c_char
}

unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    if input.is_null() {
        return PluginOutput { text: std::ptr::null_mut() };
    }
    let c_str = CStr::from_ptr((*input).text);
    let text = c_str.to_string_lossy();
    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "mistral",
            "prompt": format!("Summarize this:\n\n{}", text),
            "stream": false
        }))
        .send();
    let summary = match res {
        Ok(resp) => {
            let json: serde_json::Value = resp.json().unwrap_or_default();
            json["response"].as_str().unwrap_or("").to_string()
        },
        Err(e) => format!("Summarizer error: {}", e),
    };
    let out = CString::new(summary).unwrap().into_raw();
    PluginOutput { text: out }
}

unsafe extern "C" fn free_output(output: PluginOutput) {
    if !output.text.is_null() {
        let _ = CString::from_raw(output.text);
    }
}

unsafe extern "C" fn run_with_buffer(_input: *const lao_plugin_api::PluginInput, _buffer: *mut std::os::raw::c_char, _buffer_len: usize) -> usize {
    0 // Not implemented for SummarizerPlugin
}

#[no_mangle]
pub static PLUGIN_VTABLE: lao_plugin_api::PluginVTable = lao_plugin_api::PluginVTable {
    version: 1,
    name,
    run,
    free_output,
    run_with_buffer,
};

#[no_mangle]
pub extern "C" fn plugin_vtable() -> PluginVTablePtr {
    &PLUGIN_VTABLE
} 