use crate::error::{KurultaiError, Result};
use std::path::{Component, Path, PathBuf};

/// Resolve a user-supplied path and ensure it does not escape via `..` traversal.
pub fn resolve_allowed_path(path: &str) -> Result<PathBuf> {
    let expanded = expand_tilde(path)?;
    let canonical = if expanded.exists() {
        expanded
            .canonicalize()
            .map_err(|e| KurultaiError::security(format!("cannot canonicalize '{}': {e}", path)))?
    } else {
        // For not-yet-created paths, canonicalize the parent and rejoin the filename.
        let parent = expanded
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));

        let parent_canonical = parent.canonicalize().map_err(|e| {
            KurultaiError::security(format!("cannot canonicalize parent of '{}': {e}", path))
        })?;

        match expanded.file_name() {
            Some(name) => parent_canonical.join(name),
            None => parent_canonical,
        }
    };

    if contains_parent_traversal(&expanded) {
        return Err(KurultaiError::security(format!(
            "path traversal rejected: {path}"
        )));
    }

    Ok(canonical)
}

/// Ensure a path exists and is readable (for connector roots).
pub fn validate_readable_path(path: &str, label: &str) -> Result<PathBuf> {
    let resolved = resolve_allowed_path(path)?;
    if !resolved.exists() {
        return Err(KurultaiError::security(format!(
            "{label} path does not exist: {}",
            resolved.display()
        )));
    }
    if !resolved.is_dir() && !resolved.is_file() {
        return Err(KurultaiError::security(format!(
            "{label} path is not a file or directory: {}",
            resolved.display()
        )));
    }
    Ok(resolved)
}

fn expand_tilde(path: &str) -> Result<PathBuf> {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = dirs::home_dir()
            .ok_or_else(|| KurultaiError::security("could not resolve home directory"))?;
        Ok(home.join(rest))
    } else if path == "~" {
        dirs::home_dir().ok_or_else(|| KurultaiError::security("could not resolve home directory"))
    } else {
        Ok(PathBuf::from(path))
    }
}

fn contains_parent_traversal(path: &Path) -> bool {
    path.components().any(|c| matches!(c, Component::ParentDir))
}
