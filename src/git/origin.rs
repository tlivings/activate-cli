use std::path::Path;

use git2::Repository;

/// Detect the origin URL from a git repository.
///
/// Tries "origin" remote first (most common), then falls back to
/// the first remote found with a URL.
///
/// Returns `None` if:
/// - Path is not a git repository
/// - Repository has no remotes
/// - No remote has a URL configured
///
/// Per CONTEXT.md: non-git projects show "local" in origin column -
/// this function returns None for those (caller displays "local").
pub fn detect_origin(path: &Path) -> Option<String> {
    let repo = Repository::open(path).ok()?;

    // Try "origin" first (most common)
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            return Some(url.to_string());
        }
    }

    // Fall back to first remote with a URL
    let remotes = repo.remotes().ok()?;
    for name in remotes.iter().flatten() {
        if let Ok(remote) = repo.find_remote(name) {
            if let Some(url) = remote.url() {
                return Some(url.to_string());
            }
        }
    }

    None
}

/// Check if a path is a git repository.
///
/// Useful for distinguishing git vs non-git projects.
pub fn is_git_repo(path: &Path) -> bool {
    Repository::open(path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_git_repo_false_for_nonexistent() {
        assert!(!is_git_repo(&PathBuf::from("/nonexistent/path")));
    }

    #[test]
    fn test_detect_origin_none_for_nonexistent() {
        assert_eq!(detect_origin(&PathBuf::from("/nonexistent/path")), None);
    }

    #[test]
    fn test_is_git_repo_current_dir() {
        // This test relies on the project being a git repo
        let current = std::env::current_dir().unwrap();
        // If running from project root, this should be a git repo
        if current.join(".git").exists() {
            assert!(is_git_repo(&current));
        }
    }

    #[test]
    fn test_detect_origin_current_dir() {
        // This test relies on the project being a git repo with an origin
        let current = std::env::current_dir().unwrap();
        if current.join(".git").exists() {
            // May or may not have origin depending on clone method
            let _ = detect_origin(&current);
        }
    }
}
