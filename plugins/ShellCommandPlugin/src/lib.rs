use lao_plugin_api::{PluginInput, PluginMetadata, PluginOutput, PluginVTable, PluginVTablePtr};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::process::Command;

const CAPABILITIES_JSON: &str = "[{\"name\":\"run-shell\",\"description\":\"Run a trusted shell command and capture stdout\",\"input_type\":\"Any\",\"output_type\":\"Text\"}]\0";

unsafe extern "C" fn name() -> *const c_char {
    c"ShellCommandPlugin".as_ptr()
}

unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    lao_plugin_api::catch_panic_output(|| {
        if input.is_null() || (*input).text.is_null() {
            return PluginOutput {
                text: out("error: ShellCommandPlugin received null input"),
            };
        }

        if std::env::var("LAO_ALLOW_SHELL").unwrap_or_default() != "1" {
            return PluginOutput {
                text: out("error: shell execution disabled; set LAO_ALLOW_SHELL=1 to enable"),
            };
        }

        let command = CStr::from_ptr((*input).text).to_string_lossy();
        let command = command.trim();
        if command.is_empty() {
            return PluginOutput {
                text: out("error: ShellCommandPlugin received an empty command"),
            };
        }

        PluginOutput {
            text: match run_command(command) {
                Ok(stdout) => lao_plugin_api::owned_cstring(&stdout),
                Err(e) => out(&format!("error: {}", e)),
            },
        }
    })
}

fn run_command(command: &str) -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", command]).output()
    } else {
        Command::new("sh").args(["-c", command]).output()
    }
    .map_err(|e| format!("failed to start command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!(
            "command exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
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
    static NAME: &[u8] = b"ShellCommandPlugin\0";
    static VERSION: &[u8] = b"1.0.0\0";
    static DESCRIPTION: &[u8] = b"Runs a trusted shell command and returns its stdout\0";
    static AUTHOR: &[u8] = b"LAO Team\0";
    static TAGS: &[u8] = b"[\"shell\", \"command\", \"exec\", \"utility\"]\0";

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

    #[test]
    fn runs_simple_command() {
        let out = run_command("echo hello").unwrap();
        assert_eq!(out.trim(), "hello");
    }

    #[test]
    fn reports_command_failure() {
        let err = run_command("exit 3");
        assert!(err.is_err());
    }
}
