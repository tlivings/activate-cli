use anyhow::Result;
use std::path::PathBuf;

use crate::config::Config;
use crate::database::Database;

/// Discover new projects in the tracked directory
/// Returns list of newly discovered paths
pub fn discover_new_projects(_db: &Database, _config: &Config) -> Result<Vec<PathBuf>> {
    // Placeholder - will be implemented in Task 2
    Ok(vec![])
}

/// Add discovered projects to database as inactive
pub fn add_discovered_projects(_db: &Database, _paths: &[PathBuf]) -> Result<usize> {
    // Placeholder - will be implemented in Task 2
    Ok(0)
}
