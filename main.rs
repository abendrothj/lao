use std::path::PathBuf;

fn resolve_plugins_dir() -> String {
    // Check environment variable first
    if let Ok(dir) = std::env::var("LAO_PLUGINS_DIR") {
        if std::path::Path::new(&dir).exists() { 
            return dir; 
        }
    }
    
    // Get current working directory and try to find plugins relative to it
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    println!("Current directory: {}", current_dir.display());
    
    // Try multiple relative paths from current directory
    let candidates = [
        "plugins",
        "../plugins", 
        "../../plugins",
        "../../../plugins",
        "../../../../plugins",
    ];
    
    for candidate in &candidates {
        let path = current_dir.join(candidate);
        println!("Trying: {}", path.display());
        if path.exists() && path.is_dir() {
            println!("Found plugins directory: {}", path.display());
            return path.to_string_lossy().to_string();
        }
    }
    
    // Fallback: try to find plugins relative to the executable location
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            println!("Executable directory: {}", exe_dir.display());
            for candidate in &candidates {
                let path = exe_dir.join(candidate);
                println!("Trying from exe: {}", path.display());
                if path.exists() && path.is_dir() {
                    println!("Found plugins directory from exe: {}", path.display());
                    return path.to_string_lossy().to_string();
                }
            }
        }
    }
    
    // Last resort: return current directory + plugins
    let fallback = current_dir.join("plugins");
    println!("Using fallback: {}", fallback.display());
    fallback.to_string_lossy().to_string()
}

fn list_plugins_for_ui() -> Result<Vec<String>, String> {
    let plugins_dir = resolve_plugins_dir();
    println!("DEBUG: Resolved plugins directory: {}", plugins_dir);
    
    let mut out: Vec<String> = Vec::new();

    // Primary: scan manifests for a simple, robust list
    if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let manifest = p.join("plugin.yaml");
                if manifest.exists() {
                    if let Ok(txt) = std::fs::read_to_string(&manifest) {
                        if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(&txt) {
                            let name = val.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            if !name.is_empty() && !out.iter().any(|i| i == &name) {
                                out.push(name);
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: scan shared libs for names if no manifests or additional libs present
    if let Ok(files) = std::fs::read_dir(&plugins_dir) {
        for f in files.flatten() {
            let path = f.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if matches!(ext, "so" | "dll" | "dylib") {
                    if let Some(fname) = path.file_stem().and_then(|s| s.to_str()) {
                        // strip common prefixes like lib
                        let base = fname.strip_prefix("lib").unwrap_or(fname);
                        // keep as-is; UI will display
                        if !out.iter().any(|i| i.eq_ignore_ascii_case(base)) {
                            out.push(base.to_string());
                        }
                    }
                }
            }
        }
    }

    // Sort by name for consistent UI
    out.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    println!("DEBUG: Found {} plugins: {:?}", out.len(), out);
    Ok(out)
}

fn main() {
    println!("Testing plugin loading fix...");
    match list_plugins_for_ui() {
        Ok(plugins) => {
            println!("SUCCESS: Found {} plugins", plugins.len());
            for plugin in plugins {
                println!("  - {}", plugin);
            }
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
        }
    }
}
