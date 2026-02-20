use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::database::models::{Project, ProjectState};
use crate::utils::paths::canonicalize_project_path;

/// Add a new project to the database
pub fn add_project(conn: &Connection, name: &str, path: &Path) -> Result<i64> {
    // Canonicalize the path to ensure uniqueness
    let canonical_path =
        canonicalize_project_path(path).context("Failed to canonicalize project path")?;

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
        params![name, path_str, ProjectState::Inactive.to_string(), now],
    )
    .context("Failed to insert project into database")?;

    Ok(conn.last_insert_rowid())
}

/// Remove a project from the database by name
pub fn remove_project(conn: &Connection, name: &str) -> Result<bool> {
    let rows_affected = conn
        .execute(
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
        ProjectState::from_str(state).context("Invalid state filter")?;

        "SELECT id, name, path, state, last_touched, visit_count, git_origin, created_at, updated_at, ignored
         FROM projects
         WHERE state = ?1
         ORDER BY last_touched DESC"
    } else {
        "SELECT id, name, path, state, last_touched, visit_count, git_origin, created_at, updated_at, ignored
         FROM projects
         ORDER BY last_touched DESC"
    };

    let mut stmt = conn.prepare(query).context("Failed to prepare query")?;

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

/// Toggle the ignored status of a project
pub fn toggle_ignored(conn: &Connection, name: &str) -> Result<bool> {
    conn.execute(
        "UPDATE projects SET ignored = NOT ignored WHERE name = ?1 COLLATE NOCASE",
        params![name],
    )
    .context("Failed to toggle project ignored status")?;

    // Return the new ignored status
    let ignored: i64 = conn
        .query_row(
            "SELECT ignored FROM projects WHERE name = ?1 COLLATE NOCASE",
            params![name],
            |row| row.get(0),
        )
        .context("Failed to get ignored status")?;

    Ok(ignored != 0)
}

/// Get a project by name (case-insensitive)
pub fn get_project_by_name(conn: &Connection, name: &str) -> Result<Option<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, state, last_touched, visit_count, git_origin, created_at, updated_at, ignored
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

/// Update project state (active, inactive, archived)
pub fn update_project_state(conn: &Connection, name: &str, new_state: ProjectState) -> Result<()> {
    let rows_affected = conn
        .execute(
            "UPDATE projects SET state = ?1 WHERE name = ?2 COLLATE NOCASE",
            params![new_state.to_string(), name],
        )
        .context("Failed to update project state")?;

    if rows_affected == 0 {
        return Err(anyhow::anyhow!("Project '{}' not found", name));
    }

    Ok(())
}

/// Increment visit count and update last_touched timestamp
/// Used when activating a project
pub fn increment_visit_and_touch(conn: &Connection, name: &str) -> Result<()> {
    let now = Utc::now().timestamp();
    let rows_affected = conn.execute(
        "UPDATE projects SET visit_count = visit_count + 1, last_touched = ?1 WHERE name = ?2 COLLATE NOCASE",
        params![now, name],
    )
    .context("Failed to update project visit")?;

    if rows_affected == 0 {
        return Err(anyhow::anyhow!("Project '{}' not found", name));
    }

    Ok(())
}

/// Update the git origin URL for a project
pub fn update_git_origin(conn: &Connection, name: &str, origin: Option<&str>) -> Result<()> {
    let now = Utc::now().timestamp();
    let rows_affected = conn
        .execute(
            "UPDATE projects SET git_origin = ?, updated_at = ? WHERE name = ? COLLATE NOCASE",
            params![origin, now, name],
        )
        .context("Failed to update git origin")?;

    if rows_affected == 0 {
        return Err(anyhow::anyhow!("Project '{}' not found", name));
    }

    Ok(())
}

/// Add a new project with a specified initial state
/// Used for create-on-activate feature
pub fn add_project_with_state(
    conn: &Connection,
    name: &str,
    path: &PathBuf,
    state: ProjectState,
) -> Result<i64> {
    // Convert path to string for storage
    let path_str = path.to_string_lossy().to_string();

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

    // Check if name already exists
    let existing_name: Option<String> = conn
        .query_row(
            "SELECT name FROM projects WHERE name = ?1 COLLATE NOCASE",
            params![name],
            |row| row.get(0),
        )
        .ok();

    if existing_name.is_some() {
        return Err(anyhow::anyhow!(
            "Project with name '{}' already exists",
            name
        ));
    }

    // Insert the new project with specified state
    let now = Utc::now().timestamp();

    conn.execute(
        "INSERT INTO projects (name, path, state, last_touched)
         VALUES (?1, ?2, ?3, ?4)",
        params![name, path_str, state.to_string(), now],
    )
    .context("Failed to insert project into database")?;

    Ok(conn.last_insert_rowid())
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
        )
        .unwrap();

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
            )
            .unwrap();
        }

        let projects = list_projects(&conn, None).unwrap();

        // Should be ordered by last_touched DESC (most recent first)
        assert_eq!(projects[0].name, "project3");
        assert_eq!(projects[1].name, "project2");
        assert_eq!(projects[2].name, "project1");
    }

    #[test]
    fn test_update_project_state() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        add_project(&conn, "test-project", &project_path).unwrap();

        // Initial state should be inactive
        let project = get_project_by_name(&conn, "test-project").unwrap().unwrap();
        assert_eq!(project.state, ProjectState::Inactive);

        // Update to active
        update_project_state(&conn, "test-project", ProjectState::Active).unwrap();
        let project = get_project_by_name(&conn, "test-project").unwrap().unwrap();
        assert_eq!(project.state, ProjectState::Active);

        // Update to archived
        update_project_state(&conn, "test-project", ProjectState::Archived).unwrap();
        let project = get_project_by_name(&conn, "test-project").unwrap().unwrap();
        assert_eq!(project.state, ProjectState::Archived);
    }

    #[test]
    fn test_update_project_state_case_insensitive() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        add_project(&conn, "TestProject", &project_path).unwrap();

        // Update with different case
        update_project_state(&conn, "testproject", ProjectState::Active).unwrap();
        let project = get_project_by_name(&conn, "TestProject").unwrap().unwrap();
        assert_eq!(project.state, ProjectState::Active);
    }

    #[test]
    fn test_update_project_state_not_found() {
        let conn = setup_test_db().unwrap();
        let result = update_project_state(&conn, "nonexistent", ProjectState::Active);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_increment_visit_and_touch() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        add_project(&conn, "test-project", &project_path).unwrap();

        // Initial visit count should be 0
        let project = get_project_by_name(&conn, "test-project").unwrap().unwrap();
        assert_eq!(project.visit_count, 0);

        // Increment visit
        increment_visit_and_touch(&conn, "test-project").unwrap();
        let project = get_project_by_name(&conn, "test-project").unwrap().unwrap();
        assert_eq!(project.visit_count, 1);

        // Increment again
        increment_visit_and_touch(&conn, "test-project").unwrap();
        let project = get_project_by_name(&conn, "test-project").unwrap().unwrap();
        assert_eq!(project.visit_count, 2);
    }

    #[test]
    fn test_increment_visit_updates_last_touched() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        add_project(&conn, "test-project", &project_path).unwrap();

        let project_before = get_project_by_name(&conn, "test-project").unwrap().unwrap();
        let before_ts = project_before.last_touched;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(100));

        increment_visit_and_touch(&conn, "test-project").unwrap();
        let project_after = get_project_by_name(&conn, "test-project").unwrap().unwrap();

        // last_touched should be updated (at least equal or greater)
        assert!(project_after.last_touched >= before_ts);
    }

    #[test]
    fn test_increment_visit_not_found() {
        let conn = setup_test_db().unwrap();
        let result = increment_visit_and_touch(&conn, "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_add_project_with_state() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        let canonical = project_path.canonicalize().unwrap();
        let id = add_project_with_state(&conn, "test-project", &canonical, ProjectState::Active)
            .unwrap();
        assert!(id > 0);

        // Should be created with active state
        let project = get_project_by_name(&conn, "test-project").unwrap().unwrap();
        assert_eq!(project.state, ProjectState::Active);
    }

    #[test]
    fn test_add_project_with_state_archived() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        let canonical = project_path.canonicalize().unwrap();
        add_project_with_state(&conn, "test-project", &canonical, ProjectState::Archived).unwrap();

        let project = get_project_by_name(&conn, "test-project").unwrap().unwrap();
        assert_eq!(project.state, ProjectState::Archived);
    }

    #[test]
    fn test_add_project_with_state_duplicate_path() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_path).unwrap();

        let canonical = project_path.canonicalize().unwrap();
        add_project_with_state(&conn, "project1", &canonical, ProjectState::Active).unwrap();

        // Try to add again with same path but different name
        let result = add_project_with_state(&conn, "project2", &canonical, ProjectState::Active);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already tracked"));
    }

    #[test]
    fn test_add_project_with_state_duplicate_name() {
        let conn = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path1 = temp_dir.path().join("project1");
        let project_path2 = temp_dir.path().join("project2");
        std::fs::create_dir(&project_path1).unwrap();
        std::fs::create_dir(&project_path2).unwrap();

        let canonical1 = project_path1.canonicalize().unwrap();
        let canonical2 = project_path2.canonicalize().unwrap();

        add_project_with_state(&conn, "test-project", &canonical1, ProjectState::Active).unwrap();

        // Try to add again with same name but different path
        let result =
            add_project_with_state(&conn, "test-project", &canonical2, ProjectState::Active);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }
}
