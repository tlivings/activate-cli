use std::path::Path;

use anyhow::{Context, Result};
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};

/// Extract repository name from a git URL (HTTPS or SSH format).
///
/// Handles both:
/// - HTTPS: `https://github.com/user/repo.git`
/// - SSH: `git@github.com:user/repo.git`
///
/// Per CONTEXT.md: NO shorthand like "user/repo" - only full URLs.
pub fn extract_repo_name(url: &str) -> Option<String> {
    // Handle both formats by splitting on '/' or ':'
    // Strip trailing slashes and .git suffix first
    let name = url
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .rsplit(['/', ':'])
        .next()?;

    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// Clone a repository from a URL to a destination path.
///
/// Uses SSH agent for authentication when available, falls back to default credentials.
pub fn clone_repository(url: &str, dest: &Path) -> Result<Repository> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks.credentials(|_url, username_from_url, allowed_types| {
        // Try SSH agent first (for git@github.com URLs)
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            let username = username_from_url.unwrap_or("git");
            return Cred::ssh_key_from_agent(username);
        }

        // Fall back to default credentials (for HTTPS with credential helper)
        if allowed_types.contains(git2::CredentialType::DEFAULT) {
            return Cred::default();
        }

        // Try user/pass for HTTPS without credential helper
        if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            return Cred::default();
        }

        Err(git2::Error::from_str("no valid credentials found"))
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_opts);

    builder
        .clone(url, dest)
        .context("Failed to clone repository")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repo_name_https() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo.git"),
            Some("repo".to_string())
        );
    }

    #[test]
    fn test_extract_repo_name_https_no_git() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo"),
            Some("repo".to_string())
        );
    }

    #[test]
    fn test_extract_repo_name_ssh() {
        assert_eq!(
            extract_repo_name("git@github.com:user/repo.git"),
            Some("repo".to_string())
        );
    }

    #[test]
    fn test_extract_repo_name_trailing_slash() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo/"),
            Some("repo".to_string())
        );
    }

    #[test]
    fn test_extract_repo_name_empty() {
        assert_eq!(extract_repo_name(""), None);
    }

    #[test]
    fn test_extract_repo_name_ssh_no_git() {
        assert_eq!(
            extract_repo_name("git@github.com:user/repo"),
            Some("repo".to_string())
        );
    }

    #[test]
    fn test_extract_repo_name_gitlab() {
        assert_eq!(
            extract_repo_name("https://gitlab.com/group/subgroup/repo.git"),
            Some("repo".to_string())
        );
    }
}
