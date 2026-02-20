use anyhow::Result;

use crate::database::models::ProjectState;
use crate::database::operations::{list_projects, update_project_state};
use crate::database::Database;
use crate::navigation::ProjectMatcher;

/// Execute the archive command - mark a project as archived
pub fn execute_archive(db: &Database, name: &str) -> Result<()> {
    let project = find_project_or_error(db, name)?;
    update_project_state(&db.conn, &project.name, ProjectState::Archived)?;
    println!("Archived '{}'", project.name);
    Ok(())
}

/// Find a project by name (exact or fuzzy match) or return error
fn find_project_or_error(db: &Database, name: &str) -> Result<crate::database::models::Project> {
    let projects = list_projects(&db.conn, None)?;
    let matcher = ProjectMatcher::new();

    // Try exact match first
    if let Some(project) = matcher.find_exact(name, &projects) {
        return Ok(project);
    }

    // Try fuzzy match
    let results = matcher.match_projects(name, &projects);
    results
        .first()
        .map(|r| r.project.clone())
        .ok_or_else(|| anyhow::anyhow!("No project matching '{}' found", name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{operations, schema};
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn setup_test_db() -> Result<Database> {
        let conn = Connection::open_in_memory()?;
        schema::migrate(&conn)?;
        Ok(Database { conn })
    }

    #[test]
    fn test_archive_project() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-project", &project_path).unwrap();

        execute_archive(&db, "my-project").unwrap();

        let project = operations::get_project_by_name(&db.conn, "my-project")
            .unwrap()
            .unwrap();
        assert_eq!(project.state, ProjectState::Archived);
    }

    #[test]
    fn test_archive_fuzzy_match() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-awesome-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-awesome-project", &project_path).unwrap();

        // Archive with fuzzy name
        execute_archive(&db, "awesome").unwrap();

        let project = operations::get_project_by_name(&db.conn, "my-awesome-project")
            .unwrap()
            .unwrap();
        assert_eq!(project.state, ProjectState::Archived);
    }

    #[test]
    fn test_archive_not_found() {
        let db = setup_test_db().unwrap();
        let result = execute_archive(&db, "nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No project matching"));
    }
}
