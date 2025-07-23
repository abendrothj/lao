use lao_plugin_api::{PluginInput, PluginOutput, PluginVTable, PluginVTablePtr};
use std::ffi::{CStr, CString};
use std::process::Command;
use std::os::raw::c_char;

unsafe extern "C" fn name() -> *const c_char {
    b"WhisperPlugin\0".as_ptr() as *const c_char
}

unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    if input.is_null() {
        return PluginOutput { text: std::ptr::null_mut() };
    }
    let c_str = CStr::from_ptr((*input).text);
    let audio_path = c_str.to_string_lossy();
    let output = Command::new("./whisper.cpp")
        .arg(&*audio_path)
        .output();
    let text = match output {
        Ok(out) if out.status.success() => {
            CString::new(String::from_utf8_lossy(&out.stdout).to_string()).unwrap().into_raw()
        },
        Ok(out) => {
            CString::new(format!("whisper.cpp failed: {}", String::from_utf8_lossy(&out.stderr))).unwrap().into_raw()
        },
        Err(e) => {
            CString::new(format!("Failed to run whisper.cpp: {}", e)).unwrap().into_raw()
        }
    };
    PluginOutput { text }
}

unsafe extern "C" fn free_output(output: PluginOutput) {
    if !output.text.is_null() {
        let _ = CString::from_raw(output.text);
    }
}

unsafe extern "C" fn run_with_buffer(_input: *const lao_plugin_api::PluginInput, _buffer: *mut std::os::raw::c_char, _buffer_len: usize) -> usize {
    0 // Not implemented for WhisperPlugin
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