use std::collections::HashMap;
use std::ffi::CStr;
use std::path::Path;
use std::sync::Arc;
use lao_plugin_api::*;
use libloading::{Library, Symbol};

#[derive(Debug, Clone)]
pub struct PluginInstance {
    pub library: Arc<Library>,
    pub vtable: PluginVTablePtr,
    pub info: PluginInfo,
    pub metadata: PluginInfo, // Use PluginInfo instead of PluginMetadata for Debug/Clone
}

impl PluginInstance {
    pub fn new(library: Library, vtable: PluginVTablePtr) -> Result<Self, String> {
        unsafe {
            println!("[DEBUG] Creating PluginInstance with vtable: {:?}", vtable);
            
            // Check if vtable is valid
            if vtable.is_null() {
                return Err("VTable pointer is null".to_string());
            }
            
            let vtable_ref = &*vtable;
            println!("[DEBUG] VTable version: {}", vtable_ref.version);
            println!("[DEBUG] VTable get_metadata function pointer: {:?}", vtable_ref.get_metadata);
            
            let metadata = (vtable_ref.get_metadata)();
            println!("[DEBUG] Got metadata from plugin");
            
            let info = PluginInfo::from_metadata(&metadata);
            println!("[DEBUG] Created PluginInfo from metadata");
            
            Ok(PluginInstance {
                library: Arc::new(library),
                vtable,
                info: info.clone(),
                metadata: info,
            })
        }
    }
    
    pub fn validate_input(&self, input: &PluginInput) -> bool {
        unsafe {
            ((*self.vtable).validate_input)(input)
        }
    }
    
    pub fn get_capabilities(&self) -> Vec<PluginCapability> {
        unsafe {
            let caps_ptr = ((*self.vtable).get_capabilities)();
            if caps_ptr.is_null() {
                return Vec::new();
            }
            
            let caps_str = CStr::from_ptr(caps_ptr).to_string_lossy();
            serde_json::from_str(&caps_str).unwrap_or_default()
        }
    }
}

#[derive(Debug)]
pub struct PluginRegistry {
    pub plugins: HashMap<String, PluginInstance>,
    pub plugin_versions: HashMap<String, Vec<String>>, // name -> versions
    pub plugin_dependencies: HashMap<String, Vec<PluginDependency>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        PluginRegistry {
            plugins: HashMap::new(),
            plugin_versions: HashMap::new(),
            plugin_dependencies: HashMap::new(),
        }
    }
    
    pub fn dynamic_registry(plugin_dir: &str) -> Self {
        let mut registry = PluginRegistry::new();
        registry.load_plugins_from_directory(plugin_dir);
        registry
    }
    
    pub fn load_plugins_from_directory(&mut self, plugin_dir: &str) {
        if let Ok(entries) = std::fs::read_dir(plugin_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    // Check for plugin.yaml manifest
                    let manifest_path = path.join("plugin.yaml");
                    if manifest_path.exists() {
                        if let Ok(manifest_content) = std::fs::read_to_string(&manifest_path) {
                            if let Ok(manifest) = serde_yaml::from_str::<serde_yaml::Value>(&manifest_content) {
                                if let Some(name) = manifest.get("name").and_then(|n| n.as_str()) {
                                    println!("[DIAG] Found plugin directory: {:?}", path);
                                    println!("[DIAG] Checking manifest at: {:?}", manifest_path);
                                    
                                    // Look for DLL in the same directory
                                    let dll_name = format!("{}.dll", name.to_lowercase().replace("plugin", ""));
                                    let dll_path = path.join(&dll_name);
                                    
                                    if dll_path.exists() {
                                        if let Ok(plugin) = self.load_plugin(&dll_path) {
                                            self.register_plugin(plugin);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if path.extension().and_then(|s| s.to_str()) == Some("dll") {
                    // Direct DLL loading
                    if let Ok(plugin) = self.load_plugin(&path) {
                        self.register_plugin(plugin);
                    }
                }
            }
        }
    }
    
    pub fn load_plugin(&self, dll_path: &Path) -> Result<PluginInstance, String> {
        unsafe {
            println!("[DEBUG] Loading plugin from: {}", dll_path.display());
            
            let library = Library::new(dll_path)
                .map_err(|e| format!("Failed to load plugin {}: {}", dll_path.display(), e))?;
            
            println!("[DEBUG] Library loaded successfully");
            
            let plugin_vtable_fn: Symbol<unsafe extern "C" fn() -> PluginVTablePtr> = library
                .get(b"plugin_vtable")
                .map_err(|e| format!("Failed to get plugin_vtable from {}: {}", dll_path.display(), e))?;
            
            println!("[DEBUG] Got plugin_vtable function");
            
            let vtable = plugin_vtable_fn();
            println!("[DEBUG] Called plugin_vtable function, got pointer: {:?}", vtable);
            
            PluginInstance::new(library, vtable)
        }
    }
    
    pub fn register_plugin(&mut self, plugin: PluginInstance) {
        let name = plugin.info.name.clone();
        let version = plugin.info.version.clone();
        let dependencies = plugin.info.dependencies.clone();
        
        // Store plugin
        self.plugins.insert(name.clone(), plugin);
        
        // Track versions
        self.plugin_versions.entry(name.clone()).or_insert_with(Vec::new).push(version);
        
        // Track dependencies
        self.plugin_dependencies.insert(name.clone(), dependencies);
        
        println!("[DIAG] Loaded plugin: {}", name);
    }
    
    pub fn get(&self, name: &str) -> Option<&PluginInstance> {
        self.plugins.get(name)
    }
    
    pub fn get_with_version(&self, name: &str, version: &str) -> Option<&PluginInstance> {
        self.plugins.get(name).filter(|p| p.info.version == version)
    }
    
    pub fn list_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins.values().map(|p| &p.info).collect()
    }
    
    pub fn find_plugins_by_tag(&self, tag: &str) -> Vec<&PluginInfo> {
        self.plugins.values()
            .filter(|p| p.info.tags.iter().any(|t| t == tag))
            .map(|p| &p.info)
            .collect()
    }
    
    pub fn find_plugins_by_capability(&self, capability: &str) -> Vec<&PluginInfo> {
        self.plugins.values()
            .filter(|p| p.info.capabilities.iter().any(|c| c.name == capability))
            .map(|p| &p.info)
            .collect()
    }
    
    pub fn resolve_dependencies(&self, plugin_name: &str) -> Result<Vec<String>, String> {
        let mut resolved = Vec::new();
        let mut visited = std::collections::HashSet::new();
        
        self.resolve_dependencies_recursive(plugin_name, &mut resolved, &mut visited)?;
        
        Ok(resolved)
    }
    
    fn resolve_dependencies_recursive(
        &self,
        plugin_name: &str,
        resolved: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<(), String> {
        if visited.contains(plugin_name) {
            return Ok(()); // Already processed
        }
        
        visited.insert(plugin_name.to_string());
        
        if let Some(dependencies) = self.plugin_dependencies.get(plugin_name) {
            for dep in dependencies {
                if !dep.optional || self.plugins.contains_key(&dep.name) {
                    self.resolve_dependencies_recursive(&dep.name, resolved, visited)?;
                }
            }
        }
        
        resolved.push(plugin_name.to_string());
        Ok(())
    }
    
    pub fn validate_plugin_compatibility(&self, plugin_name: &str) -> Result<(), String> {
        if let Some(plugin) = self.plugins.get(plugin_name) {
            for dep in &plugin.info.dependencies {
                if !self.plugins.contains_key(&dep.name) && !dep.optional {
                    return Err(format!("Missing required dependency: {}", dep.name));
                }
            }
        }
        Ok(())
    }
    
    pub fn update_plugin(&mut self, plugin_name: &str, new_plugin: PluginInstance) -> Result<(), String> {
        if self.plugins.contains_key(plugin_name) {
            // Validate compatibility
            self.validate_plugin_compatibility(plugin_name)?;
            
            // Update plugin
            self.plugins.insert(plugin_name.to_string(), new_plugin);
            Ok(())
        } else {
            Err(format!("Plugin {} not found", plugin_name))
        }
    }
    
    pub fn remove_plugin(&mut self, plugin_name: &str) -> Result<(), String> {
        // Check if other plugins depend on this one
        for (name, deps) in &self.plugin_dependencies {
            if name != plugin_name {
                for dep in deps {
                    if dep.name == plugin_name && !dep.optional {
                        return Err(format!("Cannot remove {}: required by {}", plugin_name, name));
                    }
                }
            }
        }
        
        self.plugins.remove(plugin_name);
        self.plugin_versions.remove(plugin_name);
        self.plugin_dependencies.remove(plugin_name);
        
        Ok(())
    }
} 