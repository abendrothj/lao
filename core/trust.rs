//! Trust policy for dangerous plugin capabilities.
//!
//! Default-deny: filesystem, shell, network, and subprocess capabilities require
//! explicit opt-in via `lao.toml` or `LAO_ALLOW_SHELL=1` (shell only).
//!
//! Filesystem access requires explicit `filesystem_roots` — no implicit cwd/repo roots.

use crate::path_policy::{canonicalize_path, path_within_roots};
use crate::workflow_types::Workflow;
use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityClass {
    FilesystemRead,
    FilesystemWrite,
    FilesystemEnumerate,
    Shell,
    Network,
    Subprocess,
    /// Read-only access to the structural code graph via a `CodeIntelligenceProvider`.
    CodeGraphRead,
}

#[derive(Debug, Clone, Default)]
pub struct TrustPolicy {
    pub allow_filesystem_read: bool,
    pub allow_filesystem_write: bool,
    pub allow_filesystem_enumerate: bool,
    pub allow_shell: bool,
    pub allow_network: bool,
    pub allow_subprocess: bool,
    pub allow_code_graph_read: bool,
    pub allow_plugins: HashSet<String>,
    /// Explicit filesystem roots (required for any filesystem capability).
    pub filesystem_roots: Vec<PathBuf>,
    /// Allowed network endpoint prefixes (host, host:port, or URL prefix).
    pub network_endpoints: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct TrustTomlSection {
    allow_filesystem_read: Option<bool>,
    allow_filesystem_write: Option<bool>,
    allow_filesystem_enumerate: Option<bool>,
    allow_shell: Option<bool>,
    allow_network: Option<bool>,
    allow_subprocess: Option<bool>,
    allow_code_graph_read: Option<bool>,
    allow_plugins: Option<Vec<String>>,
    filesystem_roots: Option<Vec<String>>,
    network_endpoints: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct LaoToml {
    trust: Option<TrustTomlSection>,
}

impl TrustPolicy {
    pub fn load_default() -> Self {
        Self::load()
    }

    pub fn load() -> Self {
        let mut policy = Self::default();

        if let Ok(path) = env::var("LAO_CONFIG") {
            policy.merge_from_file(&path);
        } else {
            for candidate in ["lao.toml", "config/lao.toml"] {
                if PathBuf::from(candidate).exists() {
                    policy.merge_from_file(candidate);
                    break;
                }
            }
        }

        if env::var("LAO_ALLOW_SHELL")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            policy.allow_shell = true;
        }

        policy.normalize_roots();
        policy
    }

    fn merge_from_file(&mut self, path: &str) {
        let Ok(contents) = fs::read_to_string(path) else {
            return;
        };
        let Ok(parsed) = toml::from_str::<LaoToml>(&contents) else {
            tracing::warn!("Failed to parse trust config at {}", path);
            return;
        };
        if let Some(trust) = parsed.trust {
            if let Some(v) = trust.allow_filesystem_read {
                self.allow_filesystem_read = v;
            }
            if let Some(v) = trust.allow_filesystem_write {
                self.allow_filesystem_write = v;
            }
            if let Some(v) = trust.allow_filesystem_enumerate {
                self.allow_filesystem_enumerate = v;
            }
            if let Some(v) = trust.allow_shell {
                self.allow_shell = v;
            }
            if let Some(v) = trust.allow_network {
                self.allow_network = v;
            }
            if let Some(v) = trust.allow_subprocess {
                self.allow_subprocess = v;
            }
            if let Some(v) = trust.allow_code_graph_read {
                self.allow_code_graph_read = v;
            }
            if let Some(plugins) = trust.allow_plugins {
                self.allow_plugins.extend(plugins);
            }
            if let Some(roots) = trust.filesystem_roots {
                self.filesystem_roots
                    .extend(roots.into_iter().map(PathBuf::from));
            }
            if let Some(endpoints) = trust.network_endpoints {
                self.network_endpoints.extend(endpoints);
            }
        }
    }

    fn normalize_roots(&mut self) {
        let mut canonical = Vec::new();
        for root in &self.filesystem_roots {
            if let Ok(abs) = fs::canonicalize(root) {
                canonical.push(abs);
            } else if let Ok(norm) = canonicalize_path(root.to_string_lossy().as_ref()) {
                canonical.push(norm);
            }
        }
        self.filesystem_roots = canonical;
    }

    pub fn allows_class(&self, class: CapabilityClass) -> bool {
        match class {
            CapabilityClass::FilesystemRead => self.allow_filesystem_read,
            CapabilityClass::FilesystemWrite => self.allow_filesystem_write,
            CapabilityClass::FilesystemEnumerate => self.allow_filesystem_enumerate,
            CapabilityClass::Shell => self.allow_shell,
            CapabilityClass::Network => self.allow_network,
            CapabilityClass::Subprocess => self.allow_subprocess,
            CapabilityClass::CodeGraphRead => self.allow_code_graph_read,
        }
    }

    pub fn allows_plugin(&self, plugin_name: &str, class: CapabilityClass) -> bool {
        if self.allow_plugins.contains(plugin_name) {
            return true;
        }
        self.allows_class(class)
    }

    pub fn validate_workflow(&self, workflow: &Workflow) -> Result<(), String> {
        for step in &workflow.steps {
            if let Some(class) = capability_class_for_plugin(&step.run) {
                if !self.allows_plugin(&step.run, class) {
                    return Err(format!(
                        "workflow uses untrusted plugin '{}' ({:?}); configure lao.toml [trust]",
                        step.run, class
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn validate_filesystem_path(
        &self,
        raw_path: &str,
        class: CapabilityClass,
    ) -> Result<PathBuf, String> {
        if !self.allows_class(class) {
            return Err(format!(
                "filesystem capability {:?} is not allowed by trust policy",
                class
            ));
        }
        if self.filesystem_roots.is_empty() {
            return Err(
                "filesystem access denied: set trust.filesystem_roots in lao.toml (explicit roots only)"
                    .to_string(),
            );
        }
        let canonical = canonicalize_path(raw_path)?;
        let absolute = if canonical.is_absolute() {
            canonical
        } else if let Ok(cwd) = env::current_dir() {
            cwd.join(canonical)
        } else {
            canonical
        };
        if !path_within_roots(&absolute, &self.filesystem_roots) {
            return Err(format!(
                "path '{}' is outside configured filesystem_roots",
                raw_path.trim()
            ));
        }
        Ok(absolute)
    }

    pub fn validate_network_endpoint(&self, endpoint: &str) -> Result<(), String> {
        if !self.allow_network && self.allow_plugins.is_empty() {
            return Err("network access is not allowed by trust policy".to_string());
        }
        if self.network_endpoints.is_empty() && !self.allow_network {
            return Err(
                "network access denied: set trust.network_endpoints or allow_network in lao.toml"
                    .to_string(),
            );
        }
        if self.network_endpoints.is_empty() {
            // allow_network=true with no endpoint list: permit any (documented escape hatch)
            return Ok(());
        }
        let endpoint = endpoint.trim().to_lowercase();
        let allowed = self.network_endpoints.iter().any(|e| {
            let e = e.trim().to_lowercase();
            endpoint == e || endpoint.starts_with(&format!("{}/", e)) || endpoint.contains(&e)
        });
        if allowed {
            Ok(())
        } else {
            Err(format!(
                "network endpoint '{}' is not in trust.network_endpoints",
                endpoint
            ))
        }
    }

    /// Validate step input for known dangerous plugins before execution.
    pub fn validate_step_input(&self, plugin_name: &str, input_text: &str) -> Result<(), String> {
        match plugin_name {
            "FileReadPlugin" => {
                self.validate_filesystem_path(input_text, CapabilityClass::FilesystemRead)?;
            }
            "FolderMapPlugin" => {
                self.validate_filesystem_path(input_text, CapabilityClass::FilesystemEnumerate)?;
            }
            "MarkdownReportPlugin" => {
                // Input is markdown body; output path comes from workflow params — checked separately.
            }
            "SummarizerPlugin" => {
                self.require_capability(plugin_name, CapabilityClass::Network)?;
                self.validate_network_endpoint("http://127.0.0.1:11434")?;
            }
            "PromptDispatcherPlugin" => {
                self.require_capability(plugin_name, CapabilityClass::Network)?;
            }
            "ShellCommandPlugin" => {
                self.require_capability(plugin_name, CapabilityClass::Shell)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn require_capability(&self, plugin_name: &str, class: CapabilityClass) -> Result<(), String> {
        if !self.allows_plugin(plugin_name, class) {
            return Err(format!(
                "plugin '{}' requires {:?} trust; configure lao.toml [trust]",
                plugin_name, class
            ));
        }
        Ok(())
    }

    /// Reconcile a plugin's declared manifest capabilities with the runtime policy.
    ///
    /// This is the manifest-driven enforcement path: any plugin (including future or
    /// third-party ones) is gated by the capability classes it declares, not by a
    /// hardcoded plugin-name list. Unknown capability names are treated as safe.
    pub fn check_manifest_capabilities(
        &self,
        plugin_name: &str,
        capabilities: &[lao_plugin_api::PluginCapability],
    ) -> Result<(), String> {
        for cap in capabilities {
            let Some(class) = capability_class_for_manifest(&cap.name) else {
                continue;
            };
            self.require_capability(plugin_name, class).map_err(|_| {
                format!(
                    "plugin '{}' declares capability '{}' but trust policy denies {:?}",
                    plugin_name, cap.name, class
                )
            })?;
        }
        Ok(())
    }

    /// Validate filesystem path from workflow step params (e.g. MarkdownReport output_path).
    pub fn validate_param_path(
        &self,
        path: &str,
        class: CapabilityClass,
    ) -> Result<PathBuf, String> {
        self.validate_filesystem_path(path, class)
    }
}

/// Map a declared manifest capability name to a trust capability class.
/// Accepts both the bundled plugins' functional names and canonical class names so
/// external plugins can declare either form.
/// Map a declared manifest capability name to a trust capability class. `pub(crate)` so
/// `execution::descriptor::synthetic_descriptor` can reuse the same table instead of
/// duplicating it.
pub(crate) fn capability_class_for_manifest(capability_name: &str) -> Option<CapabilityClass> {
    match capability_name {
        "read-file" | "file-read" | "file_read" | "filesystem_read" => {
            Some(CapabilityClass::FilesystemRead)
        }
        "write-file" | "file-write" | "file_write" | "filesystem_write" | "markdown-report" => {
            Some(CapabilityClass::FilesystemWrite)
        }
        "map-folder"
        | "list-folder"
        | "directory-list"
        | "directory_list"
        | "filesystem_enumerate" => Some(CapabilityClass::FilesystemEnumerate),
        "run-shell" | "shell" | "shell-command" | "shell_execute" => Some(CapabilityClass::Shell),
        "summarize" | "prompt-dispatch" | "network" | "http" => Some(CapabilityClass::Network),
        "speech-to-text" | "subprocess" => Some(CapabilityClass::Subprocess),
        "code-graph-read" | "code_graph_read" | "codegraph-read" => {
            Some(CapabilityClass::CodeGraphRead)
        }
        _ => None,
    }
}

fn capability_class_for_plugin(plugin_name: &str) -> Option<CapabilityClass> {
    match plugin_name {
        "FileReadPlugin" => Some(CapabilityClass::FilesystemRead),
        "FolderMapPlugin" => Some(CapabilityClass::FilesystemEnumerate),
        "MarkdownReportPlugin" => Some(CapabilityClass::FilesystemWrite),
        "ShellCommandPlugin" => Some(CapabilityClass::Shell),
        "SummarizerPlugin" | "PromptDispatcherPlugin" => Some(CapabilityClass::Network),
        "WhisperPlugin" => Some(CapabilityClass::Subprocess),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_denies_dangerous_capabilities() {
        let policy = TrustPolicy::default();
        assert!(!policy.allows_plugin("ShellCommandPlugin", CapabilityClass::Shell));
        assert!(!policy.allows_plugin("FileReadPlugin", CapabilityClass::FilesystemRead));
        assert!(!policy.allows_plugin("SummarizerPlugin", CapabilityClass::Network));
        assert!(!policy.allows_class(CapabilityClass::CodeGraphRead));
    }

    #[test]
    fn code_graph_read_capability_name_maps_and_is_deny_by_default() {
        assert_eq!(
            capability_class_for_manifest("code-graph-read"),
            Some(CapabilityClass::CodeGraphRead)
        );
        let mut policy = TrustPolicy::default();
        assert!(!policy.allows_class(CapabilityClass::CodeGraphRead));
        policy.allow_code_graph_read = true;
        assert!(policy.allows_class(CapabilityClass::CodeGraphRead));
    }

    #[test]
    fn explicit_roots_required_for_filesystem() {
        let mut policy = TrustPolicy::default();
        policy.allow_filesystem_read = true;
        assert!(policy
            .validate_filesystem_path("./foo.txt", CapabilityClass::FilesystemRead)
            .is_err());
        policy.filesystem_roots.push(PathBuf::from("."));
        policy.normalize_roots();
        assert!(policy
            .validate_filesystem_path("./foo.txt", CapabilityClass::FilesystemRead)
            .is_ok());
    }

    #[test]
    fn allow_plugins_overrides_class() {
        let mut policy = TrustPolicy::default();
        policy
            .allow_plugins
            .insert("ShellCommandPlugin".to_string());
        assert!(policy.allows_plugin("ShellCommandPlugin", CapabilityClass::Shell));
    }

    #[test]
    fn path_outside_roots_is_rejected() {
        let mut policy = TrustPolicy::default();
        policy.allow_filesystem_read = true;
        policy.filesystem_roots.push(PathBuf::from("/tmp/allowed"));
        policy.normalize_roots();
        let err = policy
            .validate_filesystem_path("/etc/passwd", CapabilityClass::FilesystemRead)
            .unwrap_err();
        assert!(err.contains("outside configured filesystem_roots"));
    }

    #[test]
    fn parent_traversal_is_rejected_even_with_roots() {
        let mut policy = TrustPolicy::default();
        policy.allow_filesystem_read = true;
        policy.filesystem_roots.push(PathBuf::from("/tmp"));
        policy.normalize_roots();
        assert!(policy
            .validate_filesystem_path("../../etc/passwd", CapabilityClass::FilesystemRead)
            .is_err());
    }

    #[test]
    fn network_denied_without_trust() {
        let policy = TrustPolicy::default();
        assert!(policy
            .validate_network_endpoint("http://127.0.0.1:11434")
            .is_err());
    }

    #[test]
    fn network_allowed_with_open_flag() {
        let mut policy = TrustPolicy::default();
        policy.allow_network = true;
        assert!(policy
            .validate_network_endpoint("http://example.com")
            .is_ok());
    }

    #[test]
    fn network_endpoint_allowlist_enforced() {
        let mut policy = TrustPolicy::default();
        policy.allow_network = true;
        policy
            .network_endpoints
            .push("http://127.0.0.1:11434".to_string());
        assert!(policy
            .validate_network_endpoint("http://127.0.0.1:11434/api")
            .is_ok());
        assert!(policy
            .validate_network_endpoint("http://evil.example.com")
            .is_err());
    }

    #[test]
    fn validate_step_input_blocks_untrusted_shell() {
        let policy = TrustPolicy::default();
        assert!(policy
            .validate_step_input("ShellCommandPlugin", "ls")
            .is_err());
    }

    #[test]
    fn validate_step_input_allows_trusted_shell() {
        let mut policy = TrustPolicy::default();
        policy.allow_shell = true;
        assert!(policy
            .validate_step_input("ShellCommandPlugin", "ls")
            .is_ok());
    }

    #[test]
    fn validate_step_input_enforces_file_roots() {
        let mut policy = TrustPolicy::default();
        policy.allow_filesystem_read = true;
        policy.filesystem_roots.push(PathBuf::from("/tmp/data"));
        policy.normalize_roots();
        assert!(policy
            .validate_step_input("FileReadPlugin", "/etc/shadow")
            .is_err());
        assert!(policy
            .validate_step_input("FileReadPlugin", "/tmp/data/notes.txt")
            .is_ok());
    }

    #[test]
    fn validate_step_input_ignores_unknown_plugins() {
        let policy = TrustPolicy::default();
        assert!(policy.validate_step_input("EchoPlugin", "anything").is_ok());
    }

    #[test]
    fn manifest_capabilities_reconciled_with_policy() {
        use lao_plugin_api::{PluginCapability, PluginInputType, PluginOutputType};
        let policy = TrustPolicy::default();
        let caps = vec![PluginCapability {
            name: "shell".to_string(),
            description: String::new(),
            input_type: PluginInputType::Any,
            output_type: PluginOutputType::Text,
        }];
        assert!(policy
            .check_manifest_capabilities("ShellCommandPlugin", &caps)
            .is_err());

        let mut trusted = TrustPolicy::default();
        trusted.allow_shell = true;
        assert!(trusted
            .check_manifest_capabilities("ShellCommandPlugin", &caps)
            .is_ok());
    }

    #[test]
    fn manifest_uses_declared_functional_names() {
        use lao_plugin_api::{PluginCapability, PluginInputType, PluginOutputType};
        // The bundled FileReadPlugin declares the functional name "read-file"; trust
        // enforcement must recognize it without a hardcoded plugin-name allowlist.
        let caps = vec![PluginCapability {
            name: "read-file".to_string(),
            description: String::new(),
            input_type: PluginInputType::Any,
            output_type: PluginOutputType::Text,
        }];

        let denied = TrustPolicy::default();
        assert!(denied
            .check_manifest_capabilities("FileReadPlugin", &caps)
            .is_err());

        let mut trusted = TrustPolicy::default();
        trusted.allow_filesystem_read = true;
        trusted.filesystem_roots = vec![std::path::PathBuf::from("/tmp")];
        assert!(trusted
            .check_manifest_capabilities("FileReadPlugin", &caps)
            .is_ok());
    }

    #[test]
    fn workflow_with_untrusted_plugin_is_rejected() {
        use crate::workflow_types::{Workflow, WorkflowStep};
        let workflow = Workflow {
            workflow: "wf".to_string(),
            steps: vec![WorkflowStep {
                run: "ShellCommandPlugin".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: None,
                depends_on: None,
                for_each: None,
                condition: None,
            }],
        };
        assert!(TrustPolicy::default().validate_workflow(&workflow).is_err());
    }
}
