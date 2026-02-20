use anyhow::{Context, Result};
use path_clean::PathClean;
use std::path::{Path, PathBuf};

/// Canonicalize a project path, handling ~ expansion and normalization
pub fn canonicalize_project_path<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();

    // Handle ~ expansion
    let expanded = expand_tilde(path);

    // Clean the path (normalize . and ..)
    let cleaned = expanded.clean();

    // Canonicalize to absolute path
    cleaned
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", path))
}

/// Expand ~ to user's home directory
fn expand_tilde(path: &Path) -> PathBuf {
    if let Some(path_str) = path.to_str() {
        if path_str.starts_with("~/") || path_str == "~" {
            if let Some(home) = directories::UserDirs::new() {
                let without_tilde = if path_str == "~" {
                    ""
                } else {
                    &path_str[2..]
                };
                return home.home_dir().join(without_tilde);
            }
        }
    }
    path.to_path_buf()
}

/// Normalize a path for database storage (always use forward slashes)
pub fn normalize_for_storage(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Check if a path is within the tracked directory
pub fn is_within_tracked_dir(path: &Path, tracked_dir: &Path) -> Result<bool> {
    let canonical_path = canonicalize_project_path(path)?;
    let canonical_tracked = canonicalize_project_path(tracked_dir)?;

    Ok(canonical_path.starts_with(canonical_tracked))
}