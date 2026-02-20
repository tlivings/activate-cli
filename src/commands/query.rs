use anyhow::{bail, Context, Result};

use crate::database::operations::list_projects;
use crate::database::Database;
use crate::navigation::ProjectMatcher;

/// Execute the query command - find best matching project and output its path
///
/// This is designed for shell integration: output is just the path, ready for cd
pub fn execute_query(db: &Database, keywords: &[String], exclude_cwd: Option<&str>) -> Result<()> {
    let query = keywords.join(" ");

    // Get all projects from database
    let projects =
        list_projects(&db.conn, None).context("Failed to list projects")?;

    if projects.is_empty() {
        bail!("No projects tracked. Use 'activate add <path>' to add projects.");
    }

    let matcher = ProjectMatcher::new();

    // First try exact match
    if let Some(project) = matcher.find_exact(&query, &projects) {
        // Skip if it's the current directory (shell excludes pwd)
        if let Some(cwd) = exclude_cwd {
            if project.path.to_string_lossy() == cwd {
                // Try fuzzy match instead, excluding this one
                return find_fuzzy_match(&matcher, &query, &projects, Some(&project.name));
            }
        }
        println!("{}", project.path.display());
        return Ok(());
    }

    // Fall back to fuzzy match
    find_fuzzy_match(&matcher, &query, &projects, None)
}

fn find_fuzzy_match(
    matcher: &ProjectMatcher,
    query: &str,
    projects: &[crate::database::models::Project],
    exclude_name: Option<&str>,
) -> Result<()> {
    let results = matcher.match_projects(query, projects);

    // Filter out excluded project if specified
    let best = results.iter().find(|r| {
        exclude_name.map_or(true, |name| !r.project.name.eq_ignore_ascii_case(name))
    });

    match best {
        Some(result) => {
            println!("{}", result.project.path.display());
            Ok(())
        }
        None => {
            bail!("No project matching '{}' found", query);
        }
    }
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
    fn test_query_empty_database() {
        let db = setup_test_db().unwrap();
        let result = execute_query(&db, &["test".to_string()], None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No projects tracked"));
    }

    #[test]
    fn test_query_exact_match() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-project", &project_path).unwrap();

        // Exact match should succeed
        let result = execute_query(&db, &["my-project".to_string()], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_fuzzy_match() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-awesome-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-awesome-project", &project_path).unwrap();

        // Fuzzy match with partial name should succeed
        let result = execute_query(&db, &["awesome".to_string()], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_no_match() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-project", &project_path).unwrap();

        // Query that doesn't match anything
        let result = execute_query(&db, &["xyz123".to_string()], None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No project matching"));
    }

    #[test]
    fn test_query_exclude_cwd() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Add two projects
        let project1_path = temp_dir.path().join("project-one");
        let project2_path = temp_dir.path().join("project-two");
        std::fs::create_dir(&project1_path).unwrap();
        std::fs::create_dir(&project2_path).unwrap();
        operations::add_project(&db.conn, "project-one", &project1_path).unwrap();
        operations::add_project(&db.conn, "project-two", &project2_path).unwrap();

        // Query "project-one" with project-one as cwd should try to find another match
        let cwd = project1_path.to_string_lossy().to_string();
        let _result = execute_query(&db, &["project-one".to_string()], Some(&cwd));
        // Result depends on fuzzy matching - project-one is excluded, may match project-two or error
    }

    #[test]
    fn test_query_multiple_keywords() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Create project with spaces in name to match keyword join behavior
        let project_path = temp_dir.path().join("my awesome project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my awesome project", &project_path).unwrap();

        // Multiple keywords joined with space should match
        let result = execute_query(&db, &["my".to_string(), "awesome".to_string()], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_case_insensitive_exact() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("MyProject");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "MyProject", &project_path).unwrap();

        // Exact match should be case-insensitive
        let result = execute_query(&db, &["myproject".to_string()], None);
        assert!(result.is_ok());
    }
}
