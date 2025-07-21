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
    fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError>;
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
    fn execute(&self, input: PluginInput) -> Result<PluginOutput, LaoError> {
        match input {
            PluginInput::Text(s) => Ok(PluginOutput::Text(s)),
            PluginInput::Json(j) => Ok(PluginOutput::Json(j)),
            PluginInput::TaggedData(t) => Ok(PluginOutput::TaggedData(t)),
            PluginInput::AudioFile(_) => Err(LaoError::ExecutionError("Echo does not support AudioFile".into())),
        }
    }
    fn io_signature(&self) -> IOSignature {
        IOSignature {
            input_type: PluginInputType::Any,
            output_type: PluginInputType::Any,
            description: "Echoes the input as output".into(),
        }
    }
}

pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn LaoPlugin>>,
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
    pub fn default_registry() -> Self {
        let mut reg = Self::new();
        reg.register(EchoPlugin);
        reg
    }
} 