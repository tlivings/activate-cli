use std::path::Path;

use anyhow::Result;

/// Git repository status summary.
pub struct GitStatus {
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub unpushed_count: usize,
}

impl GitStatus {
    /// Check git status for a repository at the given path.
    pub fn check(_path: &Path) -> Result<Self> {
        todo!()
    }

    /// Returns true if there are any warnings (uncommitted changes or unpushed commits).
    pub fn has_warnings(&self) -> bool {
        todo!()
    }

    /// Format a warning message for the user.
    pub fn warning_message(&self) -> String {
        todo!()
    }
}
