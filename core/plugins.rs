use std::collections::HashMap;
use std::path::PathBuf;
use libloading::{Library, Symbol};
use std::fs;
use std::ffi::OsStr;

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
pub enum PluginInput {
    Text(String),
    AudioFile(PathBuf),
    Json(serde_json::Value),
    TaggedData(Vec<(String, String)>),
}

#[derive(Debug)]
pub enum PluginOutput {
    Text(String),
    Json(serde_json::Value),
    TaggedData(Vec<(String, String)>),
}

#[derive(Debug, Clone)]
pub struct IOSignature {
    pub input_type: PluginInputType,
    pub output_type: PluginInputType,
    pub description: String,
}

#[derive(Debug)]
pub enum LaoError {
    InitError(String),
    ExecutionError(String),
    ShutdownError(String),
}

pub trait LaoPlugin {
    fn name(&self) -> &'static str;
    fn init(&mut self, config: PluginConfig) -> Result<(), LaoError>;
    fn pre_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError>;
    fn post_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn io_signature(&self) -> IOSignature;
    fn shutdown(&mut self) -> Result<(), LaoError> { Ok(()) }
}

// --- Sample Echo Plugin ---
pub struct EchoPlugin;

impl LaoPlugin for EchoPlugin {
    fn name(&self) -> &'static str {
        "Echo"
    }
    fn init(&mut self, _config: PluginConfig) -> Result<(), LaoError> {
        Ok(())
    }
    fn pre_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError> {
        match input {
            PluginInput::Text(s) => Ok(PluginOutput::Text(s)),
            PluginInput::Json(j) => Ok(PluginOutput::Json(j)),
            PluginInput::TaggedData(t) => Ok(PluginOutput::TaggedData(t)),
            PluginInput::AudioFile(_) => Err(LaoError::ExecutionError("Echo does not support AudioFile".into())),
        }
    }
    fn post_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn io_signature(&self) -> IOSignature {
        IOSignature {
            input_type: PluginInputType::Any,
            output_type: PluginInputType::Any,
            description: "Echoes the input as output".into(),
        }
    }
    fn shutdown(&mut self) -> Result<(), LaoError> { Ok(()) }
}

// --- Whisper.cpp Plugin ---
pub struct WhisperPlugin;

impl LaoPlugin for WhisperPlugin {
    fn name(&self) -> &'static str { "Whisper" }
    fn init(&mut self, _config: PluginConfig) -> Result<(), LaoError> { Ok(()) }
    fn pre_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError> {
        match input {
            PluginInput::AudioFile(path) => {
                let output = std::process::Command::new("./whisper.cpp")
                    .arg(path)
                    .output()
                    .map_err(|e| LaoError::ExecutionError(format!("Failed to run whisper.cpp: {}", e)))?;
                if !output.status.success() {
                    return Err(LaoError::ExecutionError(format!("whisper.cpp failed: {}", String::from_utf8_lossy(&output.stderr))));
                }
                let out_str = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(PluginOutput::Text(out_str))
            }
            _ => Err(LaoError::ExecutionError("Whisper only supports AudioFile input".into())),
        }
    }
    fn post_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn io_signature(&self) -> IOSignature {
        IOSignature {
            input_type: PluginInputType::AudioFile,
            output_type: PluginInputType::Text,
            description: "Transcribe audio using whisper.cpp".into(),
        }
    }
    fn shutdown(&mut self) -> Result<(), LaoError> { Ok(()) }
}

// --- Ollama Plugin (real) ---
pub struct OllamaPlugin;

impl LaoPlugin for OllamaPlugin {
    fn name(&self) -> &'static str { "Ollama" }
    fn init(&mut self, _config: PluginConfig) -> Result<(), LaoError> { Ok(()) }
    fn pre_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError> {
        match input {
            PluginInput::Text(prompt) => {
                let output = std::process::Command::new("ollama")
                    .arg("run")
                    .arg("mistral") // You may want to make model configurable
                    .arg(&prompt)
                    .output()
                    .map_err(|e| LaoError::ExecutionError(format!("Failed to run ollama: {}", e)))?;
                if !output.status.success() {
                    return Err(LaoError::ExecutionError(format!("ollama failed: {}", String::from_utf8_lossy(&output.stderr))));
                }
                let out_str = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(PluginOutput::Text(out_str))
            }
            _ => Err(LaoError::ExecutionError("Ollama only supports Text input".into())),
        }
    }
    fn post_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn io_signature(&self) -> IOSignature {
        IOSignature {
            input_type: PluginInputType::Text,
            output_type: PluginInputType::Text,
            description: "Run LLM inference using Ollama".into(),
        }
    }
    fn shutdown(&mut self) -> Result<(), LaoError> { Ok(()) }
}

// --- LM Studio Plugin ---
pub struct LMStudioPlugin;

impl LaoPlugin for LMStudioPlugin {
    fn name(&self) -> &'static str { "LMStudio" }
    fn init(&mut self, _config: PluginConfig) -> Result<(), LaoError> { Ok(()) }
    fn pre_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError> {
        match input {
            PluginInput::Text(prompt) => {
                Ok(PluginOutput::Text(format!("[LM Studio output for prompt: {}]", prompt)))
            }
            _ => Err(LaoError::ExecutionError("LM Studio only supports Text input".into())),
        }
    }
    fn post_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn io_signature(&self) -> IOSignature {
        IOSignature {
            input_type: PluginInputType::Text,
            output_type: PluginInputType::Text,
            description: "Offline LLM via LM Studio".into(),
        }
    }
    fn shutdown(&mut self) -> Result<(), LaoError> { Ok(()) }
}

// --- GGUF Plugin ---
pub struct GGUFPlugin;

impl LaoPlugin for GGUFPlugin {
    fn name(&self) -> &'static str { "GGUF" }
    fn init(&mut self, _config: PluginConfig) -> Result<(), LaoError> { Ok(()) }
    fn pre_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError> {
        match input {
            PluginInput::Text(prompt) => {
                Ok(PluginOutput::Text(format!("[GGUF output for prompt: {}]", prompt)))
            }
            _ => Err(LaoError::ExecutionError("GGUF only supports Text input".into())),
        }
    }
    fn post_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn io_signature(&self) -> IOSignature {
        IOSignature {
            input_type: PluginInputType::Text,
            output_type: PluginInputType::Text,
            description: "Offline LLM via GGUF".into(),
        }
    }
    fn shutdown(&mut self) -> Result<(), LaoError> { Ok(()) }
}

// --- Prompt Dispatcher Plugin ---
pub struct PromptDispatcherPlugin;

impl LaoPlugin for PromptDispatcherPlugin {
    fn name(&self) -> &'static str { "PromptDispatcher" }
    fn init(&mut self, _config: PluginConfig) -> Result<(), LaoError> { Ok(()) }
    fn pre_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError> {
        // Embed the system prompt at compile time
        let system_prompt = include_str!("../core/prompt_dispatcher/prompt/system_prompt.txt");
        match input {
            PluginInput::Text(prompt) => {
                let full_prompt = format!("{}\n{}", system_prompt, prompt);
                let output = std::process::Command::new("ollama")
                    .arg("run")
                    .arg("mistral")
                    .arg(&full_prompt)
                    .output()
                    .map_err(|e| LaoError::ExecutionError(format!("Failed to run ollama: {}", e)))?;
                if !output.status.success() {
                    return Err(LaoError::ExecutionError(format!("ollama failed: {}", String::from_utf8_lossy(&output.stderr))));
                }
                let out_str = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(PluginOutput::Text(out_str))
            }
            _ => Err(LaoError::ExecutionError("PromptDispatcher only supports Text input".into())),
        }
    }
    fn post_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn io_signature(&self) -> IOSignature {
        IOSignature {
            input_type: PluginInputType::Text,
            output_type: PluginInputType::Text,
            description: "Generate a workflow plan from a prompt using a local LLM (Ollama)".into(),
        }
    }
    fn shutdown(&mut self) -> Result<(), LaoError> { Ok(()) }
}

// --- Summarizer Plugin ---
pub struct SummarizerPlugin;

impl LaoPlugin for SummarizerPlugin {
    fn name(&self) -> &'static str { "Summarizer" }
    fn init(&mut self, _config: PluginConfig) -> Result<(), LaoError> { Ok(()) }
    fn pre_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError> {
        match input {
            PluginInput::Text(text) => {
                // Use a local LLM to summarize the text (e.g., ollama run mistral)
                let prompt = format!("Summarize the following text:\n{}", text);
                let output = std::process::Command::new("ollama")
                    .arg("run")
                    .arg("mistral")
                    .arg(&prompt)
                    .output()
                    .map_err(|e| LaoError::ExecutionError(format!("Failed to run ollama: {}", e)))?;
                if !output.status.success() {
                    return Err(LaoError::ExecutionError(format!("ollama failed: {}", String::from_utf8_lossy(&output.stderr))));
                }
                let out_str = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(PluginOutput::Text(out_str))
            }
            _ => Err(LaoError::ExecutionError("Summarizer only supports Text input".into())),
        }
    }
    fn post_execute(&mut self) -> Result<(), LaoError> { Ok(()) }
    fn io_signature(&self) -> IOSignature {
        IOSignature {
            input_type: PluginInputType::Text,
            output_type: PluginInputType::Text,
            description: "Summarize input text using a local LLM (Ollama)".into(),
        }
    }
    fn shutdown(&mut self) -> Result<(), LaoError> { Ok(()) }
}

pub struct PluginRegistry {
    pub plugins: HashMap<String, Box<dyn LaoPlugin>>,
    pub libraries: Vec<Library>, // Hold libraries to keep them loaded
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: HashMap::new(), libraries: Vec::new() }
    }
    pub fn register<P: LaoPlugin + 'static>(&mut self, plugin: P) {
        self.plugins.insert(plugin.name().to_string(), Box::new(plugin));
    }
    pub fn get(&self, name: &str) -> Option<&Box<dyn LaoPlugin>> {
        self.plugins.get(name)
    }
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn LaoPlugin>> {
        self.plugins.get_mut(name)
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
                    let ext = path.extension().and_then(OsStr::to_str).unwrap_or("");
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
                                    let plugin_entry: Result<Symbol<unsafe extern fn() -> *mut dyn LaoPlugin>, _> = lib.get(b"plugin_entry_point");
                                    match plugin_entry {
                                        Ok(entry_fn) => {
                                            let boxed_raw = entry_fn();
                                            if !boxed_raw.is_null() {
                                                let boxed: Box<dyn LaoPlugin> = Box::from_raw(boxed_raw);
                                                let name = boxed.name().to_string();
                                                self.plugins.insert(name, boxed);
                                                self.libraries.push(lib); // Keep the library loaded
                                            }
                                        }
                                        Err(_) => { /* Not a valid plugin, skip */ }
                                    }
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
} 