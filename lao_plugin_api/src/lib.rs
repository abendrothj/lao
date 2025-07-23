use std::os::raw::{c_char};

#[repr(C)]
pub struct PluginInput {
    pub text: *const c_char,
    // Extend as needed
}

#[repr(C)]
pub struct PluginOutput {
    pub text: *mut c_char,
    // Extend as needed
}

#[repr(C)]
pub struct PluginVTable {
    pub version: u32, // Must match between host and plugin
    pub name: unsafe extern "C" fn() -> *const c_char,
    pub run: unsafe extern "C" fn(input: *const PluginInput) -> PluginOutput,
    pub free_output: unsafe extern "C" fn(output: PluginOutput),
    pub run_with_buffer: unsafe extern "C" fn(input: *const PluginInput, buffer: *mut c_char, buffer_len: usize) -> usize,
    // Add more functions as needed
}

pub type PluginVTablePtr = *const PluginVTable; 