use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;
use std::str::FromStr;

use crate::database::models::{Project, ProjectState};
use crate::utils::paths::canonicalize_project_path;

/// Add a new project to the database
pub fn add_project(conn: &Connection, name: &str, path: &Path) -> Result<i64> {
    // Canonicalize the path to ensure uniqueness
    let canonical_path = canonicalize_project_path(path)
        .context("Failed to canonicalize project path")?;

    // Convert path to string for storage
    let path_str = canonical_path.to_string_lossy().to_string();

    // Check if path already exists
    let existing: Option<String> = conn
        .query_row(
            "SELECT name FROM projects WHERE path = ?1",
            params![&path_str],
            |row| row.get(0),
        )
        .ok();

    if let Some(existing_name) = existing {
        return Err(anyhow::anyhow!(
            "Path already tracked as project '{}'",
            existing_name
        ));
    }

    // Insert the new project with initial state as 'inactive'
    let now = Utc::now().timestamp();

    conn.execute(
        "INSERT INTO projects (name, path, state, last_touched)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            name,
            path_str,
            ProjectState::Inactive.to_string(),
            now
        ],
    )
    .context("Failed to insert project into database")?;

    Ok(conn.last_insert_rowid())
}

/// Remove a project from the database by name
pub fn remove_project(conn: &Connection, name: &str) -> Result<bool> {
    let rows_affected = conn.execute(
        "DELETE FROM projects WHERE name = ?1 COLLATE NOCASE",
        params![name],
    )
    .context("Failed to delete project from database")?;

    Ok(rows_affected > 0)
}

/// List all projects, optionally filtered by state
pub fn list_projects(conn: &Connection, state_filter: Option<&str>) -> Result<Vec<Project>> {
    let query = if let Some(state) = state_filter {
        // Validate state
        ProjectState::from_str(state)
            .context("Invalid state filter")?;

        "SELECT id, name, path, state, last_touched, git_origin, created_at, updated_at
         FROM projects
         WHERE state = ?1
         ORDER BY last_touched DESC"
    } else {
        "SELECT id, name, path, state, last_touched, git_origin, created_at, updated_at
         FROM projects
         ORDER BY last_touched DESC"
    };

    let mut stmt = conn.prepare(query)
        .context("Failed to prepare query")?;

    let projects = if let Some(state) = state_filter {
        stmt.query_map(params![state], Project::from_row)?
    } else {
        stmt.query_map([], Project::from_row)?
    };

    let mut result = Vec::new();
    for project_result in projects {
        match project_result {
            Ok(project) => result.push(project),
            Err(e) => eprintln!("Warning: Failed to parse project row: {}", e),
        }
    }

    Ok(result)
}

/// Get a project by name (case-insensitive)
pub fn get_project_by_name(conn: &Connection, name: &str) -> Result<Option<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, state, last_touched, git_origin, created_at, updated_at
         FROM projects
         WHERE name = ?1 COLLATE NOCASE"
    )
    .context("Failed to prepare query")?;

    let mut projects = stmt.query_map(params![name], Project::from_row)?;

    match projects.next() {
        Some(result) => Ok(Some(result.context("Failed to fetch project")?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::schema;
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn setup_test_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        schema::migrate(&conn)?;
        Ok(conn)
    }

    #[test]
    fn test_add_project() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        let id = add_project(&conn, "test-project", &project_path).unwrap();
        assert!(id > 0);

        // Verify project was added
        let project = get_project_by_name(&conn, "test-project").unwrap().unwrap();
        assert_eq!(project.name, "test-project");
        assert_eq!(project.state, ProjectState::Inactive);
    }

    #[test]
    fn test_add_duplicate_path() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        add_project(&conn, "project1", &project_path).unwrap();
        let result = add_project(&conn, "project2", &project_path);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already tracked"));
    }

    #[test]
    fn test_remove_project() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        add_project(&conn, "test-project", &project_path).unwrap();

        // Remove the project
        let removed = remove_project(&conn, "test-project").unwrap();
        assert!(removed);

        // Verify it's gone
        let project = get_project_by_name(&conn, "test-project").unwrap();
        assert!(project.is_none());

        // Try to remove non-existent project
        let removed_again = remove_project(&conn, "test-project").unwrap();
        assert!(!removed_again);
    }

    #[test]
    fn test_remove_case_insensitive() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        add_project(&conn, "TestProject", &project_path).unwrap();

        // Remove with different case
        let removed = remove_project(&conn, "testproject").unwrap();
        assert!(removed);

        let project = get_project_by_name(&conn, "TestProject").unwrap();
        assert!(project.is_none());
    }

    #[test]
    fn test_list_projects() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Add multiple projects
        for i in 1..=3 {
            let project_path = temp_dir.path().join(format!("project{}", i));
            std::fs::create_dir(&project_path).unwrap();
            add_project(&conn, &format!("project{}", i), &project_path).unwrap();
        }

        let projects = list_projects(&conn, None).unwrap();
        assert_eq!(projects.len(), 3);
    }

    #[test]
    fn test_list_projects_with_filter() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Add projects and set different states
        for i in 1..=3 {
            let project_path = temp_dir.path().join(format!("project{}", i));
            std::fs::create_dir(&project_path).unwrap();
            add_project(&conn, &format!("project{}", i), &project_path).unwrap();
        }

        // Update one to active
        conn.execute(
            "UPDATE projects SET state = 'active' WHERE name = 'project1'",
            [],
        ).unwrap();

        // Filter by inactive
        let inactive = list_projects(&conn, Some("inactive")).unwrap();
        assert_eq!(inactive.len(), 2);

        // Filter by active
        let active = list_projects(&conn, Some("active")).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "project1");
    }

    #[test]
    fn test_get_project_by_name() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        add_project(&conn, "TestProject", &project_path).unwrap();

        // Case-insensitive lookup
        let project = get_project_by_name(&conn, "testproject").unwrap().unwrap();
        assert_eq!(project.name, "TestProject");

        // Non-existent project
        let missing = get_project_by_name(&conn, "missing").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn test_projects_ordered_by_last_touched() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Add projects with different timestamps
        for i in 1..=3 {
            let project_path = temp_dir.path().join(format!("project{}", i));
            std::fs::create_dir(&project_path).unwrap();
            add_project(&conn, &format!("project{}", i), &project_path).unwrap();

            // Update last_touched with different values
            conn.execute(
                "UPDATE projects SET last_touched = ?1 WHERE name = ?2",
                params![1000 + i * 100, format!("project{}", i)],
            ).unwrap();
        }

        let projects = list_projects(&conn, None).unwrap();

        // Should be ordered by last_touched DESC (most recent first)
        assert_eq!(projects[0].name, "project3");
        assert_eq!(projects[1].name, "project2");
        assert_eq!(projects[2].name, "project1");
    }
}