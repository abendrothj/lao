use lao_plugin_api::{PluginInput, PluginMetadata, PluginOutput, PluginVTable, PluginVTablePtr};
use regex::Regex;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

const CAPABILITIES_JSON: &str = "[{\"name\":\"extract-regex\",\"description\":\"Extract regex matches from text\",\"input_type\":\"Any\",\"output_type\":\"Text\"}]\0";
const MAX_PATTERN_BYTES: usize = 4 * 1024;
const MAX_TEXT_BYTES: usize = 1024 * 1024;
const MAX_MATCHES: usize = 10_000;

unsafe extern "C" fn name() -> *const c_char {
    c"RegexExtractPlugin".as_ptr()
}

unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    lao_plugin_api::catch_panic_output(|| {
        if input.is_null() || (*input).text.is_null() {
            return PluginOutput {
                text: out("error: RegexExtractPlugin received null input"),
            };
        }

        let raw = CStr::from_ptr((*input).text).to_string_lossy().to_string();
        PluginOutput {
            text: match extract(&raw) {
                Ok(s) => lao_plugin_api::owned_cstring(&s),
                Err(e) => out(&format!("error: {}", e)),
            },
        }
    })
}

fn extract(raw: &str) -> Result<String, String> {
    let mut lines = raw.splitn(2, '\n');
    let pattern = lines.next().unwrap_or("").trim();
    if pattern.is_empty() {
        return Err("missing regex pattern on the first line".to_string());
    }
    if pattern.len() > MAX_PATTERN_BYTES {
        return Err(format!(
            "regex pattern exceeds {} byte limit",
            MAX_PATTERN_BYTES
        ));
    }
    let text = lines.next().unwrap_or("");
    if text.len() > MAX_TEXT_BYTES {
        return Err(format!("regex text exceeds {} byte limit", MAX_TEXT_BYTES));
    }

    let re = Regex::new(pattern).map_err(|e| format!("invalid regex: {}", e))?;

    let has_group = re.captures_len() > 1;
    let mut matches = Vec::new();
    for caps in re.captures_iter(text) {
        let value = if has_group {
            caps.get(1).map(|m| m.as_str())
        } else {
            caps.get(0).map(|m| m.as_str())
        };
        if let Some(v) = value {
            matches.push(v.to_string());
            if matches.len() > MAX_MATCHES {
                return Err(format!("regex match count exceeds {} limit", MAX_MATCHES));
            }
        }
    }

    if matches.is_empty() {
        return Err(format!("no matches found for pattern '{}'", pattern));
    }
    Ok(matches.join("\n"))
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
    static NAME: &[u8] = b"RegexExtractPlugin\0";
    static VERSION: &[u8] = b"1.0.0\0";
    static DESCRIPTION: &[u8] = b"Extracts regex matches from text\0";
    static AUTHOR: &[u8] = b"LAO Team\0";
    static TAGS: &[u8] = b"[\"regex\", \"extract\", \"text\", \"utility\"]\0";

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
    fn extracts_whole_matches() {
        let result = extract("\\d+\nthere are 3 cats and 12 dogs").unwrap();
        assert_eq!(result, "3\n12");
    }

    #[test]
    fn extracts_capture_group() {
        let result = extract("(\\w+)@\\w+\\.com\nreach me at ada@mail.com").unwrap();
        assert_eq!(result, "ada");
    }

    #[test]
    fn errors_on_no_matches() {
        let err = extract("\\d+\nno numbers here");
        assert!(err.is_err());
    }

    #[test]
    fn errors_on_invalid_regex() {
        let err = extract("(unclosed\nsome text");
        assert!(err.is_err());
    }

    #[test]
    fn errors_when_too_many_matches() {
        let text = "a".repeat(MAX_MATCHES + 1);
        let err = extract(&format!("a\n{}", text)).unwrap_err();
        assert!(err.contains("match count"));
    }
}
