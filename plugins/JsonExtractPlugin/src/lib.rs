use lao_plugin_api::{PluginInput, PluginMetadata, PluginOutput, PluginVTable, PluginVTablePtr};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

const CAPABILITIES_JSON: &str = "[{\"name\":\"extract-json\",\"description\":\"Extract a value from a JSON document using a dotted path selector\",\"input_type\":\"Any\",\"output_type\":\"Text\"}]\0";

unsafe extern "C" fn name() -> *const c_char {
    c"JsonExtractPlugin".as_ptr()
}

unsafe extern "C" fn run(input: *const PluginInput) -> PluginOutput {
    lao_plugin_api::catch_panic_output(|| {
        if input.is_null() || (*input).text.is_null() {
            return PluginOutput {
                text: out("error: JsonExtractPlugin received null input"),
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
    let (selector, json_src) = split_selector(raw);

    let doc: Value =
        serde_json::from_str(json_src.trim()).map_err(|e| format!("invalid JSON: {}", e))?;

    let selected = match selector {
        Some(sel) => navigate(&doc, &sel)?,
        None => &doc,
    };

    Ok(match selected {
        Value::String(s) => s.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_else(|_| other.to_string()),
    })
}

fn split_selector(raw: &str) -> (Option<String>, String) {
    let mut lines = raw.splitn(2, '\n');
    let first = lines.next().unwrap_or("").trim();
    if first.starts_with('$') {
        let rest = lines.next().unwrap_or("").to_string();
        (Some(first.to_string()), rest)
    } else {
        (None, raw.to_string())
    }
}

fn navigate<'a>(doc: &'a Value, selector: &str) -> Result<&'a Value, String> {
    let mut current = doc;
    for token in tokenize(selector)? {
        match token {
            Token::Key(k) => {
                current = current
                    .get(&k)
                    .ok_or_else(|| format!("key '{}' not found", k))?;
            }
            Token::Index(i) => {
                current = current
                    .get(i)
                    .ok_or_else(|| format!("index [{}] out of range", i))?;
            }
        }
    }
    Ok(current)
}

enum Token {
    Key(String),
    Index(usize),
}

fn tokenize(selector: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    if !selector.starts_with('$') {
        return Err("selector must start with '$'".to_string());
    }

    let body = selector.trim_start_matches('$').trim_start_matches('.');
    for segment in body.split('.') {
        if segment.is_empty() {
            continue;
        }

        if let Some(bracket) = segment.find('[') {
            let key = &segment[..bracket];
            if !key.is_empty() {
                tokens.push(Token::Key(key.to_string()));
            }
            let mut rest = &segment[bracket..];
            while !rest.is_empty() {
                if !rest.starts_with('[') {
                    return Err(format!("invalid selector segment '{}'", segment));
                }
                let close = rest
                    .find(']')
                    .ok_or_else(|| format!("missing ']' in selector segment '{}'", segment))?;
                let idx = &rest[1..close];
                let n = idx
                    .parse::<usize>()
                    .map_err(|_| format!("invalid array index '{}'", idx))?;
                tokens.push(Token::Index(n));
                rest = &rest[close + 1..];
            }
        } else {
            tokens.push(Token::Key(segment.to_string()));
        }
    }
    Ok(tokens)
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
    static NAME: &[u8] = b"JsonExtractPlugin\0";
    static VERSION: &[u8] = b"1.0.0\0";
    static DESCRIPTION: &[u8] = b"Extracts a value from a JSON document using a dotted path\0";
    static AUTHOR: &[u8] = b"LAO Team\0";
    static TAGS: &[u8] = b"[\"json\", \"extract\", \"parse\", \"utility\"]\0";

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
    fn extracts_nested_key() {
        let out = extract("$.user.name\n{\"user\": {\"name\": \"Ada\"}}").unwrap();
        assert_eq!(out, "Ada");
    }

    #[test]
    fn extracts_array_index() {
        let out = extract("$.items[1]\n{\"items\": [10, 20, 30]}").unwrap();
        assert_eq!(out, "20");
    }

    #[test]
    fn pretty_prints_whole_document_without_selector() {
        let out = extract("{\"a\":1}").unwrap();
        assert!(out.contains("\"a\""));
        assert!(out.contains('\n'));
    }

    #[test]
    fn errors_on_missing_key() {
        let err = extract("$.missing\n{\"a\": 1}");
        assert!(err.is_err());
    }

    #[test]
    fn errors_on_invalid_json() {
        let err = extract("$.a\n{not json}");
        assert!(err.is_err());
    }

    #[test]
    fn errors_on_malformed_selector() {
        let err = extract("$.items[nope]\n{\"items\": [1]}");
        assert!(err.is_err());
    }
}
