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
    pub run_with_buffer: Option<unsafe extern "C" fn(input: *const lao_plugin_api::PluginInput, buffer: *mut std::os::raw::c_char, buffer_len: usize) -> usize>,
}

pub struct PluginRegistry {
    pub plugins: HashMap<String, LoadedPlugin>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: HashMap::new() }
    }
    pub fn load_dynamic_plugins(&mut self, plugins_dir: &str) {
        // Always use absolute path for plugins_dir
        let plugins_dir = std::fs::canonicalize(plugins_dir).unwrap_or_else(|_| std::path::PathBuf::from(plugins_dir));
        // Diagnostic: check for EchoPlugin manifest existence
        let echo_manifest = plugins_dir.join("EchoPlugin").join("plugin.yaml");
        println!("[DIAG] EchoPlugin manifest absolute path: {} exists: {}", echo_manifest.display(), echo_manifest.exists());
        let entries = match fs::read_dir(&plugins_dir) {
            Ok(e) => e,
            Err(e) => {
                println!("[DIAG] Failed to read plugins dir {}: {}", plugins_dir.display(), e);
                return;
            }
        };
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                println!("[DIAG] Found file: {}", path.display());
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                #[cfg(target_os = "windows")]
                let is_dynlib = ext.eq_ignore_ascii_case("dll");
                #[cfg(target_os = "linux")]
                let is_dynlib = ext == "so";
                #[cfg(target_os = "macos")]
                let is_dynlib = ext == "dylib";
                if !is_dynlib {
                    println!("[DIAG] Skipping non-dynamic library: {}", path.display());
                    continue;
                }
                // Find the plugin subdirectory based on DLL stem
                let dll_stem = path.file_stem().unwrap().to_string_lossy();
                let base = if let Some(idx) = dll_stem.find("_plugin") {
                    &dll_stem[..idx]
                } else {
                    &dll_stem
                };
                // Convert base to PascalCase for directory (e.g., prompt_dispatcher -> PromptDispatcher)
                let cap = base
                    .split('_')
                    .filter(|s| !s.is_empty())
                    .map(|s| {
                        let mut chars = s.chars();
                        match chars.next() {
                            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                            None => String::new(),
                        }
                    })
                    .collect::<String>();
                let plugin_dir = std::path::Path::new(&plugins_dir).join(format!("{}Plugin", cap));
                let manifest_path = plugin_dir.join("plugin.yaml");
                println!("[DIAG] Checking manifest at: {}", manifest_path.canonicalize().unwrap_or(manifest_path.clone()).display());
                let manifest_str = match fs::read_to_string(&manifest_path) {
                    Ok(s) => s,
                    Err(_) => {
                        println!("[DIAG] Skipping {}: missing manifest {}", path.display(), manifest_path.display());
                        continue;
                    }
                };
                let manifest: serde_yaml::Value = match serde_yaml::from_str(&manifest_str) {
                    Ok(m) => m,
                    Err(e) => {
                        println!("[DIAG] Skipping {}: malformed manifest {}: {}", path.display(), manifest_path.display(), e);
                        continue;
                    }
                };
                let required_fields = ["name", "version", "description", "input", "output"];
                let mut missing = Vec::new();
                for field in &required_fields {
                    if !manifest.get(*field).is_some() {
                        missing.push(*field);
                    }
                }
                if !missing.is_empty() {
                    println!("[DIAG] Skipping {}: manifest {} missing fields {:?}", path.display(), manifest_path.display(), missing);
                    continue;
                }
                unsafe {
                    match Library::new(&path) {
                        Ok(lib) => {
                            let vtable_fn: libloading::Symbol<unsafe extern "C" fn() -> lao_plugin_api::PluginVTablePtr> = match lib.get(b"plugin_vtable") {
                                Ok(f) => f,
                                Err(_) => {
                                    println!("[DIAG] Skipping {}: missing plugin_vtable symbol", path.display());
                                    continue;
                                }
                            };
                            let vtable = vtable_fn();
                            if (*vtable).version != 1 {
                                println!("[DIAG] Skipping {}: vtable version {} != 1", path.display(), (*vtable).version);
                                continue;
                            }
                            let name_ptr = ((*vtable).name)();
                            let name = if !name_ptr.is_null() {
                                std::ffi::CStr::from_ptr(name_ptr).to_string_lossy().to_string()
                            } else {
                                println!("[DIAG] Skipping {}: plugin name is null", path.display());
                                continue;
                            };
                            let run_with_buffer = (*vtable).run_with_buffer;
                            self.plugins.insert(name.clone(), LoadedPlugin { vtable, lib, run_with_buffer: Some(run_with_buffer) });
                            println!("[DIAG] Loaded plugin: {} from {}", name, path.display());
                        }
                        Err(e) => {
                            println!("[DIAG] Skipping {}: failed to load library: {}", path.display(), e);
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