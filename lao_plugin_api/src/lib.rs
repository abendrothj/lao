use std::ffi::{c_char, CStr, CString};

/// Current ABI version produced by this crate. Plugins built against this crate
/// advertise this version in their vtable.
pub const LAO_PLUGIN_ABI_VERSION: u32 = 2;

/// Oldest ABI version the host still accepts. v1 plugins remain loadable; the host
/// falls back to the text `run`/`free_output` path for them.
pub const LAO_PLUGIN_ABI_MIN_SUPPORTED: u32 = 1;

/// Structured result status codes (ABI v2). Kept as plain `u32` for C compatibility.
pub const LAO_STATUS_SUCCESS: u32 = 0;
pub const LAO_STATUS_VALIDATION_FAILED: u32 = 1;
pub const LAO_STATUS_RUNTIME_ERROR: u32 = 2;

#[repr(C)]
pub struct PluginInput {
    pub text: *mut c_char,
}

/// Structured plugin result (ABI v2). `text` carries the output on success or the
/// error message otherwise. Ownership: the plugin allocates `text`; the host returns
/// it via `free_result`.
#[repr(C)]
pub struct PluginResult {
    pub status: u32,
    pub text: *mut c_char,
}

impl PluginResult {
    /// Convert a v1-style text output into a structured result, taking ownership of
    /// `out.text`. Status is derived from the legacy `error:`/empty convention so
    /// existing plugin logic keeps working under the structured channel.
    ///
    /// # Safety
    /// `out.text` must be either null or a valid heap `CString` pointer that the
    /// resulting `PluginResult` now owns (freed via `free_result`).
    pub unsafe fn from_text_output(out: PluginOutput) -> PluginResult {
        if out.text.is_null() {
            return PluginResult {
                status: LAO_STATUS_RUNTIME_ERROR,
                text: std::ptr::null_mut(),
            };
        }
        let view = CStr::from_ptr(out.text).to_string_lossy();
        let trimmed = view.trim();
        let is_error = trimmed.is_empty() || trimmed.to_lowercase().starts_with("error:");
        let status = if is_error {
            LAO_STATUS_RUNTIME_ERROR
        } else {
            LAO_STATUS_SUCCESS
        };
        // Move ownership of the existing allocation into the result.
        PluginResult {
            status,
            text: out.text,
        }
    }
}

/// Message carried by the fallback outputs the `catch_panic_*` helpers produce
/// when a plugin body panics.
pub const LAO_PANIC_ERROR: &str = "error: plugin panicked";

/// Allocate an owned C string for `msg`, stripping interior NUL bytes so the
/// conversion cannot fail. The returned pointer follows the usual plugin output
/// ownership: the host releases it via `free_output`/`free_result`.
pub fn owned_cstring(msg: &str) -> *mut c_char {
    let sanitized: Vec<u8> = msg.bytes().filter(|&b| b != 0).collect();
    CString::new(sanitized)
        .expect("NUL bytes stripped")
        .into_raw()
}

/// Run a plugin `run` body, containing panics at the FFI boundary. A panic
/// unwinding out of an `extern "C"` fn aborts the host process, so every
/// plugin entry point should route its body through one of these helpers.
/// Panics are mapped to an owned [`LAO_PANIC_ERROR`] output.
pub fn catch_panic_output<F: FnOnce() -> PluginOutput>(body: F) -> PluginOutput {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)).unwrap_or_else(|_| PluginOutput {
        text: owned_cstring(LAO_PANIC_ERROR),
    })
}

/// Run a plugin `run_structured` body, mapping panics to
/// [`LAO_STATUS_RUNTIME_ERROR`] with an owned [`LAO_PANIC_ERROR`] message.
pub fn catch_panic_result<F: FnOnce() -> PluginResult>(body: F) -> PluginResult {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)).unwrap_or_else(|_| PluginResult {
        status: LAO_STATUS_RUNTIME_ERROR,
        text: owned_cstring(LAO_PANIC_ERROR),
    })
}

/// Run a plugin `run_with_buffer` body, mapping panics to `0` (no bytes written).
pub fn catch_panic_buffer_len<F: FnOnce() -> usize>(body: F) -> usize {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)).unwrap_or(0)
}

/// Run a plugin `validate_input` body, mapping panics to `false`.
pub fn catch_panic_bool<F: FnOnce() -> bool>(body: F) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)).unwrap_or(false)
}

/// Generate ABI v2 structured adapters (`__lao_run_structured` / `__lao_free_result`)
/// that bridge a plugin's existing v1 `run` function to the structured channel.
///
/// Usage inside a plugin crate:
/// ```ignore
/// lao_plugin_api::lao_structured_adapter!(run);
/// ```
#[macro_export]
macro_rules! lao_structured_adapter {
    ($run:ident) => {
        unsafe extern "C" fn __lao_run_structured(
            input: *const $crate::PluginInput,
        ) -> $crate::PluginResult {
            $crate::catch_panic_result(|| {
                let out = $run(input);
                $crate::PluginResult::from_text_output(out)
            })
        }

        unsafe extern "C" fn __lao_free_result(result: $crate::PluginResult) {
            if !result.text.is_null() {
                let _ = std::ffi::CString::from_raw(result.text);
            }
        }
    };
}

#[repr(C)]
pub struct MultiModalInput {
    pub input_type: u32, // Maps to PluginInputType as discriminant
    pub text_data: *mut c_char,
    pub file_path: *mut c_char,
    pub binary_data: *mut u8,
    pub binary_size: usize,
    pub metadata: *mut c_char, // JSON metadata for additional context
}

#[repr(C)]
pub struct PluginOutput {
    pub text: *mut c_char,
}

#[repr(C)]
pub struct PluginMetadata {
    pub name: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
    pub author: *const c_char,
    pub dependencies: *const c_char, // JSON array of dependency strings
    pub tags: *const c_char,         // JSON array of tag strings
    pub input_schema: *const c_char, // JSON schema for input validation
    pub output_schema: *const c_char, // JSON schema for output validation
    pub capabilities: *const c_char, // JSON array of capabilities
}

/// Plugin vtable. Fields through `get_capabilities` are the stable ABI v1 prefix.
/// `run_structured`/`free_result` are appended in ABI v2; the host only reads them
/// when `version >= 2`, so v1 plugins (with a shorter static) remain sound.
#[repr(C)]
pub struct PluginVTable {
    pub version: u32,
    pub name: unsafe extern "C" fn() -> *const c_char,
    pub run: unsafe extern "C" fn(*const PluginInput) -> PluginOutput,
    pub free_output: unsafe extern "C" fn(PluginOutput),
    pub run_with_buffer: unsafe extern "C" fn(*const PluginInput, *mut c_char, usize) -> usize,
    pub get_metadata: unsafe extern "C" fn() -> PluginMetadata,
    pub validate_input: unsafe extern "C" fn(*const PluginInput) -> bool,
    pub get_capabilities: unsafe extern "C" fn() -> *const c_char, // JSON array of capabilities
    // --- ABI v2 (appended) ---
    pub run_structured: unsafe extern "C" fn(*const PluginInput) -> PluginResult,
    pub free_result: unsafe extern "C" fn(PluginResult),
}

pub type PluginVTablePtr = *const PluginVTable;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PluginInputType {
    Text,
    Json,
    Binary,
    File,
    Audio,
    Image,
    Video,
    Any,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PluginOutputType {
    Text,
    Json,
    Binary,
    File,
    Audio,
    Image,
    Video,
    Any,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginCapability {
    pub name: String,
    pub description: String,
    pub input_type: PluginInputType,
    pub output_type: PluginOutputType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub dependencies: Vec<PluginDependency>,
    pub tags: Vec<String>,
    pub capabilities: Vec<PluginCapability>,
    pub input_schema: Option<String>,
    pub output_schema: Option<String>,
}

impl PluginInfo {
    pub fn from_metadata(metadata: &PluginMetadata) -> Self {
        // Helper function to safely convert C string pointer to String
        fn safe_cstr_to_string(ptr: *const c_char) -> String {
            if ptr.is_null() {
                return String::new();
            }
            // Try to create CStr, if it fails, return empty string
            unsafe {
                match CStr::from_ptr(ptr).to_str() {
                    Ok(s) => s.to_string(),
                    Err(_) => String::new(),
                }
            }
        }

        let name = safe_cstr_to_string(metadata.name);
        let version = safe_cstr_to_string(metadata.version);
        let description = safe_cstr_to_string(metadata.description);
        let author = safe_cstr_to_string(metadata.author);

        let dependencies = if !metadata.dependencies.is_null() {
            let deps_str = safe_cstr_to_string(metadata.dependencies);
            serde_json::from_str::<Vec<PluginDependency>>(&deps_str).unwrap_or_default()
        } else {
            Vec::new()
        };

        let tags = if !metadata.tags.is_null() {
            let tags_str = safe_cstr_to_string(metadata.tags);
            serde_json::from_str::<Vec<String>>(&tags_str).unwrap_or_default()
        } else {
            Vec::new()
        };

        let capabilities = if !metadata.capabilities.is_null() {
            let caps_str = safe_cstr_to_string(metadata.capabilities);
            serde_json::from_str::<Vec<PluginCapability>>(&caps_str).unwrap_or_default()
        } else {
            Vec::new()
        };

        let input_schema = if !metadata.input_schema.is_null() {
            let schema_str = safe_cstr_to_string(metadata.input_schema);
            if schema_str.is_empty() {
                None
            } else {
                Some(schema_str)
            }
        } else {
            None
        };

        let output_schema = if !metadata.output_schema.is_null() {
            let schema_str = safe_cstr_to_string(metadata.output_schema);
            if schema_str.is_empty() {
                None
            } else {
                Some(schema_str)
            }
        } else {
            None
        };

        PluginInfo {
            name,
            version,
            description,
            author,
            dependencies,
            tags,
            capabilities,
            input_schema,
            output_schema,
        }
    }
}
