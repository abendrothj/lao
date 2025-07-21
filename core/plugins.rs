use std::collections::HashMap;
use libloading::Library;
use std::fs;

#[derive(Debug)]
pub struct PluginConfig {
    pub parameters: HashMap<String, String>,
    pub verbose: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum PluginInputType {
    Text,
    AudioFile,
    Json,
    TaggedData,
    Any,
}

#[derive(Debug)]
pub enum LaoError {
    InitError(String),
    ExecutionError(String),
    ShutdownError(String),
}

pub struct LoadedPlugin {
    pub vtable: lao_plugin_api::PluginVTablePtr,
    pub lib: Library,
}

pub struct PluginRegistry {
    pub plugins: HashMap<String, LoadedPlugin>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: HashMap::new() }
    }
    pub fn load_dynamic_plugins(&mut self, plugins_dir: &str) {
        let entries = match fs::read_dir(plugins_dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    #[cfg(target_os = "windows")]
                    let is_dynlib = ext.eq_ignore_ascii_case("dll");
                    #[cfg(target_os = "linux")]
                    let is_dynlib = ext == "so";
                    #[cfg(target_os = "macos")]
                    let is_dynlib = ext == "dylib";
                    if is_dynlib {
                        unsafe {
                            match Library::new(&path) {
                                Ok(lib) => {
                                    let vtable_fn: libloading::Symbol<unsafe extern "C" fn() -> lao_plugin_api::PluginVTablePtr> = match lib.get(b"plugin_vtable") {
                                        Ok(f) => f,
                                        Err(_) => continue,
                                    };
                                    let vtable = vtable_fn();
                                    let name_ptr = ((*vtable).name)();
                                    let name = if !name_ptr.is_null() {
                                        std::ffi::CStr::from_ptr(name_ptr).to_string_lossy().to_string()
                                    } else {
                                        continue;
                                    };
                                    self.plugins.insert(name, LoadedPlugin { vtable, lib });
                                }
                                Err(_) => { /* Could not load library, skip */ }
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn dynamic_registry(plugins_dir: &str) -> Self {
        let mut reg = Self::new();
        reg.load_dynamic_plugins(plugins_dir);
        reg
    }
    pub fn get(&self, name: &str) -> Option<&LoadedPlugin> {
        self.plugins.get(name)
    }
} 