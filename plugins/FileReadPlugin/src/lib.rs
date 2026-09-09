use lao_plugin_api::{PluginInput, PluginMetadata, PluginOutput, PluginVTable, PluginVTablePtr};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

const CAPABILITIES_JSON: &str = "[{\"name\":\"read-file\",\"description\":\"Read a file from disk and return its contents as text\",\"input_type\":\"Any\",\"output_type\":\"Text\"}]\0";

unsafe extern "C" fn name() -> *const c_char {
    c"FileReadPlugin".as_ptr()
}

unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    lao_plugin_api::catch_panic_output(|| {
        if input.is_null() || (*input).text.is_null() {
            return PluginOutput {
                text: error_output("error: FileReadPlugin received null input"),
            };
        }

        let path = CStr::from_ptr((*input).text).to_string_lossy();
        let path = path.trim();
        if path.is_empty() {
            return PluginOutput {
                text: error_output("error: FileReadPlugin received an empty path"),
            };
        }

        let text = match std::fs::read_to_string(path) {
            Ok(contents) => lao_plugin_api::owned_cstring(&contents),
            Err(e) => error_output(&format!("error: failed to read '{}': {}", path, e)),
        };

        PluginOutput { text }
    })
}

fn error_output(msg: &str) -> *mut c_char {
    lao_plugin_api::owned_cstring(msg)
}

unsafe extern "C" fn free_output(output: PluginOutput) {
    if !output.text.is_null() {
        let _ = CString::from_raw(output.text);
    }
}

unsafe extern "C" fn run_with_buffer(
    _input: *const PluginInput,
    _buffer: *mut c_char,
    _buffer_len: usize,
) -> usize {
    0
}

unsafe extern "C" fn get_metadata() -> PluginMetadata {
    static NAME: &[u8] = b"FileReadPlugin\0";
    static VERSION: &[u8] = b"1.0.0\0";
    static DESCRIPTION: &[u8] = b"Reads a file from disk and returns its contents as text\0";
    static AUTHOR: &[u8] = b"LAO Team\0";
    static TAGS: &[u8] = b"[\"file\", \"io\", \"read\", \"utility\"]\0";

    PluginMetadata {
        name: NAME.as_ptr() as *const c_char,
        version: VERSION.as_ptr() as *const c_char,
        description: DESCRIPTION.as_ptr() as *const c_char,
        author: AUTHOR.as_ptr() as *const c_char,
        dependencies: std::ptr::null(),
        tags: TAGS.as_ptr() as *const c_char,
        input_schema: std::ptr::null(),
        output_schema: std::ptr::null(),
        capabilities: CAPABILITIES_JSON.as_ptr() as *const c_char,
    }
}

unsafe extern "C" fn validate_input(input: *const PluginInput) -> bool {
    lao_plugin_api::catch_panic_bool(|| {
        if input.is_null() || (*input).text.is_null() {
            return false;
        }
        !CStr::from_ptr((*input).text)
            .to_string_lossy()
            .trim()
            .is_empty()
    })
}

unsafe extern "C" fn get_capabilities() -> *const c_char {
    CAPABILITIES_JSON.as_ptr() as *const c_char
}

lao_plugin_api::lao_structured_adapter!(run);

#[no_mangle]
pub static PLUGIN_VTABLE: PluginVTable = PluginVTable {
    version: 2,
    name,
    run,
    free_output,
    run_with_buffer,
    get_metadata,
    validate_input,
    get_capabilities,
    run_structured: __lao_run_structured,
    free_result: __lao_free_result,
};

#[no_mangle]
pub extern "C" fn plugin_vtable() -> PluginVTablePtr {
    &PLUGIN_VTABLE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_with(text: &str) -> String {
        unsafe {
            let input_text = CString::new(text).unwrap();
            let input = PluginInput {
                text: input_text.into_raw(),
            };
            let output = run(&input);
            let result = CStr::from_ptr(output.text).to_string_lossy().to_string();
            free_output(output);
            let _ = CString::from_raw(input.text);
            result
        }
    }

    #[test]
    fn reads_existing_file() {
        let mut path = std::env::temp_dir();
        path.push(format!("lao_fileread_{}.txt", std::process::id()));
        std::fs::write(&path, "hello lao").unwrap();

        let out = run_with(path.to_str().unwrap());
        assert_eq!(out, "hello lao");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn errors_on_missing_file() {
        let out = run_with("/definitely/not/a/real/path_xyz.txt");
        assert!(out.starts_with("error:"));
    }

    #[test]
    fn errors_on_empty_path() {
        let out = run_with("   ");
        assert!(out.starts_with("error:"));
    }
}
