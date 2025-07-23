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
    let c_str = std::ffi::CStr::from_ptr((*input).text);
    let input_str = c_str.to_string_lossy();
    let output = if input_str.contains("nonsense") {
        "error: could not generate workflow"
    } else {
        // For demo, just echo the input as a fake YAML
        "workflow: demo\nsteps:\n  - run: Echo\n    input: demo"
    };
    let c_string = std::ffi::CString::new(output).unwrap();
    PluginOutput { text: c_string.into_raw() }
}

unsafe extern "C" fn free_output(output: PluginOutput) {
    if !output.text.is_null() {
        let _ = CString::from_raw(output.text);
    }
}

unsafe extern "C" fn run_with_buffer(input: *const PluginInput, buffer: *mut c_char, buffer_len: usize) -> usize {
    if input.is_null() || buffer.is_null() || buffer_len == 0 {
        return 0;
    }
    let c_str = std::ffi::CStr::from_ptr((*input).text);
    let input_str = c_str.to_string_lossy();
    let output: &[u8] = if input_str.contains("nonsense") {
        b"error: could not generate workflow".as_ref()
    } else {
        b"workflow: demo\nsteps:\n  - run: Echo\n    input: demo".as_ref()
    };
    let max_copy = std::cmp::min(output.len(), buffer_len - 1);
    std::ptr::copy_nonoverlapping(output.as_ptr(), buffer as *mut u8, max_copy);
    *buffer.add(max_copy) = 0; // null terminator
    max_copy
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