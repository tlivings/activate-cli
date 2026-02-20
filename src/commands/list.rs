use anyhow::{Context, Result};
use std::str::FromStr;

use crate::database::Database;
use crate::database::models::ProjectState;
use crate::database::operations;

/// Execute the list command to display tracked projects
/// Default: one name per line (pipe-friendly)
/// --paths: show full paths instead of names
/// --json: full project details as JSON
pub fn execute_list(
    db: &Database,
    state_filter: Option<&str>,
    json: bool,
    paths: bool,
) -> Result<()> {
    // Validate state filter if provided
    if let Some(state) = state_filter {
        ProjectState::from_str(state)
            .context(format!(
                "Invalid state filter '{}'. Valid options: active, inactive, archived",
                state
            ))?;
    }

    // Retrieve projects from database
    let projects = operations::list_projects(&db.conn, state_filter)
        .context("Failed to retrieve projects from database")?;

    if json {
        // Full JSON output
        let json_str = serde_json::to_string_pretty(&projects)?;
        println!("{}", json_str);
    } else if paths {
        // Full paths, one per line
        for project in &projects {
            println!("{}", project.path.display());
        }
    } else {
        // Simple: one name per line (most useful for piping)
        for project in &projects {
            println!("{}", project.name);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::schema;
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn setup_test_db() -> Result<Database> {
        let conn = Connection::open_in_memory()?;
        schema::migrate(&conn)?;
        Ok(Database { conn })
    }

    #[test]
    fn test_execute_list_empty() {
        let db = setup_test_db().unwrap();

        // Should not error on empty database
        let result = execute_list(&db, None, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_list_with_projects() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Add a project
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "test-project", &project_path).unwrap();

        // List should succeed with all formats
        assert!(execute_list(&db, None, false, false).is_ok()); // names
        assert!(execute_list(&db, None, false, true).is_ok());  // paths
        assert!(execute_list(&db, None, true, false).is_ok());  // json
    }

    #[test]
    fn test_execute_list_with_state_filter() {
        let db = setup_test_db().unwrap();

        // Valid state filters should work
        assert!(execute_list(&db, Some("active"), false, false).is_ok());
        assert!(execute_list(&db, Some("inactive"), false, false).is_ok());
        assert!(execute_list(&db, Some("archived"), false, false).is_ok());

        // Invalid state filter should error
        let result = execute_list(&db, Some("invalid"), false, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid state filter"));
    }

    #[test]
    fn test_execute_list_case_insensitive_state() {
        let db = setup_test_db().unwrap();

        // State filter should be case-insensitive
        assert!(execute_list(&db, Some("ACTIVE"), false, false).is_ok());
        assert!(execute_list(&db, Some("Inactive"), false, false).is_ok());
    }
}