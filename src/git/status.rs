use std::path::Path;

use anyhow::{Context, Result};
use git2::{Repository, Status};

/// Git repository status summary.
///
/// Tracks staged changes, unstaged changes, and unpushed commits.
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub unpushed_count: usize,
}

impl GitStatus {
    /// Check git status for a repository at the given path.
    ///
    /// Returns error if path is not a git repository.
    pub fn check(path: &Path) -> Result<Self> {
        let repo = Repository::open(path).context("Not a git repository")?;
        let statuses = repo.statuses(None).context("Failed to get git status")?;

        let mut staged = 0;
        let mut unstaged = 0;

        for entry in statuses.iter() {
            let status = entry.status();

            // INDEX_* flags = staged changes
            if status.intersects(
                Status::INDEX_NEW
                    | Status::INDEX_MODIFIED
                    | Status::INDEX_DELETED
                    | Status::INDEX_RENAMED,
            ) {
                staged += 1;
            }

            // WT_* flags = working tree (unstaged) changes
            // Note: We don't count WT_NEW (untracked files) per CONTEXT.md
            if status.intersects(Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_RENAMED) {
                unstaged += 1;
            }
        }

        let unpushed = count_unpushed_commits(&repo);

        Ok(GitStatus {
            staged_count: staged,
            unstaged_count: unstaged,
            unpushed_count: unpushed,
        })
    }

    /// Returns true if there are any warnings (uncommitted changes or unpushed commits).
    pub fn has_warnings(&self) -> bool {
        self.staged_count > 0 || self.unstaged_count > 0 || self.unpushed_count > 0
    }

    /// Format a warning message for the user.
    ///
    /// Per CONTEXT.md format: "3 uncommitted changes, 2 unpushed commits"
    /// Combines staged + unstaged as "uncommitted changes".
    #[allow(dead_code)]
    pub fn warning_message(&self) -> String {
        let changes = self.staged_count + self.unstaged_count;
        match (changes > 0, self.unpushed_count > 0) {
            (true, true) => format!(
                "{} uncommitted changes, {} unpushed commits",
                changes, self.unpushed_count
            ),
            (true, false) => format!("{} uncommitted changes", changes),
            (false, true) => format!("{} unpushed commits", self.unpushed_count),
            (false, false) => String::new(),
        }
    }
}

/// Count commits in HEAD that are not in the upstream tracking branch.
///
/// Returns 0 if:
/// - No HEAD (empty repository)
/// - No upstream branch configured
/// - Any error occurs (graceful degradation)
fn count_unpushed_commits(repo: &Repository) -> usize {
    // Get HEAD reference
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return 0, // No HEAD (empty repo)
    };

    // Get HEAD commit
    let head_commit = match head.peel_to_commit() {
        Ok(c) => c,
        Err(_) => return 0,
    };

    // Find the local branch from HEAD
    let branch_name = match head.shorthand() {
        Some(name) => name,
        None => return 0,
    };

    let branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(b) => b,
        Err(_) => return 0, // Not on a branch (detached HEAD)
    };

    // Find upstream tracking branch
    let upstream = match branch.upstream() {
        Ok(u) => u,
        Err(_) => return 0, // No upstream configured
    };

    let upstream_commit = match upstream.get().peel_to_commit() {
        Ok(c) => c,
        Err(_) => return 0,
    };

    // Count commits in HEAD but not in upstream using revwalk
    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(_) => return 0,
    };

    if revwalk.push(head_commit.id()).is_err() {
        return 0;
    }

    if revwalk.hide(upstream_commit.id()).is_err() {
        return 0;
    }

    revwalk.count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_default() {
        let status = GitStatus::default();
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.unstaged_count, 0);
        assert_eq!(status.unpushed_count, 0);
    }

    #[test]
    fn test_has_warnings_none() {
        let status = GitStatus::default();
        assert!(!status.has_warnings());
    }

    #[test]
    fn test_has_warnings_staged() {
        let status = GitStatus {
            staged_count: 1,
            ..Default::default()
        };
        assert!(status.has_warnings());
    }

    #[test]
    fn test_has_warnings_unstaged() {
        let status = GitStatus {
            unstaged_count: 1,
            ..Default::default()
        };
        assert!(status.has_warnings());
    }

    #[test]
    fn test_has_warnings_unpushed() {
        let status = GitStatus {
            unpushed_count: 1,
            ..Default::default()
        };
        assert!(status.has_warnings());
    }

    #[test]
    fn test_warning_message_empty() {
        let status = GitStatus::default();
        assert_eq!(status.warning_message(), "");
    }

    #[test]
    fn test_warning_message_uncommitted_only() {
        let status = GitStatus {
            staged_count: 2,
            unstaged_count: 1,
            unpushed_count: 0,
        };
        assert_eq!(status.warning_message(), "3 uncommitted changes");
    }

    #[test]
    fn test_warning_message_unpushed_only() {
        let status = GitStatus {
            staged_count: 0,
            unstaged_count: 0,
            unpushed_count: 5,
        };
        assert_eq!(status.warning_message(), "5 unpushed commits");
    }

    #[test]
    fn test_warning_message_both() {
        let status = GitStatus {
            staged_count: 2,
            unstaged_count: 1,
            unpushed_count: 4,
        };
        assert_eq!(
            status.warning_message(),
            "3 uncommitted changes, 4 unpushed commits"
        );
    }
}
