use lao_plugin_api::{PluginInput, PluginMetadata, PluginOutput, PluginVTable, PluginVTablePtr};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

const CAPABILITIES_JSON: &str = "[{\"name\":\"map-folder\",\"description\":\"Recursively list files under a directory\",\"input_type\":\"Any\",\"output_type\":\"Text\"}]\0";

const MAX_ENTRIES: usize = 10_000;

unsafe extern "C" fn name() -> *const c_char {
    c"FolderMapPlugin".as_ptr()
}

unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    lao_plugin_api::catch_panic_output(|| {
        if input.is_null() || (*input).text.is_null() {
            return PluginOutput {
                text: out("error: FolderMapPlugin received null input"),
            };
        }

        let root = CStr::from_ptr((*input).text).to_string_lossy();
        let root = root.trim();
        if root.is_empty() {
            return PluginOutput {
                text: out("error: FolderMapPlugin received an empty path"),
            };
        }

        let root_path = Path::new(root);
        if !root_path.is_dir() {
            return PluginOutput {
                text: out(&format!("error: '{}' is not a directory", root)),
            };
        }

        match collect_files(root_path) {
            Ok(mut files) => {
                files.sort();
                let listing = files.join("\n");
                PluginOutput {
                    text: lao_plugin_api::owned_cstring(&listing),
                }
            }
            Err(e) => PluginOutput {
                text: out(&format!("error: failed to map '{}': {}", root, e)),
            },
        }
    })
}

fn collect_files(root: &Path) -> std::io::Result<Vec<String>> {
    let mut files = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path.strip_prefix(root).unwrap_or(&path);
                files.push(rel.to_string_lossy().replace('\\', "/"));
                if files.len() >= MAX_ENTRIES {
                    return Ok(files);
                }
            }
        }
    }
    Ok(files)
}

fn out(msg: &str) -> *mut c_char {
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
    static NAME: &[u8] = b"FolderMapPlugin\0";
    static VERSION: &[u8] = b"1.0.0\0";
    static DESCRIPTION: &[u8] = b"Recursively lists files under a directory\0";
    static AUTHOR: &[u8] = b"LAO Team\0";
    static TAGS: &[u8] = b"[\"file\", \"directory\", \"walk\", \"utility\"]\0";

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
    fn lists_files_recursively() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("lao_foldermap_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.txt"), "a").unwrap();
        std::fs::write(dir.join("sub/b.txt"), "b").unwrap();

        let out = run_with(dir.to_str().unwrap());
        assert!(out.contains("a.txt"));
        assert!(out.contains("sub/b.txt"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn errors_on_non_directory() {
        let out = run_with("/definitely/not/a/dir_xyz");
        assert!(out.starts_with("error:"));
    }
}
