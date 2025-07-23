use lao_plugin_api::{PluginInput, PluginOutput, PluginVTable, PluginVTablePtr};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

unsafe extern "C" fn name() -> *const c_char {
    CString::new("Echo").unwrap().into_raw()
}

unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    if input.is_null() {
        println!("[EchoPlugin] Received null input");
        return PluginOutput { text: std::ptr::null_mut() };
    }
    let c_str = CStr::from_ptr((*input).text);
    let s = c_str.to_string_lossy();
    println!("[EchoPlugin] Received input: {}", s);
    let out = CString::new(s.as_ref()).unwrap();
    println!("[EchoPlugin] Returning output: {}", out.to_string_lossy());
    PluginOutput { text: out.into_raw() }
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
    let bytes = c_str.to_bytes();
    if bytes.is_empty() {
        return 0;
    }
    let max_copy = std::cmp::min(bytes.len(), buffer_len - 1);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, max_copy);
    *buffer.add(max_copy) = 0; // null terminator
    max_copy
}

#[no_mangle]
pub static PLUGIN_VTABLE: PluginVTable = PluginVTable {
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