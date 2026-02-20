use anyhow::{bail, Context, Result};
use std::io::{self, Write};

use crate::config::Config;
use crate::database::operations::{
    add_project_with_state, increment_visit_and_touch, list_projects, update_project_state,
};
use crate::database::models::ProjectState;
use crate::database::Database;
use crate::navigation::ProjectMatcher;

/// Execute the activate command - find or create project, mark active, output path
///
/// This is the primary navigation command. It:
/// 1. Searches for project by name (exact then fuzzy match)
/// 2. If found: marks active, increments visit, outputs path
/// 3. If not found AND interactive (TTY): prompts to create
/// 4. If --create flag: creates without prompting
pub fn execute_activate(db: &Database, name: &str, create_flag: bool) -> Result<()> {
    let projects = list_projects(&db.conn, None)?;
    let matcher = ProjectMatcher::new();

    // Try exact match first (highest priority)
    if let Some(project) = matcher.find_exact(name, &projects) {
        activate_existing(db, &project.name)?;
        println!("{}", project.path.display());
        return Ok(());
    }

    // Try fuzzy match
    let results = matcher.match_projects(name, &projects);
    if let Some(best) = results.first() {
        activate_existing(db, &best.project.name)?;
        println!("{}", best.project.path.display());
        return Ok(());
    }

    // No match - offer to create
    create_new_project(db, name, create_flag)
}

/// Activate an existing project: update state and increment visit
fn activate_existing(db: &Database, name: &str) -> Result<()> {
    update_project_state(&db.conn, name, ProjectState::Active)?;
    increment_visit_and_touch(&db.conn, name)?;
    Ok(())
}

/// Create a new project when no match found
fn create_new_project(db: &Database, name: &str, force_create: bool) -> Result<()> {
    if !force_create {
        // Check if interactive terminal
        if !atty::is(atty::Stream::Stdin) {
            bail!("Project '{}' not found. Use --create to create it.", name);
        }

        eprint!("Project '{}' not found. Create it? [y/N]: ", name);
        io::stderr().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            bail!("Project creation cancelled");
        }
    }

    // Load config to get tracked directory
    let config = Config::load()?;
    let new_path = config.tracked_directory.join(name);

    // Create directory
    std::fs::create_dir_all(&new_path)
        .with_context(|| format!("Failed to create directory: {}", new_path.display()))?;

    // Canonicalize the path
    let canonical = new_path
        .canonicalize()
        .context("Failed to canonicalize new project path")?;

    // Add to database as active
    add_project_with_state(&db.conn, name, &canonical, ProjectState::Active)?;
    increment_visit_and_touch(&db.conn, name)?;

    eprintln!("Created project '{}' at {}", name, canonical.display());
    println!("{}", canonical.display());
    Ok(())
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
    fn test_activate_existing_project() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-project", &project_path).unwrap();

        // Activate should succeed
        activate_existing(&db, "my-project").unwrap();

        // State should be active
        let project = operations::get_project_by_name(&db.conn, "my-project")
            .unwrap()
            .unwrap();
        assert_eq!(project.state, ProjectState::Active);
        assert_eq!(project.visit_count, 1);
    }

    #[test]
    fn test_activate_increments_visit() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-project", &project_path).unwrap();

        // Activate multiple times
        activate_existing(&db, "my-project").unwrap();
        activate_existing(&db, "my-project").unwrap();
        activate_existing(&db, "my-project").unwrap();

        let project = operations::get_project_by_name(&db.conn, "my-project")
            .unwrap()
            .unwrap();
        assert_eq!(project.visit_count, 3);
    }

    #[test]
    fn test_activate_sets_state_to_active() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-project", &project_path).unwrap();

        // Initially inactive
        let project = operations::get_project_by_name(&db.conn, "my-project")
            .unwrap()
            .unwrap();
        assert_eq!(project.state, ProjectState::Inactive);

        activate_existing(&db, "my-project").unwrap();

        // Now active
        let project = operations::get_project_by_name(&db.conn, "my-project")
            .unwrap()
            .unwrap();
        assert_eq!(project.state, ProjectState::Active);
    }
}
