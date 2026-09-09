use lao_plugin_api::{PluginInput, PluginMetadata, PluginOutput, PluginVTable, PluginVTablePtr};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

const CAPABILITIES_JSON: &str = "[{\"name\":\"markdown-report\",\"description\":\"Format text into a Markdown report, optionally writing to disk\",\"input_type\":\"Any\",\"output_type\":\"Text\"}]\0";

unsafe extern "C" fn name() -> *const c_char {
    c"MarkdownReportPlugin".as_ptr()
}

unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    lao_plugin_api::catch_panic_output(|| {
        if input.is_null() || (*input).text.is_null() {
            return PluginOutput {
                text: out("error: MarkdownReportPlugin received null input"),
            };
        }

        let raw = CStr::from_ptr((*input).text).to_string_lossy().to_string();
        let report = build_report(&raw);
        let markdown = render(&report);

        if let Some(path) = &report.path {
            if let Err(e) = std::fs::write(path, &markdown) {
                return PluginOutput {
                    text: out(&format!("error: failed to write '{}': {}", path, e)),
                };
            }
        }

        PluginOutput {
            text: lao_plugin_api::owned_cstring(&markdown),
        }
    })
}

struct Report {
    title: String,
    body: String,
    path: Option<String>,
}

fn build_report(raw: &str) -> Report {
    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(raw.trim()) {
        let title = map
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Report")
            .to_string();
        let body = map
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let path = map
            .get("path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        return Report { title, body, path };
    }

    Report {
        title: "Report".to_string(),
        body: raw.to_string(),
        path: None,
    }
}

fn render(report: &Report) -> String {
    format!("# {}\n\n{}\n", report.title.trim(), report.body.trim())
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
    static NAME: &[u8] = b"MarkdownReportPlugin\0";
    static VERSION: &[u8] = b"1.0.0\0";
    static DESCRIPTION: &[u8] =
        b"Formats text into a Markdown report, optionally writing it to disk\0";
    static AUTHOR: &[u8] = b"LAO Team\0";
    static TAGS: &[u8] = b"[\"markdown\", \"report\", \"format\", \"utility\"]\0";

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
    fn wraps_plain_text() {
        let report = build_report("just some notes");
        let md = render(&report);
        assert_eq!(md, "# Report\n\njust some notes\n");
    }

    #[test]
    fn uses_structured_fields() {
        let report = build_report("{\"title\": \"Weekly\", \"body\": \"all good\"}");
        assert_eq!(report.title, "Weekly");
        let md = render(&report);
        assert!(md.starts_with("# Weekly"));
        assert!(md.contains("all good"));
    }

    #[test]
    fn treats_non_object_json_as_body() {
        let report = build_report("[1, 2, 3]");
        assert_eq!(report.title, "Report");
        assert!(report.path.is_none());
    }
}
