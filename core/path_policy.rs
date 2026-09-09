use std::path::{Component, Path, PathBuf};

/// Validate an identifier that will be interpolated into a filesystem path
/// (cache keys, workflow ids, workflow file names).
///
/// Allows only `[A-Za-z0-9._-]`, rejects `..`, path separators, and empty or
/// oversized values, so an identifier can never traverse outside its directory.
pub fn validate_identifier(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("identifier must not be empty".to_string());
    }
    if id.len() > 200 {
        return Err("identifier too long (max 200 characters)".to_string());
    }
    if id.contains("..") {
        return Err(format!("identifier '{}' must not contain '..'", id));
    }
    if let Some(bad) = id
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')))
    {
        return Err(format!(
            "identifier '{}' contains invalid character '{}' (allowed: A-Z a-z 0-9 . _ -)",
            id, bad
        ));
    }
    Ok(())
}

/// Canonicalize a path for trust checks without requiring the target to exist.
pub fn canonicalize_path(path: &str) -> Result<PathBuf, String> {
    let path = Path::new(path.trim());
    if path.as_os_str().is_empty() {
        return Err("empty path".to_string());
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(p) => normalized.push(p.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err("path escapes above root via '..'".to_string());
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err("path resolved to empty".to_string());
    }

    Ok(normalized)
}

/// Resolve symlinks in `path` by canonicalizing its deepest existing ancestor and
/// re-joining the (already lexically normalized) remainder.
///
/// This makes trust checks compare canonical candidate paths against canonical
/// roots: a symlink inside an allowed root can no longer escape containment, and
/// platform aliases (e.g. macOS `/tmp` -> `/private/tmp`) match correctly even
/// when the target file does not exist yet.
pub fn resolve_symlinks_best_effort(path: &Path) -> PathBuf {
    if let Ok(canonical) = std::fs::canonicalize(path) {
        return canonical;
    }
    let mut ancestor = path.to_path_buf();
    while ancestor.pop() {
        if let Ok(canonical) = std::fs::canonicalize(&ancestor) {
            let mut resolved = canonical;
            // Re-append the non-existing tail components (normalized: no `..`).
            let tail = path.strip_prefix(&ancestor).unwrap_or(path);
            for component in tail.components() {
                resolved.push(component.as_os_str());
            }
            return resolved;
        }
        if ancestor.as_os_str().is_empty() {
            break;
        }
    }
    path.to_path_buf()
}

pub fn path_within_roots(path: &Path, roots: &[PathBuf]) -> bool {
    if roots.is_empty() {
        return false;
    }
    roots.iter().any(|root| path.starts_with(root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_traversal() {
        assert!(canonicalize_path("../etc/passwd").is_err());
    }

    #[test]
    fn normalizes_relative_path() {
        let p = canonicalize_path("./foo/bar").unwrap();
        assert_eq!(p, PathBuf::from("foo/bar"));
    }

    #[test]
    fn identifier_accepts_safe_names() {
        assert!(validate_identifier("my-workflow_1.0").is_ok());
        assert!(validate_identifier("EchoPlugin-1.0.0-abc123").is_ok());
    }

    #[test]
    fn identifier_rejects_traversal_and_separators() {
        assert!(validate_identifier("../escape").is_err());
        assert!(validate_identifier("a/b").is_err());
        assert!(validate_identifier("a\\b").is_err());
        assert!(validate_identifier("").is_err());
        assert!(validate_identifier("a..b").is_err());
    }

    #[test]
    fn resolves_nonexistent_tail_against_existing_ancestor() {
        let dir = std::env::temp_dir();
        let target = dir.join("lao_path_policy_test_missing").join("leaf.txt");
        let resolved = resolve_symlinks_best_effort(&target);
        assert!(resolved.ends_with("lao_path_policy_test_missing/leaf.txt"));
        // The existing ancestor prefix is canonical (e.g. /private/tmp on macOS).
        let canonical_dir = std::fs::canonicalize(&dir).unwrap();
        assert!(resolved.starts_with(&canonical_dir));
    }
}
