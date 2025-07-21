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
    pub name: unsafe extern "C" fn() -> *const c_char,
    pub run: unsafe extern "C" fn(input: *const PluginInput) -> PluginOutput,
    pub free_output: unsafe extern "C" fn(output: PluginOutput),
    // Add more functions as needed
}

pub type PluginVTablePtr = *const PluginVTable; 