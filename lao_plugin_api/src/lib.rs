use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginInput {
    pub text: Option<String>,
    pub audio: Option<String>, // Path to audio file
    pub json: Option<serde_json::Value>,
    pub tagged_data: Option<Vec<(String, String)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginOutput {
    pub text: Option<String>,
    pub audio: Option<String>,
    pub json: Option<serde_json::Value>,
    pub tagged_data: Option<Vec<(String, String)>>,
}

pub trait Plugin {
    fn run(&self, input: PluginInput) -> Result<PluginOutput, String>;
} 