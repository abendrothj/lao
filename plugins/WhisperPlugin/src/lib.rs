use lao_plugin_api::{PluginInput, PluginMetadata, PluginOutput, PluginVTablePtr};
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::process::Command;

unsafe extern "C" fn name() -> *const c_char {
    c"WhisperPlugin".as_ptr()
}

// Find whisper.cpp binary in common locations
fn find_whisper_binary() -> Option<String> {
    // Check environment variable first (highest priority)
    if let Ok(path) = env::var("WHISPER_CPP_PATH") {
        if Path::new(&path).exists() {
            return Some(path);
        }
    }

    // Check PATH using `which` command (Unix/macOS)
    #[cfg(unix)]
    {
        for cmd in &["whisper.cpp", "whisper-cpp"] {
            if let Ok(output) = Command::new("which").arg(cmd).output() {
                if output.status.success() {
                    if let Ok(path_str) = String::from_utf8(output.stdout) {
                        let path = path_str.trim().to_string();
                        if !path.is_empty() && Path::new(&path).exists() {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }

    // Check common file system locations
    let candidates = vec![
        "./whisper.cpp",
        "./whisper-cpp",
        "/usr/local/bin/whisper.cpp",
        "/usr/local/bin/whisper-cpp",
        "/usr/bin/whisper.cpp",
        "/usr/bin/whisper-cpp",
        "~/.local/bin/whisper.cpp",
        "~/.local/bin/whisper-cpp",
    ];

    for candidate in candidates {
        // Expand ~ to home directory
        let expanded = if candidate.starts_with("~/") {
            if let Ok(home) = env::var("HOME") {
                candidate.replacen("~", &home, 1)
            } else {
                continue; // Skip if we can't expand ~
            }
        } else {
            candidate.to_string()
        };

        // Check if it exists
        if Path::new(&expanded).exists() {
            return Some(expanded);
        }
    }

    None
}

unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    lao_plugin_api::catch_panic_output(|| run_inner(input))
}

unsafe fn run_inner(input: *const PluginInput) -> PluginOutput {
    if input.is_null() || (*input).text.is_null() {
        return PluginOutput {
            text: std::ptr::null_mut(),
        };
    }
    let c_str = CStr::from_ptr((*input).text);
    let audio_path = c_str.to_string_lossy();

    // Find whisper binary
    let whisper_bin = match find_whisper_binary() {
        Some(path) => path,
        None => {
            let error_msg = "whisper.cpp binary not found. Please install whisper.cpp or set WHISPER_CPP_PATH environment variable.\n\
                Common locations checked: ./whisper.cpp, whisper.cpp (in PATH), /usr/local/bin/whisper.cpp\n\
                Install from: https://github.com/ggerganov/whisper.cpp"
                .to_string();
            return PluginOutput {
                text: lao_plugin_api::owned_cstring(&error_msg),
            };
        }
    };

    let output = Command::new(&whisper_bin).arg(&*audio_path).output();
    let text = match output {
        Ok(out) if out.status.success() => {
            lao_plugin_api::owned_cstring(&String::from_utf8_lossy(&out.stdout))
        }
        Ok(out) => lao_plugin_api::owned_cstring(&format!(
            "whisper.cpp failed: {}\nCommand: {} {}\nStderr: {}",
            out.status,
            whisper_bin,
            audio_path,
            String::from_utf8_lossy(&out.stderr)
        )),
        Err(e) => lao_plugin_api::owned_cstring(&format!(
            "Failed to run whisper.cpp: {}\nBinary path: {}\nAudio file: {}\n\n\
            Troubleshooting:\n\
            1. Ensure whisper.cpp is installed and in your PATH\n\
            2. Or set WHISPER_CPP_PATH environment variable to the full path\n\
            3. Verify the audio file exists: {}",
            e, whisper_bin, audio_path, audio_path
        )),
    };
    PluginOutput { text }
}

unsafe extern "C" fn free_output(output: PluginOutput) {
    if !output.text.is_null() {
        let _ = CString::from_raw(output.text);
    }
}

unsafe extern "C" fn run_with_buffer(
    _input: *const lao_plugin_api::PluginInput,
    _buffer: *mut std::os::raw::c_char,
    _buffer_len: usize,
) -> usize {
    0 // Not implemented for WhisperPlugin
}

unsafe extern "C" fn get_metadata() -> PluginMetadata {
    // Use static byte arrays to ensure proper memory management
    static NAME: &[u8] = b"WhisperPlugin\0";
    static VERSION: &[u8] = b"1.0.0\0";
    static DESCRIPTION: &[u8] = b"Whisper speech-to-text plugin for LAO\0";
    static AUTHOR: &[u8] = b"LAO Team\0";
    static TAGS: &[u8] = b"[\"speech\", \"whisper\", \"audio\", \"transcription\"]\0";
    static CAPABILITIES: &[u8] = b"[{\"name\":\"speech-to-text\",\"description\":\"Convert speech to text using Whisper\",\"input_type\":\"Text\",\"output_type\":\"Text\"}]\0";

    PluginMetadata {
        name: NAME.as_ptr() as *const c_char,
        version: VERSION.as_ptr() as *const c_char,
        description: DESCRIPTION.as_ptr() as *const c_char,
        author: AUTHOR.as_ptr() as *const c_char,
        dependencies: std::ptr::null(),
        tags: TAGS.as_ptr() as *const c_char,
        input_schema: std::ptr::null(),
        output_schema: std::ptr::null(),
        capabilities: CAPABILITIES.as_ptr() as *const c_char,
    }
}

unsafe extern "C" fn validate_input(input: *const PluginInput) -> bool {
    lao_plugin_api::catch_panic_bool(|| {
        if input.is_null() || (*input).text.is_null() {
            return false;
        }
        let c_str = CStr::from_ptr((*input).text);
        let text = c_str.to_string_lossy();
        !text.trim().is_empty()
    })
}

unsafe extern "C" fn get_capabilities() -> *const c_char {
    static CAPABILITIES: &[u8] = b"[{\"name\":\"speech-to-text\",\"description\":\"Convert speech to text using Whisper\",\"input_type\":\"Text\",\"output_type\":\"Text\"}]\0";
    CAPABILITIES.as_ptr() as *const c_char
}

lao_plugin_api::lao_structured_adapter!(run);

#[no_mangle]
pub static PLUGIN_VTABLE: lao_plugin_api::PluginVTable = lao_plugin_api::PluginVTable {
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
    use std::ffi::CString;

    #[test]
    fn test_plugin_name() {
        unsafe {
            let name_ptr = name();
            let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
            let name_str = name_cstr.to_str().unwrap();
            assert_eq!(name_str, "WhisperPlugin");
        }
    }

    #[test]
    fn test_validate_input() {
        unsafe {
            let valid_input = CString::new("path/to/audio.wav").unwrap();
            let input = PluginInput {
                text: valid_input.into_raw(),
            };
            assert!(validate_input(&input));

            let empty_input = CString::new("   ").unwrap();
            let input = PluginInput {
                text: empty_input.into_raw(),
            };
            assert!(!validate_input(&input));
        }
    }
}
