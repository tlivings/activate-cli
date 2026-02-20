use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::config::Config;
use crate::database::operations::{add_project, list_projects};
use crate::database::Database;

/// Discover new projects in the tracked directory
/// Returns list of newly discovered paths
pub fn discover_new_projects(db: &Database, config: &Config) -> Result<Vec<PathBuf>> {
    let tracked_dir = config.tracked_directory();

    if !tracked_dir.exists() {
        return Ok(vec![]);
    }

    // Get existing project paths
    let existing = list_projects(&db.conn, None)?;
    let existing_paths: HashSet<_> = existing.iter().map(|p| p.path.clone()).collect();

    // Scan tracked directory for immediate subdirectories
    let mut discovered = Vec::new();

    for entry in WalkDir::new(tracked_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.file_name(), &config.ignore_patterns))
    {
        let entry = entry.context("Failed to read directory entry")?;

        if !entry.file_type().is_dir() {
            continue;
        }

        let path = entry
            .path()
            .canonicalize()
            .context("Failed to canonicalize discovered path")?;

        if !existing_paths.contains(&path) {
            discovered.push(path);
        }
    }

    Ok(discovered)
}

fn is_ignored(name: &std::ffi::OsStr, patterns: &[String]) -> bool {
    let name_str = name.to_string_lossy();

    // Always ignore hidden directories
    if name_str.starts_with('.') {
        return true;
    }

    // Check ignore patterns
    patterns.iter().any(|p| name_str.contains(p))
}

/// Add discovered projects to database as inactive
pub fn add_discovered_projects(db: &Database, paths: &[PathBuf]) -> Result<usize> {
    let mut added = 0;

    for path in paths {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid directory name"))?;

        // Add as inactive - user must activate to mark active
        match add_project(&db.conn, name, path) {
            Ok(_) => added += 1,
            Err(e) => {
                // Log but continue - might be duplicate name
                eprintln!("Warning: Could not add '{}': {}", name, e);
            }
        }
    }

    Ok(added)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_ignored_hidden() {
        let patterns = vec![];
        assert!(is_ignored(std::ffi::OsStr::new(".git"), &patterns));
        assert!(is_ignored(std::ffi::OsStr::new(".hidden"), &patterns));
        assert!(!is_ignored(std::ffi::OsStr::new("project"), &patterns));
    }

    #[test]
    fn test_is_ignored_patterns() {
        let patterns = vec!["node_modules".to_string(), "target".to_string()];
        assert!(is_ignored(std::ffi::OsStr::new("node_modules"), &patterns));
        assert!(is_ignored(std::ffi::OsStr::new("target"), &patterns));
        assert!(!is_ignored(std::ffi::OsStr::new("src"), &patterns));
    }

    #[test]
    fn test_discovery_empty_dir() {
        use crate::database::schema;
        use rusqlite::Connection;

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("test.db");

        let conn = Connection::open(&db_path).unwrap();
        schema::migrate(&conn).unwrap();
        let db = Database { conn };

        let config = Config {
            tracked_directory: Some(temp.path().to_path_buf()),
            ignore_patterns: vec![],
            database_path: None,
        };

        let discovered = discover_new_projects(&db, &config).unwrap();
        assert!(discovered.is_empty());
    }

    #[test]
    fn test_discovery_finds_subdirs() {
        use crate::database::schema;
        use rusqlite::Connection;
        use std::fs;

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("test.db");

        // Create some subdirectories
        fs::create_dir(temp.path().join("project1")).unwrap();
        fs::create_dir(temp.path().join("project2")).unwrap();
        fs::create_dir(temp.path().join(".hidden")).unwrap();

        let conn = Connection::open(&db_path).unwrap();
        schema::migrate(&conn).unwrap();
        let db = Database { conn };

        let config = Config {
            tracked_directory: Some(temp.path().to_path_buf()),
            ignore_patterns: vec![],
            database_path: None,
        };

        let discovered = discover_new_projects(&db, &config).unwrap();
        assert_eq!(discovered.len(), 2); // Should not include .hidden
    }
}
