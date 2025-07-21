use std::collections::HashMap;
use std::path::PathBuf;

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

pub struct PluginRegistry {
    pub plugins: HashMap<String, Box<dyn LaoPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: HashMap::new() }
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
    pub fn default_registry() -> Self {
        let mut reg = Self::new();
        reg.register(EchoPlugin);
        reg.register(WhisperPlugin);
        reg.register(OllamaPlugin);
        reg.register(LMStudioPlugin);
        reg.register(GGUFPlugin);
        reg
    }
} 