use anyhow::{Context, Result};
use std::str::FromStr;

use crate::database::Database;
use crate::database::models::ProjectState;
use crate::database::operations;
use crate::output::formatters;
use crate::cli::OutputFormat;

/// Execute the list command to display tracked projects
pub fn execute_list(
    db: &Database,
    state_filter: Option<&str>,
    format: OutputFormat,
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

    // Format output based on requested format
    let output = match format {
        OutputFormat::Table => formatters::format_table(projects),
        OutputFormat::Json => formatters::format_json(projects),
        OutputFormat::Tsv => formatters::format_tsv(projects),
    };

    // Print to stdout
    println!("{}", output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::schema;
    use rusqlite::Connection;
    use std::path::Path;
    use tempfile::TempDir;

    fn setup_test_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        schema::migrate(&conn)?;
        Ok(conn)
    }

    #[test]
    fn test_execute_list_empty() {
        let conn = setup_test_db().unwrap();

        // Should not error on empty database
        let result = execute_list(&conn, None, OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_list_with_projects() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Add a project
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&conn, "test-project", &project_path).unwrap();

        // List should succeed with all formats
        assert!(execute_list(&conn, None, OutputFormat::Table).is_ok());
        assert!(execute_list(&conn, None, OutputFormat::Json).is_ok());
        assert!(execute_list(&conn, None, OutputFormat::Tsv).is_ok());
    }

    #[test]
    fn test_execute_list_with_state_filter() {
        let conn = setup_test_db().unwrap();

        // Valid state filters should work
        assert!(execute_list(&conn, Some("active"), OutputFormat::Table).is_ok());
        assert!(execute_list(&conn, Some("inactive"), OutputFormat::Table).is_ok());
        assert!(execute_list(&conn, Some("archived"), OutputFormat::Table).is_ok());

        // Invalid state filter should error
        let result = execute_list(&conn, Some("invalid"), OutputFormat::Table);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid state filter"));
    }

    #[test]
    fn test_execute_list_case_insensitive_state() {
        let conn = setup_test_db().unwrap();

        // State filter should be case-insensitive
        assert!(execute_list(&conn, Some("ACTIVE"), OutputFormat::Table).is_ok());
        assert!(execute_list(&conn, Some("Inactive"), OutputFormat::Table).is_ok());
    }
}